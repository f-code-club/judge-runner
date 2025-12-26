use std::{
    env, fs,
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    process::Stdio,
    thread,
    time::Duration,
};

use state_shift::{impl_state, type_state};
use uuid::Uuid;

use crate::{Language, Resource, Sandbox, Verdict};

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

#[impl_state]
impl Judge {
    #[require(Created)]
    pub fn new(main: (&[u8], Language), checker: Option<(&[u8], Language)>) -> io::Result<Judge> {
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
        })
    }

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
    ) -> io::Result<Verdict> {
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
            .spawn()?;
        let mut cin = checker.stdin.take().unwrap();
        let cout = checker.stdout.take().unwrap();

        let sandbox = Sandbox::new(resource, time_limit)?;
        let mut submission_command = language.get_run_command(MAIN);
        submission_command
            .current_dir(&project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        let mut submission = sandbox.spawn(submission_command)?;
        let mut sin = submission.stdin.take().unwrap();
        let sout = submission.stdout.take().unwrap();

        let monitor_thread = thread::spawn(move || sandbox.monitor(submission));

        if !is_interactive {
            sin.write_all(input)?;
            sin.write_all(b"\n")?;
            sin.flush()?;
        }
        cin.write_all(input)?;
        cin.write_all(b"\n")?;
        cin.flush()?;

        forward(cout, sin);
        forward(sout, cin);

        if let Some(verdict) = monitor_thread.join().unwrap()? {
            return Ok(verdict);
        }

        let status = checker.wait()?;
        let verdict = if status.success() {
            Verdict::Accepted
        } else {
            Verdict::WrongAnswer
        };

        Ok(verdict)
    }
}

fn forward<R: Read + Send + 'static, W: Write + Send + 'static>(
    mut reader: R,
    mut writer: W,
) -> thread::JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                drop(writer);
                break;
            }
            writer.write_all(&buffer[..n])?;
            writer.flush()?;
        }

        Ok(())
    })
}
