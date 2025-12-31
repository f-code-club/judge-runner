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

pub struct Code<'a> {
    pub language: Language,
    pub content: &'a [u8],
}

#[type_state(
    states = (Created, Compiled),
    slots = (Created)
)]
#[derive(Default)]
pub struct Judge {
    pub project_path: PathBuf,
    pub language: Language,
    pub checker_language: Option<Language>,
    pub is_interactive: bool,
    pub resource: Resource,
    pub time_limit: Duration,
}

#[bon]
impl Judge<Created> {
    #[builder]
    pub async fn new<'a>(
        main: Code<'a>,
        checker: Option<Code<'a>>,
        #[builder(default = false, name = "interactive")] is_interactive: bool,
        #[builder(default)] resource: Resource,
        #[builder(default)] time_limit: Duration,
    ) -> io::Result<Judge<Created>> {
        let project_path = generate_project_path(main.content);
        fs::create_dir(&project_path).await?;

        tokio::try_join! {
            async {
                let main_path = project_path.join(MAIN).with_extension(main.language.extension);
                fs::write(&main_path, main.content).await?;

                Ok::<_, io::Error>(())
            },
            async {
                if let Some(checker) = &checker {
                    let mut checker_path = project_path.join(CHECKER);
                    if checker.language.is_interpreted() {
                        checker_path.set_extension(checker.language.extension);
                    }
                    let mut checker_file = fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .mode(0o755)
                        .open(&checker_path)
                    .await?;
                    checker_file.write_all(checker.content).await?;
                }

                Ok(())
            }
        }?;

        Ok(Judge {
            project_path,
            language: main.language,
            checker_language: checker.map(|checker| checker.language),
            is_interactive,
            resource,
            time_limit,
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
            is_interactive: self.is_interactive,
            resource: self.resource,
            time_limit: self.time_limit,
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
    pub async fn run(&self, input: &[u8]) -> io::Result<Metrics> {
        let checker_language = self
            .checker_language
            .ok_or(io::Error::other("Missing checker"))?;
        let mut checker = checker_language
            .get_run_command(CHECKER)
            .current_dir(&self.project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut cstdin = checker.stdin.take().unwrap();
        let mut cstdout = checker.stdout.take().unwrap();

        let sandbox = Sandbox::new(self.resource, self.time_limit)?;
        let mut cmd = self.language.get_run_command(MAIN);
        cmd.current_dir(&self.project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut main = sandbox.spawn(cmd)?;
        let mut stdin = main.stdin.take().unwrap();
        let mut stdout = main.stdout.take().unwrap();
        let mut stderr = main.stderr.take().unwrap();

        let monitor = tokio::spawn(async move { sandbox.monitor(main).await });

        tokio::try_join! {
            async {
                if !self.is_interactive {
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
                let monitor_result = monitor_result.unwrap()?;
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
                if !self.is_interactive {
                    drop(stdin);
                } else {
                    let mut buffer = [0u8; BUFFER_SIZE];
                    loop {
                        let n = cstdout.read(&mut buffer).await?;
                        if n == 0 { break; }
                        if stdin.write_all(&buffer[..n]).await.is_err() {
                            break;
                        }
                        stdin.flush().await?;
                    }

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

fn generate_project_path(content: &[u8]) -> PathBuf {
    let mut hasher = DefaultHasher::default();
    content.hash(&mut hasher);
    Uuid::new_v4().hash(&mut hasher);
    let id = hasher.finish();

    env::temp_dir().join(id.to_string())
}
