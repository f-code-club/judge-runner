use std::{env, fs, io, path::PathBuf};

use state_shift::{impl_state, type_state};
use uuid::Uuid;

use crate::{Language, Verdict};

const MAIN: &str = "main";
const CHECKER: &str = "checker";

#[type_state(
    states = (Created, Compiled),
    slots = (Created)
)]
pub struct Runner {
    pub project_path: PathBuf,
    pub language: Language,
}

#[impl_state]
impl Runner {
    #[require(Created)]
    pub fn new(code: &[u8], checker: &[u8], language: Language) -> io::Result<Runner> {
        let project_path = env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir(&project_path)?;
        let main_path = project_path
            .with_file_name(MAIN)
            .with_extension(language.extension);
        let checker_path = project_path.with_file_name(CHECKER);

        fs::write(&main_path, code)?;
        fs::write(&checker_path, checker)?;

        Ok(Runner {
            project_path,
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
}
