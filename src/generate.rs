use crate::{syntex, utils::copy_dir};
use pulldown_cmark::{html, Options, Parser};
use std::{
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};
use syntect::parsing::SyntaxSet;

const INDEX_HEADER: &str = include_str!("../layout/index_header.html");
const HEADER: &str = include_str!("../layout/header.html");
const FOOTER: &str = include_str!("../layout/footer.html");

fn convert_to_html(markdown: &str, syntax_set: &SyntaxSet) -> io::Result<String> {
    let events = Parser::new_ext(markdown, Options::ENABLE_MATH);
    let events = syntex::process(events, syntax_set)?.into_iter();

    let mut html_content = String::with_capacity(markdown.len() * 3 / 2);
    html::push_html(&mut html_content, events);

    Ok(html_content)
}

struct Article {
    name: String,
    dir: String,
    path: PathBuf,
}

// TODO: Better path handling
fn process_articles<P>(input_dir: P, output_dir: P, syntax_set: &SyntaxSet) -> io::Result<String>
where
    P: AsRef<Path> + Display,
{
    let article_list = fs::read_dir(input_dir)?
        .flatten()
        .filter_map(|article| {
            let dir_path = article.path();
            let index_path = dir_path.join("index.md");
            let metadata_path = dir_path.join("metadata.txt");

            (index_path.exists() && metadata_path.exists()).then_some(Article {
                name: std::fs::read_to_string(metadata_path).ok()?,
                dir: dir_path.file_stem().unwrap().to_str().unwrap().to_string(),
                path: index_path,
            })
        })
        .map(|article| {
            let article_contents = fs::read_to_string(&article.path)?;

            let html_contents = convert_to_html(&article_contents, syntax_set)?;
            let html_page = format!("{}\n{}\n{}", HEADER, html_contents, FOOTER);

            let output_path = format!("{}/{}", output_dir, article.dir);
            fs::create_dir_all(&output_path)?;
            fs::write(format!("{output_path}/index.html"), &html_page)?;

            Ok(format!(
                "<li><a href=\"./{}/\">{}</a></li>",
                article.dir, article.name,
            ))
        })
        .collect::<io::Result<Vec<String>>>()?
        .join("\n");

    Ok(format!(
        "\n<h2>Articles</h2>\n<ul>\n{}\n</ul>\n",
        article_list
    ))
}

// TODO: Better path handling
pub(crate) fn static_pages<P>(input_dir: P, styles_dir: P, output_dir: P) -> io::Result<()>
where
    P: AsRef<Path> + Display + Copy,
{
    copy_dir(styles_dir, output_dir)?;

    let syntax_set = SyntaxSet::load_defaults_newlines();

    let index_path = format!("./{input_dir}/index.md");
    let file_string = fs::read_to_string(&index_path)?;
    let mut index_html = convert_to_html(&file_string, &syntax_set)?;

    let articles_list = process_articles(input_dir, output_dir, &syntax_set)?;
    index_html.push_str(&articles_list);
    let index_page = format!("{}\n{}\n{}", INDEX_HEADER, index_html, FOOTER);

    let index_output = format!("./{output_dir}/index.html");
    fs::write(index_output, &index_page)?;

    Ok(())
}
