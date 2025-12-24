use crate::Language;

pub const PYTHON: Language = Language {
    compile_args: None,
    run_args: "python {main}.py",
    extension: "py",
};
