use std::{env, fs, io, path::PathBuf, time::Duration};

use state_shift::{impl_state, type_state};
use uuid::Uuid;

use crate::{Language, Resource, Sandbox, Verdict};

const MAIN: &str = "main";
const CHECKER: &str = "checker";

#[type_state(
    states = (Created, Compiled),
    slots = (Created)
)]
pub struct Judge {
    pub project_path: PathBuf,
    pub sandbox: Sandbox,
    pub language: Language,
}

#[impl_state]
impl Judge {
    #[require(Created)]
    pub fn new(
        code: &[u8],
        checker: &[u8],
        resource: Resource,
        time_limit: Duration,
        language: Language,
    ) -> io::Result<Judge> {
        let project_path = env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir(&project_path)?;
        let main_path = project_path
            .with_file_name(MAIN)
            .with_extension(language.extension);
        let checker_path = project_path.with_file_name(CHECKER);

        fs::write(&main_path, code)?;
        fs::write(&checker_path, checker)?;

        let sandbox = Sandbox::new(resource, time_limit)?;

        Ok(Judge {
            project_path,
            sandbox,
            language,
        })
    }

    #[require(Created)]
    #[switch_to(Compiled)]
    pub fn compile(&self) -> io::Result<Option<Verdict>> {
        let Some(mut command) = self.language.get_compile_command(MAIN) else {
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
    pub fn run(&self) -> io::Result<Verdict> {
        todo!()
    }
}
