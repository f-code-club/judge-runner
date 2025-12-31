mod util;

use std::path::Path;
use std::time::Duration;

use byte_unit::Byte;
use judge_runner::{Code, Judge, Language, Resource, Verdict, language::*};
use rstest::rstest;

#[rstest]
#[tokio::test(flavor = "multi_thread")]
pub async fn should_return_accepted(
    #[rustfmt::skip]
    #[values(RUST, CPP, PYTHON, JAVA)]
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

    let judge = Judge::builder()
        .checker(Code {
            content: &checker,
            language: CPP,
        })
        .main(Code {
            content: &solution,
            language,
        })
        .resource(Resource {
            memory: Byte::GIGABYTE,
            ..Default::default()
        })
        .time_limit(Duration::from_secs(1))
        .build()
        .await
        .unwrap();
    let judge = judge.compile().await.unwrap().unwrap();

    for input in inputs {
        let metrics = judge.run(input.as_bytes()).await.unwrap();
        assert_eq!(metrics.verdict, Verdict::Accepted);
    }
}
