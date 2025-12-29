import java.util.Scanner;

public class main {
    public static long sumDigits(long x) {
        long sum = 0;
        while (x > 0) {
            sum += x % 10;
            x /= 10;
        }
        return sum;
    }

    public static long findMaxDigitSum(long n) {
        if (n < 10) {
            return n; 
        }
        long temp = n;
        long pow10 = 1;
        while (pow10 <= n / 10) {
            pow10 *= 10;
        }
        long candidate = pow10 - 1;
        long a = candidate;
        long b = n - a;
        return sumDigits(a) + sumDigits(b);
    }

    public static void main(String[] args) {
        Scanner scanner = new Scanner(System.in);
        long n = scanner.nextLong();
        System.out.println(findMaxDigitSum(n));
        scanner.close();
    }
}
