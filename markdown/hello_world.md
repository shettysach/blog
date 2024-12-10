# Hello, World

---

Syntax highlighting using `syntect`

```rust
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

```c
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    return 0;
}
```

```haskell
main :: IO ()
main = putStrLn "Hello, World!"
```
