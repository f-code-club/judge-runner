use std::io::stdin;

fn main() {
    let s = stdin().lines().next().unwrap().unwrap();
    println!("{s}");
}
