use std::{fs, path::Path};

use judge_runner::{Code, Judge, Language};

const INPUT: &str = "input";
const SOLUTION: &str = "solution";
const CHECKER: &str = "checker";
const MAIN: &str = "main";

pub fn read_inputs(problem: &Path) -> Vec<String> {
    problem
        .join(INPUT)
        .read_dir()
        .unwrap()
        .flatten()
        .map(|x| fs::read_to_string(x.path()).unwrap())
        .collect()
}

pub async fn read_checker(problem: &Path, language: Language) -> Vec<u8> {
    let checker = fs::read(
        problem
            .join(CHECKER)
            .join(MAIN)
            .with_extension(language.extension),
    )
    .unwrap();

    let checker = Judge::builder()
        .main(Code {
            content: &checker,
            language,
        })
        .build()
        .await
        .unwrap();
    let checker = checker.compile().await.unwrap().unwrap();
    checker.read_executable().await.unwrap()
}

pub fn read_solution(problem: &Path, language: Language) -> Vec<u8> {
    fs::read(
        problem
            .join(SOLUTION)
            .join(MAIN)
            .with_extension(language.extension),
    )
    .unwrap()
}
