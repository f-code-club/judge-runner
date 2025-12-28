# Judge Runner

A code runner library for online judge system.

## Supported Languages

- C++
- Java
- JavaScript
- Python
- Rust
- TypeScript

## Usage

```rust
use std::time::Duration;

use byte_unit::Byte;
use judge_runner::{Judge, Resource, language};

#[tokio::main]
async fn main() {
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
        .await
        .unwrap();
    let checker = checker.compile().await.unwrap().unwrap();
    let checker = checker.read_executable().await.unwrap();

    let judge = Judge::builder()
        .checker(&checker, language::PYTHON)
        .main(code.as_bytes(), language::CPP)
        .build()
        .await
        .unwrap();
    let judge = judge.compile().await.unwrap().unwrap();

    let input = "4";
    let resource = Resource {
        memory: Byte::MEBIBYTE.multiply(1030).unwrap(),
        ..Default::default()
    };
    let time_limit = Duration::from_secs(1);
    let verdict = judge
        .run(input.as_bytes(), false, resource, time_limit)
        .await
        .unwrap();
    println!("{:#?}", verdict);
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
