use super::Language;

pub const JAVA: Language = Language {
    compile_args: Some(&["javac", "Main.java"]),
    run_args: &["java", "Main"],
};
