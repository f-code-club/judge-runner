use super::Language;

pub const TYPESCRIPT: Language = Language::new(None, &["bun", "run", "{main}.ts"], "ts");
