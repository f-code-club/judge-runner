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
    states = (Builder, Created, Compiled),
    slots = (Builder)
)]
#[derive(Default)]
pub struct Judge {
    pub project_path: PathBuf,
    pub language: Language,
    pub checker_language: Option<Language>,
}

#[impl_state]
impl Judge {
    #[require(Builder)]
    pub fn new() -> io::Result<Judge> {
        let project_path = env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir(&project_path)?;

        Ok(Judge {
            project_path,
            language: Default::default(),
            checker_language: Default::default(),
        })
    }

    #[require(Builder)]
    pub fn with_checker(self, code: &[u8], language: Language) -> io::Result<Judge> {
        let mut path = self.project_path.join(CHECKER);
        if language.is_interpreted() {
            path.set_extension(language.extension);
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o755)
            .open(&path)?;
        file.write_all(code)?;

        Ok(Judge {
            project_path: self.project_path,
            language: self.language,
            checker_language: Some(language),
        })
    }

    #[require(Builder)]
    #[switch_to(Created)]
    pub fn with_main(self, code: &[u8], language: Language) -> io::Result<Judge> {
        let main_path = self
            .project_path
            .join(MAIN)
            .with_extension(language.extension);
        fs::write(&main_path, code)?;

        Ok(Judge {
            project_path: self.project_path,
            language: self.language,
            checker_language: self.checker_language,
        })
    }

    #[require(Created)]
    #[switch_to(Compiled)]
    pub fn compile(self) -> io::Result<Result<Judge, Verdict>> {
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
