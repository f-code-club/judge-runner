use std::{
    env,
    hash::{DefaultHasher, Hash, Hasher},
    io,
    marker::PhantomData,
    path::PathBuf,
    process::Stdio,
    time::Duration,
};

use bon::bon;
use byte_unit::Byte;
use state_shift::{impl_state, type_state};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    time::sleep,
};
use uuid::Uuid;

use crate::{Language, Metrics, Resource, Sandbox, Verdict};

const MAIN: &str = "main";
const CHECKER: &str = "checker";
const BUFFER_SIZE: usize = 512;

#[type_state(
    states = (Created, Compiled),
    slots = (Created)
)]
#[derive(Default)]
pub struct Judge {
    pub project_path: PathBuf,
    pub language: Language,
    pub checker_language: Option<Language>,
}

#[bon]
impl Judge<Created> {
    #[builder]
    pub async fn new<'a>(
        #[rustfmt::skip]
        #[builder(with = |code: &'a [u8], language: Language| (code, language))]
        main: (&'a [u8], Language),

        #[rustfmt::skip]
        #[builder(with = |code: &'a [u8], language: Language| (code, language))]
        checker: Option<(&'a [u8], Language)>,
    ) -> io::Result<Judge<Created>> {
        let mut hasher = DefaultHasher::default();
        main.0.hash(&mut hasher);
        Uuid::new_v4().hash(&mut hasher);
        let id = hasher.finish();
        let project_path = env::temp_dir().join(id.to_string());
        fs::create_dir(&project_path).await?;

        tokio::try_join! {
            async {
                let (code, language) = main;
                let main_path = project_path.join(MAIN).with_extension(language.extension);
                fs::write(&main_path, code).await?;

                Ok::<_, io::Error>(())
            },
            async {
                if let Some((code, language)) = checker {
                    let mut checker_path = project_path.join(CHECKER);
                    if language.is_interpreted() {
                        checker_path.set_extension(language.extension);
                    }
                    let mut checker_file = fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .mode(0o755)
                        .open(&checker_path)
                    .await?;
                    checker_file.write_all(code).await?;
                }

                Ok(())
            }
        }?;

        Ok(Judge {
            project_path,
            language: main.1,
            checker_language: checker.map(|checker| checker.1),
            _state: PhantomData,
        })
    }
}

#[impl_state]
impl Judge {
    #[require(Created)]
    #[switch_to(Compiled)]
    pub async fn compile(self) -> io::Result<Result<Judge<Compiled>, Verdict>> {
        if let Some(mut cmd) = self.language.get_compile_command(MAIN) {
            let mut process = cmd.current_dir(&self.project_path).spawn()?;
            let status = process.wait().await?;
            if !status.success() {
                return Ok(Err(Verdict::CompilationError));
            }
        }

        Ok(Ok(Judge {
            project_path: self.project_path,
            language: self.language,
            checker_language: self.checker_language,
        }))
    }

    #[require(Compiled)]
    pub async fn read_executable(&self) -> io::Result<Vec<u8>> {
        let mut path = self.project_path.join(MAIN);
        if self.language.is_interpreted() {
            path.set_extension(self.language.extension);
        }

        fs::read(path).await
    }

    #[require(Compiled)]
    pub async fn run(
        &self,
        input: &[u8],
        is_interactive: bool,
        resource: Resource,
        time_limit: Duration,
    ) -> io::Result<Metrics> {
        self.project_path
            .read_dir()
            .unwrap()
            .for_each(|x| println!("{:?}", x));
        let Judge {
            project_path,
            language,
            checker_language,
            ..
        } = self;

        let checker_language = checker_language.ok_or(io::Error::other("Missing checker"))?;
        let mut checker = checker_language
            .get_run_command(CHECKER)
            .current_dir(project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut cstdin = checker.stdin.take().unwrap();
        let mut cstdout = checker.stdout.take().unwrap();

        let sandbox = Sandbox::new(resource, time_limit).unwrap();
        let mut cmd = language.get_run_command(MAIN);
        cmd.current_dir(project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut main = sandbox.spawn(cmd).unwrap();
        let mut stdin = main.stdin.take().unwrap();
        let mut stdout = main.stdout.take().unwrap();
        let mut stderr = main.stderr.take().unwrap();

        let monitor = tokio::spawn(async move { sandbox.monitor(main).await });

        tokio::try_join! {
            async {
                if !is_interactive {
                    stdin.write_all(input).await?;
                    stdin.write_all(b"\n").await?;
                    stdin.flush().await?;
                }

                Ok::<_, io::Error>(())
            },
            async {
                cstdin.write_all(input).await?;
                cstdin.write_all(b"\n").await?;
                cstdin.flush().await?;

                Ok::<_, io::Error>(())
            }
        }?;

        let mut out: Vec<u8> = vec![];
        let mut err: Vec<u8> = vec![];
        let mut verdict: Option<Verdict> = None;
        let mut run_time: Duration = Duration::default();
        let mut memory_usage: Byte = Byte::default();
        tokio::select! {
            monitor_result = monitor => {
                let monitor_result = monitor_result.unwrap().unwrap();
                (verdict, run_time, memory_usage) = monitor_result;
            }
            err = async {
                let mut buffer = [0u8; BUFFER_SIZE];
                loop {
                    let n = stdout.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    if cstdin.write_all(&buffer[..n]).await.is_err() {
                        break;
                    }
                    cstdin.flush().await?;
                    out.extend_from_slice(&buffer[0..n]);
                }

                // sleep indefinitely until sandbox return
                sleep(Duration::MAX).await;

                Ok::<_, io::Error>(())
            } => { err? }
            err = async {
                let mut buffer = [0u8; BUFFER_SIZE];
                loop {
                    let n = cstdout.read(&mut buffer).await?;
                    if n == 0 { break; }
                    if stdin.write_all(&buffer[..n]).await.is_err() {
                        break;
                    }
                    stdin.flush().await?;
                }

                // sleep indefinitely until sandbox return
                sleep(Duration::MAX).await;

                Ok::<_, io::Error>(())
            } => { err? }
            err = async {
                let mut buffer = [0u8; BUFFER_SIZE];
                loop {
                    let n = stderr.read(&mut buffer).await?;
                    if n == 0 { break; }
                    err.extend_from_slice(&buffer[0..n]);
                }

                // sleep indefinitely until sandbox return
                sleep(Duration::MAX).await;

                Ok::<_, io::Error>(())
            } => { err? }
        };
        let out = String::from_utf8(out).map_err(io::Error::other)?;
        let err = String::from_utf8(err).map_err(io::Error::other)?;

        if let Some(verdict) = verdict {
            return Ok(Metrics {
                verdict,
                run_time,
                stdout: out,
                stderr: err,
                memory_usage,
            });
        }

        let status = checker.wait().await?;
        let verdict = if status.success() {
            Verdict::Accepted
        } else {
            Verdict::WrongAnswer
        };

        Ok(Metrics {
            verdict,
            run_time,
            stdout: out,
            stderr: err,
            memory_usage,
        })
    }
}
