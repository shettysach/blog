use crate::syntex::SynTex;
use pulldown_cmark::{html, Options, Parser};
use std::{fmt::Display, fs, io, path::Path};

const HEADER: &str = include_str!("../layout/header.html");
const FOOTER: &str = include_str!("../layout/footer.html");

fn convert_to_html(markdown: &str, syn_tex: &SynTex) -> io::Result<String> {
    let events = Parser::new_ext(markdown, Options::ENABLE_MATH);
    let events = syn_tex.process(events)?.into_iter();

    let mut html_content = String::with_capacity(markdown.len() * 3 / 2);
    html::push_html(&mut html_content, events);

    Ok(html_content)
}

fn process_articles<P>(input_dir: P, output_dir: P, syn_tex: &SynTex) -> io::Result<String>
where
    P: AsRef<Path> + Display,
{
    let article_list = fs::read_dir(input_dir)?
        .flatten()
        .filter_map(|article| {
            let article_path = article.path();
            let is_markdown = article_path.extension() == Some("md".as_ref());
            let is_not_index = article_path.file_stem() != Some("index".as_ref());

            (is_markdown && is_not_index).then_some(article_path)
        })
        .map(|article_path| {
            let article_name = article_path.file_stem().unwrap().to_string_lossy();
            let article_contents = fs::read_to_string(&article_path)?;

            let html_contents = convert_to_html(&article_contents, syn_tex)?;
            let html_page = format!("{}\n{}\n{}", HEADER, html_contents, FOOTER);
            let output_path = format!("{}/{}.html", output_dir, article_name);
            fs::write(&output_path, &html_page)?;

            Ok(format!(
                "<li><a href=\"./{}.html\">{}</a></li>",
                article_name, article_name
            ))
        })
        .collect::<io::Result<Vec<String>>>()?
        .join("\n");

    Ok(format!(
        "\n<h2>Articles</h2>\n<ul>\n{}\n</ul>\n",
        article_list
    ))
}

pub(crate) fn static_pages<P>(input_dir: P, styles_dir: P, output_dir: P) -> io::Result<()>
where
    P: AsRef<Path> + Display + Copy,
{
    fs::create_dir_all(output_dir)?;

    for entry in fs::read_dir(styles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let destination = output_dir.as_ref().join(file_name);
            fs::copy(&path, destination)?;
        }
    }

    let syntax_highlighter = SynTex::new();

    let index_path = format!("./{input_dir}/index.md");
    let file_string = fs::read_to_string(&index_path)?;
    let mut index_html = convert_to_html(&file_string, &syntax_highlighter)?;

    let articles_list = process_articles(input_dir, output_dir, &syntax_highlighter)?;
    index_html.push_str(&articles_list);

    let index_output = format!("./{output_dir}/index.html");
    let html_page = format!("{}\n{}\n{}", HEADER, index_html, FOOTER);
    fs::write(index_output, &html_page)?;

    Ok(())
}
