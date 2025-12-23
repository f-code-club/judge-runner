mod cpp;
mod java;
mod javascript;
mod python;
mod rust;
mod typescript;

pub use cpp::CPP;
pub use java::JAVA;
pub use javascript::JAVASCRIPT;
pub use python::PYTHON;
pub use rust::RUST;
pub use typescript::TYPESCRIPT;

pub struct Language<'a> {
    pub checker_compile_args: Option<&'a [&'a str]>,
    pub checker_run_args: &'a [&'a str],

    pub submission_compile_args: Option<&'a [&'a str]>,
    pub submission_run_args: &'a [&'a str],

    pub extension: &'a str,
}

#[macro_export]
macro_rules! language {
    ([ $( $r:expr ),* $(,)? ], $e:expr) => {{
        let checker_run_args: &[&str] = &[
            $(
                const_format::str_replace!($r, "{main}", "checker")
            ),*
        ];
        let submission_run_args: &[&str] = &[
            $(
                const_format::str_replace!($r, "{main}", "main")
            ),*
        ];

        Language {
            checker_compile_args: None,
            checker_run_args,
            submission_compile_args: None,
            submission_run_args,
            extension: $e,
        }
    }};

    ([ $( $c:expr ),* $(,)? ], [ $( $r:expr ),* $(,)? ], $e:expr) => {{
        let checker_compile_args: &[&str] = &[
            $(
                const_format::str_replace!($c, "{main}", "checker")
            ),*
        ];
        let checker_run_args: &[&str] = &[
            $(
                const_format::str_replace!($r, "{main}", "checker")
            ),*
        ];
        let submission_compile_args: &[&str] = &[
            $(
                const_format::str_replace!($c, "{main}", "main")
            ),*
        ];
        let submission_run_args: &[&str] = &[
            $(
                const_format::str_replace!($r, "{main}", "main")
            ),*
        ];

        Language {
            checker_compile_args: Some(checker_compile_args),
            checker_run_args,
            submission_compile_args: Some(submission_compile_args),
            submission_run_args,
            extension: $e,
        }
    }};
}
