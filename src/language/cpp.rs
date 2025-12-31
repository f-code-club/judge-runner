use crate::Language;

pub const CPP: Language = Language {
    compile_args: Some("g++ -o {main} {main}.cpp"),
    run_args: "./{main}",
    extension: "cpp",
};
