use crate::{Language, language};

pub const TYPESCRIPT: Language = language!(["bun", "run", "{main}.ts"], "ts");
