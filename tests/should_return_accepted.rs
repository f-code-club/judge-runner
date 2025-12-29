mod util;

use std::path::Path;
use std::time::Duration;

use byte_unit::Byte;
use judge_runner::{Judge, Language, Resource, Verdict, language::*};
use rstest::rstest;

#[rstest]
#[tokio::test(flavor = "multi_thread")]
pub async fn should_return_accepted(
    #[rustfmt::skip]
    #[values(RUST, CPP, TYPESCRIPT, JAVASCRIPT, PYTHON, JAVA)]
    language: Language,

    #[dirs]
    #[files("tests/problem/easy/*")]
    #[exclude("wrong-answer")]
    #[by_ref]
    problem: &Path,
) {
    let inputs = util::read_inputs(problem);
    let checker = util::read_checker(problem, CPP).await;
    let solution = util::read_solution(problem, language);
    let resource = Resource {
        memory: Byte::GIGABYTE,
        ..Default::default()
    };
    let time_limit = Duration::from_secs(5);

    let judge = Judge::builder()
        .checker(&checker, CPP)
        .main(&solution, language)
        .build()
        .await
        .unwrap();
    let judge = judge.compile().await.unwrap().unwrap();

    for input in inputs {
        let metrics = judge
            .run(input.as_bytes(), false, resource, time_limit)
            .await
            .unwrap();
        println!("{:#?}", metrics);
        assert_eq!(metrics.verdict, Verdict::Accepted);
    }
}
