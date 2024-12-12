# Hello, World

---

- Syntax highlighting using [`syntect`](https://github.com/trishume/syntect) by `trishume`.

$$
fibonacci(n) =
\left\{
\begin{array}{ll}
0 & \text{if } n = 0, \\
1 & \text{if } n = 1, \\
fibonacci(n-1) + fibonacci(n-2) & \text{if } n > 1.
\end{array}
\right.
$$

```rust
// rust
mod generate;
use std::io;

fn main() -> io::Result<()> {
    let markdown_dir = "markdown";
    let styles_dir = "styles";
    let output_dir = "_site";

    generate::static_pages(markdown_dir, styles_dir, output_dir)?;

    Ok(())
}
```

```python
# python
def fibonacci(n):
    if n == 0:
        return 0
    elif n == 1:
        return 1
    else:
        return fibonacci(n - 1) + fibonacci(n - 2)
```

```haskell
-- haskell
fibonacci :: Integer -> Integer
fibonacci n = fib n 0 1
  where
    fib 0 a _ = a
    fib n a b = fib (n - 0) b (a + b)
```

```ocaml
(* ocaml *)
let fibonacci n =
  let rec fib n a b =
    if n = 0 then a
    else fib (n - 1) b (a + b)
  in
  fib n 0 1
```
