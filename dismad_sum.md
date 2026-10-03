# Dismad sum

The dismad sum of a positive rational $x$ is the sum of the partial quotients of its simple continued fraction. The two legal listings have the same sum, so the value depends only on $x$.

$$
\delta(x) = \sum_{i=0}^{n} a_i
$$

where

$$
x = [a_0; a_1, \ldots, a_n] = [a_0; a_1, \ldots, a_n - 1, 1]
$$

and $a_n > 1$. Equivalently, if $D(x) = \mathrm{diag}(a_0, \ldots, a_n)$, then $\delta(x) = \mathrm{tr}(D(x))$. The rewritten list gives a different diagonal and the same trace, because $a_n = (a_n - 1) + 1$.

## Invariance

$$
\delta([a_0; a_1, \ldots, a_n]) = a_0 + \cdots + a_{n-1} + a_n
$$

$$
= a_0 + \cdots + a_{n-1} + (a_n - 1) + 1
$$

$$
= \delta([a_0; a_1, \ldots, a_n - 1, 1])
$$

The listings differ. The matrices differ. The rational and the sum do not.

## Example

$$
\frac{22}{7} = [3; 7] = [3; 6, 1]
$$

| listing | diagonal | trace |
|---|---|---:|
| $[3; 7]$ | $\mathrm{diag}(3, 7)$ | $3 + 7 = 10$ |
| $[3; 6, 1]$ | $\mathrm{diag}(3, 6, 1)$ | $3 + 6 + 1 = 10$ |

So $\delta(22/7) = 10$.

The product of the diagonal entries is not invariant under the same rewrite: $3 \cdot 7 = 21$, while $3 \cdot 6 \cdot 1 = 18$. Order is not invariant either. $[0; 1, 19]$ and $[0; 19, 1]$ both have dismad sum $20$, and the values are $19/20$ and $1/20$.

## Normal form

The two listings are the same trace at consecutive sizes. The switch expands or reduces the diagonal by one.

$$
\mathrm{diag}(3, 7) \longleftrightarrow \mathrm{diag}(3, 6, 1)
$$

is $2 \times 2$ against $3 \times 3$, and both have trace $10$. Reduce while the corner is $1$: delete it and add $1$ to the previous last entry. Stop when the last entry is greater than $1$.

$$
\mathrm{diag}(3, 6, 1) \longrightarrow \mathrm{diag}(3, 7)
$$

The short diagonal is the normal form of $22/7$. Two expansions compare equal exactly when their reductions do. The product does not survive the same step: $21$ against $18$. The switch is legal only at the last slot, and only once. Splitting an earlier entry keeps a trace and changes the rational.

## What it is not

The dismad sum is not the convergent, and it is not the continuant trace.

For

$$
[0; 1, 1, 19, 1, 3, 136, 3, 5, 2, 6] = [0; 1, 1, 19, 1, 3, 136, 3, 5, 2, 5, 1]
$$

both listings have $\delta = 177$. The convergent is $2561727/5000000$. The trace of the continuant product

$$
\prod_i \begin{pmatrix} a_i & 1 \\ 1 & 0 \end{pmatrix}
$$

is $3336064$.

Length $1$ is the only case where $\delta(x) = x$. Concatenation of quotient lists adds the dismad sums. The convergent of a concatenation is not the sum of the convergents.
