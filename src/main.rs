mod generate;
mod syntex;
mod utils;
use std::io;

fn main() -> io::Result<()> {
    let markdown_dir = "markdown";
    let styles_dir = "styles";
    let output_dir = "_site";

    generate::static_pages(markdown_dir, styles_dir, output_dir)?;

    Ok(())
}
