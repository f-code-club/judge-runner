// Bun-compatible version

function sumDigitsStr(s) {
    return [...s].reduce((sum, ch) => sum + Number(ch), 0);
}

function findMaxDigitSum(n) {
    if (n < 10n) return Number(n);

    let pow10 = 1n;
    while (pow10 <= n / 10n) {
        pow10 *= 10n;
    }

    const a = pow10 - 1n;
    const b = n - a;

    return sumDigitsStr(a.toString()) + sumDigitsStr(b.toString());
}

for await (const n of console) {
    console.log(findMaxDigitSum(BigInt(n)));
    break;
}
