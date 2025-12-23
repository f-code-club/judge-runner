use crate::{Language, language};

pub const CPP: Language = language!(
    ["g++", "-o", "{main}", "{main}.cpp"],
    ["./{main}"],
    "cpp"
);
