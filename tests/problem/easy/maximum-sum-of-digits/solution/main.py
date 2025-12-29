def sum_digits(x):
    return sum(int(d) for d in str(x))

def find_max_digit_sum(n):
    if n < 10:
        return n  
    pow10 = 1
    while pow10 <= n // 10:
        pow10 *= 10
    candidate = pow10 - 1
    a = candidate
    b = n - a
    return sum_digits(a) + sum_digits(b)

n = int(input())
print(find_max_digit_sum(n))
