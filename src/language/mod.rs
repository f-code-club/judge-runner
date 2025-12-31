mod cpp;
mod java;
mod python;
mod rust;

use tokio::process::Command;

pub use cpp::CPP;
pub use java::JAVA;
pub use python::PYTHON;
pub use rust::RUST;

#[derive(Debug, Clone, Copy, Default)]
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
        let binary = args.next().unwrap();

        let mut command = Command::new(binary);
        command.args(args);

        Some(command)
    }
    pub fn get_run_command(&self, main: &str) -> Command {
        let args = self.run_args.replace("{main}", main);
        let mut args = args.split_whitespace();
        let binary = args.next().unwrap();

        let mut command = Command::new(binary);
        command.args(args);

        command
    }
    pub fn is_interpreted(&self) -> bool {
        self.compile_args.is_none()
    }
}
