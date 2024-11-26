mod generate;
use std::io;

fn main() -> io::Result<()> {
    let markdown_dir = "markdown";
    let styles_path = "styles/styles.css";
    let output_dir = "_site";

    generate::static_pages(markdown_dir, styles_path, output_dir)?;

    Ok(())
}
