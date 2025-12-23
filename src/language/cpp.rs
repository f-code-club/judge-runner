use super::Language;

pub const CPP: Language =
    Language::new(Some(&["g++", "-o", "{main}", "{main}.cpp"]), &["./{main}"]);
