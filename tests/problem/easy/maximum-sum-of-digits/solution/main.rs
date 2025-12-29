use std::io;

fn sum_digits(mut x: u64) -> u64 {
    let mut sum = 0;
    while x > 0 {
        sum += x % 10;
        x /= 10;
    }
    sum
}

fn find_max_digit_sum(n: u64) -> u64 {
    if n < 10 {
        return n; // because a + b = n, a can be 0 and b n, sum is 0 + S(n) = S(n)
    }
    // Generate the candidate 999...9 where the number has m digits
    let mut pow10 = 1;
    while pow10 <= n / 10 {
        pow10 *= 10;
    }
    let candidate = pow10 - 1;
    let a = candidate;
    let b = n - a;
    sum_digits(a) + sum_digits(b)
}

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let n: u64 = input.trim().parse().expect("Please enter a valid number");
    println!("{}", find_max_digit_sum(n));
}
