use super::Language;

pub const JAVASCRIPT: Language = Language {
    compile_args: None,
    run_args: &["bun", "run", "main.js"],
};
