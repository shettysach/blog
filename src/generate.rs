use anyhow::{Result, anyhow};
use itertools::Itertools;
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

    if let Some(out_dir) = dst.parent() {
        fs::create_dir_all(out_dir)?;
        if let Some(src_dir) = src.parent() {
            utils::copy_assets(src_dir, out_dir)?;
        }
    }

    fs::write(dst, page)?;
    Ok(metadata)
}

pub(crate) fn static_pages(markdown_dir: &Path, styles_dir: &Path, html_dir: &Path) -> Result<()> {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    utils::copy_directory(styles_dir, html_dir)?;

    let index_md = fs::read_to_string(markdown_dir.join("index.md"))?;
    let article = markdown_to_html(&index_md, &syntax_set)?;

    let articles_html: Result<String> = WalkDir::new(markdown_dir)
        .max_depth(2)
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "md")
                && entry.file_name() != "index.md"
        })
        .sorted_by(|a, b| a.file_name().cmp(b.file_name()))
        .try_fold(String::new(), |mut acc, entry| {
            let rel_path = entry.path().strip_prefix(markdown_dir).unwrap();
            let mut dest_path = html_dir.join(rel_path);
            dest_path.set_extension("html");

            let metadata = render_markdown_file(entry.path(), &dest_path, &syntax_set)?;

            let label = match metadata.subtitle {
                Some(sub) => format!("{}<br><div class=\"subt\">{sub}</div>", metadata.title),
                None => metadata.title,
            };

            let link = rel_path.with_extension("html");
            let link = link.to_string_lossy();
            writeln!(acc, "<li><a href=\"{link}\">{label}</a></li>")?;

            Ok(acc)
        });
    let articles_html = articles_html?;

    let mut index_html = String::with_capacity(index_md.len() * 3 / 2 + articles_html.len());
    html::push_html(&mut index_html, article.events.into_iter());
    write!(
        index_html,
        "\n<h2>Articles</h2>\n<ul>\n{articles_html}</ul>\n"
    )?;

    let final_page = format!("{HEADER}\n{index_html}\n{FOOTER}");
    fs::write(html_dir.join("index.html"), final_page)?;

    Ok(())
}
