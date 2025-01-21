# Fibonacci sequence

---

In mathematics, the _Fibonacci sequence_ is a sequence in which each number is the sum of the two numbers that precede it. Numbers that are part of the sequence are known as _Fibonacci numbers_. The $n^{th}$ Fibonacci number is represented by $F_n$.

## Head Recursive Fibonacci

$$
F_{\text{n}} =
\begin{cases}
0 & \text{if } n = 0, \\
1 & \text{if } n = 1, \\
F_{\text{n-1}} + F_{\text{n-2}} & \text{if } n \geq 2.
\end{cases}
$$

```rust
// rust
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

```python
# python
def fibonacci(n: int) -> int:
    if n == 0:
        return 0
    elif n == 1:
        return 1
    else:
        return fibonacci(n - 1) + fibonacci(n - 2)
```

## Tail Recursive Fibonacci

$$
F_{\text{n,a,b}} =
\begin{cases}
a & \text{if } n = 0, \\
F_{\text{n-1, b, a+b}} & \text{if } n > 0.
\end{cases}
$$

```ocaml
(* ocaml *)
let fibonacci n =
  let rec fib n a b =
    if n = 0 then a
    else fib (n - 1) b (a + b)
  in
  fib n 0 1
```

```haskell
-- haskell
fibonacci :: Integer -> Integer
fibonacci n = fib n 0 1
  where
    fib 0 a _ = a
    fib n a b = fib (n - 1) b (a + b)
```
