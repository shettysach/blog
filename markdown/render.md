# Internals of the blog

## Rendering equations and highlighted code

This section explains how the blog renders math and highlights syntax, using only static HTML and CSS.
The code for this is located in the [`syntex`](https://github.com/shettysach/blog/blob/main/src/syntex.rs)
module, a portmanteau of "syntax" and "TeX."

---

## Math - LaTeX to MathML

LaTeX is a high-quality typesetting system; it includes features designed for the production of technical and scientific documentation. LaTeX is the de facto standard for the communication and publication of scientific documents. LaTeX is available as free software. [[The Latex Project]](https://www.latex-project.org/)

### LaTeX representation of 2 x 2 identity matrix

```
\left[
\begin{matrix}
1 & 0\\
0 & 1
\end{matrix}
\right]
```

However, web browsers and standard HTML cannot directly interpret LaTeX syntax. Therefore, this blog converts LaTeX into MathML Core.

Mathematical Markup Language (MathML) is an XML-based language for describing mathematical notation.
MathML Core is a subset with increased implementation details based on rules from LaTeX and the Open Font Format. It is tailored for browsers and designed specifically to work well with other web standards including HTML, CSS, DOM, JavaScript.
[[MDN Web Docs]](https://developer.mozilla.org/en-US/docs/Web/MathML)

### MathML representation of 2 x 2 identity matrix

```xml
<math display="block" xmlns="http://www.w3.org/1998/Math/MathML">
  <mrow>
    <mo stretchy="true">[</mo>
    <mtable class="menv-arraylike">
      <mtr>
        <mtd>
          <mn>1</mn>
        </mtd>
        <mtd>
          <mn>0</mn>
        </mtd>
      </mtr>
      <mtr>
        <mtd>
          <mn>0</mn>
        </mtd>
        <mtd>
          <mn>1</mn>
        </mtd>
      </mtr>
    </mtable>
    <mo stretchy="true">]</mo>
  </mrow>
</math>
```

To convert from LaTeX to MathML, first `pulldown-cmark` events are used to identify math and code sections. When `pulldown-cmark` parses the markdown file, it detects the tags for both inline and block math expressions, as well as code blocks with language identifiers, including math.
Then, the LaTeX inside these sections, is converted to MathML using the [`pulldown_latex`](https://crates.io/crates/pulldown-latex) crate. The resultant MathML is added to the output events vector.

### Final output for 2 x 2 identity matrix

```math
\left[
\begin{matrix}
1 & 0\\
0 & 1
\end{matrix}
\right]
```

### Markdown syntax for math sections

There are two types of math displays,

- **Inline display** - $F(n)$, written as

```
$F(n)$
```

- **Block display**
  $$
  F_{\text{head}}(n) =
  \begin{cases}
  0 & \text{if } n = 0, \\
  1 & \text{if } n = 1, \\
  F_{\text{head}}(n-1) +
  F_{\text{head}}(n-2) & \text{if } n \geq 2.
  \end{cases}
  $$
  written as

```console
$$
F_{\text{head}}(n) =
\begin{cases}
0 & \text{if } n = 0, \\
1 & \text{if } n = 1, \\
F_{\text{head}}(n-1) +
F_{\text{head}}(n-2) & \text{if } n \geq 2.
\end{cases}
$$
```

or

````
```math
F_{\text{head}}(n) =
\begin{cases}
0 & \text{if } n = 0, \\
1 & \text{if } n = 1, \\
F_{\text{head}}(n-1) +
F_{\text{head}}(n-2) & \text{if } n \geq 2.
\end{cases}
```
````

---

## Code - Syntax highlighting

For syntax highlighting, the blog uses the [`syntect`](https://crates.io/crates/pulldown-cmark),
which is also used in various other programs such as [`bat`](https://github.com/sharkdp/bat), the alternative
to `cat`.

When `pulldown-cmark` identifies a code block with a language identifier,
`syntect` generates HTML with CSS classes for different code elements
like keywords, variables and constants. This enables for syntax highlighting
through CSS style classes that can be customized. See
[`code.css`](https://github.com/shettysach/blog/tree/main/styles/code.css).

### Markdown syntax for code sections

Similarly, there are two types of code displays,

- **Inline display** - `fibonacci(5)`, written as

```
`fibonacci(5)`
```

- **Block display**

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

written as

````
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
````

---

Of course, if you value your time, you could use JS libraries for 
implementing the above, such as KaTex or MathJax for LaTeX rendering
and highlight.js or PrismJS for syntax highlighting.
