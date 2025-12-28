use std::{
    env, fs,
    io::{self, Read, Write},
    marker::PhantomData,
    os::{fd::AsFd, unix::fs::OpenOptionsExt},
    path::PathBuf,
    process::Stdio,
    thread,
    time::Duration,
};

use bon::bon;
use state_shift::{impl_state, type_state};
use timeout_readwrite::{TimeoutReadExt, TimeoutReader};
use uuid::Uuid;

use crate::{Language, Metrics, Resource, Sandbox, Verdict};

const MAIN: &str = "main";
const CHECKER: &str = "checker";
const BUFFER_SIZE: usize = 512;
const READ_TIMEOUT: Duration = Duration::from_millis(100);

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
    pub fn new<'a>(
        #[rustfmt::skip]
        #[builder(with = |code: &'a [u8], language: Language| (code, language))]
        main: (&'a [u8], Language),

        #[rustfmt::skip]
        #[builder(with = |code: &'a [u8], language: Language| (code, language))]
        checker: Option<(&'a [u8], Language)>,
    ) -> io::Result<Judge<Created>> {
        let project_path = env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir(&project_path)?;

        let (code, language) = main;
        let main_path = project_path.join(MAIN).with_extension(language.extension);
        fs::write(&main_path, code)?;

        let checker_language = if let Some((code, language)) = checker {
            let mut checker_path = project_path.join(CHECKER);
            if language.is_interpreted() {
                checker_path.set_extension(language.extension);
            }
            let mut checker_file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .mode(0o755)
                .open(&checker_path)?;
            checker_file.write_all(code)?;

            Some(language)
        } else {
            None
        };

        Ok(Judge {
            project_path,
            language,
            checker_language,
            _state: PhantomData,
        })
    }
}

#[impl_state]
impl Judge {
    #[require(Created)]
    #[switch_to(Compiled)]
    pub fn compile(self) -> io::Result<Result<Judge<Compiled>, Verdict>> {
        if let Some(mut cmd) = self.language.get_compile_command(MAIN) {
            let mut process = cmd.current_dir(&self.project_path).spawn()?;
            let status = process.wait()?;
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
    pub fn read_executable(&self) -> io::Result<Vec<u8>> {
        let mut path = self.project_path.join(MAIN);
        if self.language.is_interpreted() {
            path.set_extension(self.language.extension);
        }

        fs::read(path)
    }

    #[require(Compiled)]
    pub fn run(
        self,
        input: &[u8],
        is_interactive: bool,
        resource: Resource,
        time_limit: Duration,
    ) -> io::Result<Metrics> {
        let Judge {
            project_path,
            language,
            checker_language,
            ..
        } = self;

        let checker_language = checker_language.ok_or(io::Error::other("Missing checker"))?;
        let mut checker = checker_language
            .get_run_command(CHECKER)
            .current_dir(&project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut cstdin = checker.stdin.take().unwrap();
        let cstdout = checker.stdout.take().unwrap().with_timeout(READ_TIMEOUT);

        let sandbox = Sandbox::new(resource, time_limit)?;
        let mut cmd = language.get_run_command(MAIN);
        cmd.current_dir(&project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut main = sandbox.spawn(cmd)?;
        let mut stdin = main.stdin.take().unwrap();
        let stdout = main.stdout.take().unwrap().with_timeout(READ_TIMEOUT);
        let mut stderr = main.stderr.take().unwrap().with_timeout(READ_TIMEOUT);

        let ((verdict, run_time, memory_usage), stdout) = thread::scope(|scope| {
            let monitor_thread = scope.spawn(|| sandbox.monitor(main));

            if !is_interactive {
                stdin.write_all(input)?;
                stdin.write_all(b"\n")?;
                stdin.flush()?;
            }
            cstdin.write_all(input)?;
            cstdin.write_all(b"\n")?;
            cstdin.flush()?;

            scope.spawn(|| forward(cstdout, stdin));
            let main_to_checker = scope.spawn(|| forward(stdout, cstdin));

            let monitor_result = match monitor_thread.join().unwrap() {
                Ok(v) => v,
                Err(err) => return Err(err),
            };
            let output = main_to_checker.join().unwrap()?;
            let output = String::from_utf8(output).map_err(io::Error::other)?;

            Ok((monitor_result, output))
        })?;
        let mut err = String::new();
        // TODO: handle error
        let _ = stderr.read_to_string(&mut err);

        if let Some(verdict) = verdict {
            return Ok(Metrics {
                verdict,
                run_time,
                stdout,
                stderr: err,
                memory_usage,
            });
        }

        let status = checker.wait()?;
        let verdict = if status.success() {
            Verdict::Accepted
        } else {
            Verdict::WrongAnswer
        };

        Ok(Metrics {
            verdict,
            run_time,
            stdout,
            stderr: err,
            memory_usage,
        })
    }
}

fn forward<R: Read + AsFd, W: Write>(
    mut reader: TimeoutReader<R>,
    mut writer: W,
) -> io::Result<Vec<u8>> {
    let mut stdout: Vec<u8> = vec![];
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        // TODO:handle error
        let n = match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if writer.write_all(&buffer[..n]).is_err() {
            break;
        }
        stdout.extend_from_slice(&buffer[0..n]);
        writer.flush()?;
    }
    drop(writer);

    Ok(stdout)
}
