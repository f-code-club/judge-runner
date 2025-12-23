use crate::{Language, language};

pub const JAVA: Language = language!(["javac", "{main}.java"], ["java", "{main}"], "java");
