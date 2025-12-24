use std::{env, fs, io};

use uuid::Uuid;

use crate::Language;

const MAIN: &str = "checker";

pub fn compile(code: &[u8], language: Language) -> io::Result<Vec<u8>> {
    let Some(mut command) = language.get_compile_command(MAIN) else {
        return Ok(code.to_vec());
    };
    let project_path = env::temp_dir().join(Uuid::new_v4().to_string());
    fs::create_dir(&project_path)?;
    let main = project_path
        .with_file_name(MAIN)
        .with_extension(language.extension);
    fs::write(&main, code)?;

    let mut process = command.current_dir(&project_path).spawn()?;
    let _ = process.wait()?;
    fs::read(main.with_extension(""))
}
