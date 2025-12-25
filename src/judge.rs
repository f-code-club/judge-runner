use std::{env, fs, io, path::PathBuf, time::Duration};

use nix::{sys::stat, unistd};
use state_shift::{impl_state, type_state};
use uuid::Uuid;

use crate::{Language, Resource, Sandbox, Verdict};

const SUBMISSION: &str = "main";
const CHECKER: &str = "checker";
const INPUT: &str = "input";
const OUTPUT: &str = "output";

#[type_state(
    states = (Created, Compiled),
    slots = (Created)
)]
pub struct Judge {
    pub project_path: PathBuf,
    pub language: Language,
}

#[impl_state]
impl Judge {
    #[require(Created)]
    pub fn new(code: &[u8], checker: &[u8], language: Language) -> io::Result<Judge> {
        let project_path = env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir(&project_path)?;
        let main_path = project_path
            .with_file_name(SUBMISSION)
            .with_extension(language.extension);
        let checker_path = project_path.with_file_name(CHECKER);

        fs::write(&main_path, code)?;
        fs::write(&checker_path, checker)?;

        Ok(Judge {
            project_path,
            language,
        })
    }

    #[require(Created)]
    #[switch_to(Compiled)]
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

    #[require(Compiled)]
    pub fn run(
        self,
        input: &[u8],
        is_interactive: bool,
        resource: Resource,
        time_limit: Duration,
    ) -> io::Result<Verdict> {
        let checker_to_submission = self.project_path.join(INPUT);
        unistd::mkfifo(&checker_to_submission, stat::Mode::S_IRWXU)?;
        if !is_interactive {
            fs::write(&checker_to_submission, input)?;
        }

        let submission_to_checker = self.project_path.join(OUTPUT);
        unistd::mkfifo(&submission_to_checker, stat::Mode::S_IRWXU)?;
        fs::write(&submission_to_checker, input)?;

        let sandbox = Sandbox::new(resource, time_limit)?;

        let mut checker = self
            .language
            .get_run_command(CHECKER)
            .current_dir(&self.project_path)
            .stdin(
                fs::OpenOptions::new()
                    .read(true)
                    .open(&submission_to_checker)?,
            )
            .stdout(
                fs::OpenOptions::new()
                    .write(true)
                    .open(&checker_to_submission)?,
            )
            .spawn()?;
        let submission = self
            .language
            .get_run_command(SUBMISSION)
            .current_dir(&self.project_path)
            .stdin(
                fs::OpenOptions::new()
                    .read(true)
                    .open(&checker_to_submission)?,
            )
            .stdout(
                fs::OpenOptions::new()
                    .write(true)
                    .open(&submission_to_checker)?,
            )
            .spawn()?;
        if let Some(verdict) = sandbox.monitor(submission)? {
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
