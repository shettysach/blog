mod generate;
mod syntex;
mod utils;
use std::path::Path;

use anyhow::Result;

fn main() -> Result<()> {
    let markdown_dir = Path::new("markdown");
    let styles_dir = Path::new("styles");
    let html_dir = Path::new("_site");

    generate::index_page(markdown_dir, html_dir)?;
    utils::copy_directory(styles_dir, html_dir)?;

    Ok(())
}
