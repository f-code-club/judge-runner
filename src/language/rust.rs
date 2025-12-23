use super::Language;

pub const RUST: Language = Language::new(Some(&["rustc", "-O", "{main}.rs"]), &["./{main}"], "rs");
