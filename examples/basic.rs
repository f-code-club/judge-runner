use std::time::Duration;

use byte_unit::Byte;
use judge_runner::{Judge, Resource, language};

fn main() {
    let checker_code = r#"
import sys
n = int(input())
res = int(input())

if n == res:
    sys.exit(0)
else:
    sys.exit(1)
    "#;
    let code = r#"
        #include<bits/stdc++.h>
        
        using namespace std;

        int main() {
            int n;
            cin >> n;
            cout << n << endl;
        }
    "#;

    let checker = Judge::builder()
        .main(checker_code.as_bytes(), language::PYTHON)
        .build()
        .unwrap()
        .compile()
        .unwrap()
        .unwrap()
        .read_executable()
        .unwrap();

    let judge = Judge::builder()
        .checker(&checker, language::PYTHON)
        .main(code.as_bytes(), language::CPP)
        .build()
        .unwrap();
    let judge = judge.compile().unwrap().unwrap();

    let input = rand::random::<i32>().to_string();
    let resource = Resource {
        memory: Byte::MEGABYTE,
        ..Default::default()
    };
    let time_limit = Duration::from_secs(1);
    let verdict = judge
        .run(input.as_bytes(), false, resource, time_limit)
        .unwrap();
    println!("{:?}", verdict);
}
