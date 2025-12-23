use super::Language;

pub const RUST: Language = Language {
    compile_args: Some(&["rustc", "-O", "{main}.rs"]),
    run_args: &["./{main}"],
};
