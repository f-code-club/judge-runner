mod cpp;
mod java;
mod javascript;
mod python;
mod rust;
mod typescript;

use std::process::Command;

pub use cpp::CPP;
pub use java::JAVA;
pub use javascript::JAVASCRIPT;
pub use python::PYTHON;
pub use rust::RUST;
pub use typescript::TYPESCRIPT;

#[derive(Clone, Copy)]
pub struct Language {
    pub compile_args: Option<&'static str>,
    pub run_args: &'static str,
    pub extension: &'static str,
}

impl Language {
    pub fn get_compile_command(&self, main: &str) -> Option<Command> {
        let args = self.compile_args?;
        let args = args.replace("{main}", main);
        let mut args = args.split_whitespace();
        // SAFETY: there is always at least 1 element
        let binary = args.next().unwrap();

        let mut command = Command::new(binary);
        command.args(args);

        Some(command)
    }
    pub fn get_run_command(&self, main: &str) -> Command {
        let args = self.run_args.replace("{main}", main);
        let mut args = args.split_whitespace();
        // SAFETY: there is always at least 1 element
        let binary = args.next().unwrap();

        let mut command = Command::new(binary);
        command.args(args);

        command
    }
    pub fn is_interpreted(&self) -> bool {
        self.compile_args.is_none()
    }
}
