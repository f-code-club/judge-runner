mod cpp;
mod java;
mod javascript;
mod python;
mod rust;
mod typescript;

pub use cpp::CPP;
pub use java::JAVA;
pub use javascript::JAVASCRIPT;
pub use python::PYTHON;
pub use rust::RUST;
pub use typescript::TYPESCRIPT;

pub struct Language<'a> {
    pub compile_args: Option<&'a [&'a str]>,
    pub run_args: &'a [&'a str],
}
