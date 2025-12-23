use crate::{Language, language};

pub const RUST: Language = language!(["rustc", "-O", "{main}.rs"], ["./{main}"], "rs");
