use super::Language;

pub const JAVA: Language = Language::new(Some(&["javac", "{main}.java"]), &["java", "{main}"]);
