use super::Language;

pub const PYTHON: Language = Language {
    compile_args: None,
    run_args: &["python", "main.py"],
};
