# Structure ladder fixtures

Toy polynomials run through `d1-measure` on the same region and bounds as the instances (a in [−3000, 3000], b in [1, 1500]; 2^14, 2^16, 2^18, 2^20), with `--no-hw-sample`. `n: 1000003` in each file is a label for the parser, not a number being factored; the rational side a − 2b is trivially smooth and is ignored. Per polynomial: `<key>.poly` is the input and `<key>.scaled-down.json` the result. `../patterns.scaled-down.csv` and `../patterns.scaled-down.u_bins.csv` collect them beside d1-01.

| key | polynomial | structure |
|---|---|---|
| x2m1 … x12m1 | x^n − 1 for n = 2, 3, 4, 5, 6, 7, 8, 10, 12 | reducible, one cyclotomic factor per divisor of n |
| x4m1-big | x^4 − 1 on a in [−10000, 10000], b in [1, 5000] (60.8 million values), bounds 2^14 to 2^24 | reducible, larger region |
| x4m1.b28, x6m1.b28 | x^4 − 1 and x^6 − 1 at 2^20, 2^24, 2^28 on the standard region; every value is smooth at 2^24 and above because every factor is smaller than B | reducible, protocol bounds |
| a4 | x^4 + 8x + 12 | irreducible, Galois group A4 |
| s4 | x^4 + x + 1 | irreducible, S4 |
| c4 | x^4 + x^3 + x^2 + x + 1 | irreducible, cyclic C4 |
| d4 | x^4 − 2 | irreducible, D4 |
| v4 | x^4 + 1 | irreducible, V4 |
