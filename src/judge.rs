use std::{
    env, fs,
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    process::Stdio,
    thread,
    time::Duration,
};

use uuid::Uuid;

use crate::{Language, Resource, Sandbox, Verdict};

const SUBMISSION: &str = "main";
const CHECKER: &str = "checker";
const BUFFER_SIZE: usize = 512;

pub struct Judge {
    pub project_path: PathBuf,
    pub language: Language,
}

impl Judge {
    pub fn new(code: &[u8], checker: &[u8], language: Language) -> io::Result<Judge> {
        let project_path = env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir(&project_path)?;
        let main_path = project_path
            .join(SUBMISSION)
            .with_extension(language.extension);
        let checker_path = project_path.join(CHECKER);

        fs::write(&main_path, code)?;
        let mut checker_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o755)
            .open(&checker_path)?;
        checker_file.write_all(checker)?;

        Ok(Judge {
            project_path,
            language,
        })
    }

    pub fn compile(&self) -> io::Result<Option<Verdict>> {
        let Some(mut command) = self.language.get_compile_command(SUBMISSION) else {
            return Ok(None);
        };
        let mut process = command.current_dir(&self.project_path).spawn()?;
        let status = process.wait()?;
        if !status.success() {
            return Ok(Some(Verdict::CompilationError));
        }

        Ok(None)
    }

    pub fn run(
        self,
        input: &[u8],
        is_interactive: bool,
        resource: Resource,
        time_limit: Duration,
    ) -> io::Result<Verdict> {
        let mut checker = self
            .language
            .get_run_command(CHECKER)
            .current_dir(&self.project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let mut cin = checker.stdin.take().unwrap();
        let cout = checker.stdout.take().unwrap();

        let mut submission = self
            .language
            .get_run_command(SUBMISSION)
            .current_dir(&self.project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let mut sin = submission.stdin.take().unwrap();
        let sout = submission.stdout.take().unwrap();

        let monitor_thread = thread::spawn(move || {
            let sandbox = Sandbox::new(resource, time_limit)?;
            sandbox.monitor(submission)
        });

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
