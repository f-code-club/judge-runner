mod cpp;
mod java;
mod javascript;
mod python;
mod rust;
mod typescript;

use std::collections::HashMap;

use anyhow::Result;
use strfmt::Format;

pub use cpp::CPP;
pub use java::JAVA;
pub use javascript::JAVASCRIPT;
pub use python::PYTHON;
pub use rust::RUST;
pub use typescript::TYPESCRIPT;

pub struct Language<'a> {
    compile_args: Option<&'a [&'a str]>,
    run_args: &'a [&'a str],
    pub extension: &'a str,
}

impl Language<'_> {
    #[inline]
    pub const fn new<'a>(
        compile_args: Option<&'a [&'a str]>,
        run_args: &'a [&'a str],
        extension: &'a str,
    ) -> Language<'a> {
        Language {
            compile_args,
            run_args,
            extension,
        }
    }

    pub fn compile_args(&self, main: &str) -> Option<Result<Vec<String>>> {
        self.compile_args.map(|raw| format(raw, main))
    }

    pub fn run_args(&self, main: &str) -> Result<Vec<String>> {
        format(self.run_args, main)
    }
}

fn format(args: &[&str], main: &str) -> Result<Vec<String>> {
    const MAIN: &str = "main";
    let vars = HashMap::from_iter([(MAIN.to_string(), main)]);

    args.iter()
        .map(|&x| x.format(&vars).map_err(anyhow::Error::from))
        .collect()
}
