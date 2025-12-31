use crate::Language;

pub const JAVA: Language = Language {
    compile_args: Some("javac {main}.java"),
    run_args: "java {main}",
    extension: "java",
};
