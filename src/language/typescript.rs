use crate::Language;

pub const TYPESCRIPT: Language = Language {
    compile_args: None,
    run_args: "bun run {main}.ts",
    extension: "ts",
};
