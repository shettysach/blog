use anyhow::{Result, anyhow};
use pulldown_cmark::{Options, Parser, html};
use std::{fmt::Write, fs, path::Path};
use syntect::parsing::SyntaxSet;
use walkdir::WalkDir;

use crate::syntex::{Article, Metadata, process};
use crate::utils;

const HEADER: &str = include_str!("../layout/header.html");
const FOOTER: &str = include_str!("../layout/footer.html");

const OPTIONS: Options = Options::empty()
    .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
    .union(Options::ENABLE_MATH)
    .union(Options::ENABLE_HEADING_ATTRIBUTES);

fn markdown_to_html<'a>(markdown: &'a str, syntax_set: &'a SyntaxSet) -> Result<Article<'a>> {
    process(Parser::new_ext(markdown, OPTIONS), syntax_set)
}

fn render_markdown_file(src: &Path, dst: &Path, syntax_set: &SyntaxSet) -> Result<Metadata> {
    let markdown = fs::read_to_string(src)?;
    let Article { events, metadata } =
        markdown_to_html(&markdown, syntax_set).map_err(|e| anyhow!("{}: {}", e, src.display()))?;

    let mut page = String::with_capacity(markdown.len() * 3 / 2);
    page.push_str(HEADER);
    html::push_html(&mut page, events.into_iter());
    page.push_str(FOOTER);

    fs::write(dst, page)?;
    Ok(metadata)
}

pub(crate) fn static_pages(markdown_dir: &Path, styles_dir: &Path, html_dir: &Path) -> Result<()> {
    let syntax_set = SyntaxSet::load_defaults_newlines(); // TODO: Lazy/Once init
    utils::copy_directory(styles_dir, html_dir)?;

    let num_articles = WalkDir::new(markdown_dir).max_depth(1).into_iter().count() - 2;
    let mut articles = Vec::with_capacity(num_articles);

    for entry in WalkDir::new(markdown_dir)
        .max_depth(2)
        .sort_by_file_name()
        .into_iter()
        .flatten()
    {
        let path = entry.path();
        let rel_path = path.strip_prefix(markdown_dir)?;

        if path.is_dir() {
            fs::create_dir_all(html_dir.join(rel_path))?;
        } else if path.extension().is_some_and(|ext| ext == "md")
            && path.file_name().is_some_and(|i| i != "index.md")
        {
            let dst_path = html_dir.join(rel_path).with_extension("html");
            let metadata = render_markdown_file(path, &dst_path, &syntax_set)?;
            articles.push((rel_path.with_extension("html"), metadata));
        } else {
            let dst_path = html_dir.join(rel_path);
            fs::copy(path, dst_path)?;
        }
    }

    let index_md = fs::read_to_string(markdown_dir.join("index.md"))?;
    let index_events = markdown_to_html(&index_md, &syntax_set)?.events.into_iter();

    let mut index_html = String::new();

    index_html.push_str(HEADER);
    html::push_html(&mut index_html, index_events);
    index_html.push_str("\n<h2>Articles</h2>\n<ul>\n");
    for (link, metadata) in articles.into_iter() {
        let label = match metadata.subtitle {
            Some(sub) => format!("{}<br><div class=\"subt\">{sub}</div>", metadata.title),
            None => metadata.title,
        };
        writeln!(
            index_html,
            "<li><a href=\"{}\">{}</a></li>",
            link.to_string_lossy(),
            label
        )?;
    }
    index_html.push_str("</ul>");
    index_html.push_str(FOOTER);

    fs::write(html_dir.join("index.html"), index_html)?;

    Ok(())
}
