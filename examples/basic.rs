use std::time::Duration;

use byte_unit::Byte;
use judge_runner::{Code, Judge, Resource, language};

#[tokio::main]
async fn main() {
    let checker_code = r#"
#include<bits/stdc++.h>

using namespace std;

int main() {
    string s, res;
    cin >> s >> res;
    if (s == res) {
        return 0;
    } else {
        return 1;
    }
}
    "#;
    let code = r#"
#include<bits/stdc++.h>

using namespace std;

int main() {
    string s;
    cin >> s;
    cout << s << endl;
}
"#;

    let checker = Judge::builder()
        .main(Code {
            content: checker_code.as_bytes(),
            language: language::CPP,
        })
        .build()
        .await
        .unwrap();
    let checker = checker.compile().await.unwrap().unwrap();
    let checker = checker.read_executable().await.unwrap();

    let judge = Judge::builder()
        .checker(Code {
            content: &checker,
            language: language::CPP,
        })
        .main(Code {
            content: code.as_bytes(),
            language: language::CPP,
        })
        .time_limit(Duration::from_secs(1))
        .resource(Resource {
            memory: Byte::GIGABYTE,
            ..Default::default()
        })
        .build()
        .await
        .unwrap();
    let judge = judge.compile().await.unwrap().unwrap();

    let input = "4";
    let metrics = judge.run(input.as_bytes()).await.unwrap();
    println!("{:#?}", metrics);
}
