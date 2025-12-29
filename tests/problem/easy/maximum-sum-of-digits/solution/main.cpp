#include <iostream>
using namespace std;

long long sum_digits(long long x) {
    long long sum = 0;
    while (x > 0) {
        sum += x % 10;
        x /= 10;
    }
    return sum;
}

long long find_max_digit_sum(long long n) {
    if (n < 10) {
        return n;
    }

    long long temp = n;
    long long pow10 = 1;
    while (pow10 <= n / 10) {
        pow10 *= 10;
    }
    long long candidate = pow10 - 1;
    long long a = candidate;
    long long b = n - a;
    return sum_digits(a) + sum_digits(b);
}

int main() {
    long long n;
    cin >> n;
    cout << find_max_digit_sum(n) << endl;
    return 0;
}
