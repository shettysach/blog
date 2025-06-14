mod generate;
mod syntex;
mod utils;
use std::path::Path;

use anyhow::Result;

fn main() -> Result<()> {
    let markdown_dir = Path::new("markdown");
    let styles_dir = Path::new("styles");
    let html_dir = Path::new("_site");

    generate::static_pages(markdown_dir, styles_dir, html_dir)
}
