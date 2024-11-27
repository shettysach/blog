use pulldown_cmark::{html, Parser};
use std::{fmt::Display, fs, io, path::Path};

const HEADER: &str = include_str!("../layout/header.html");
const FOOTER: &str = include_str!("../layout/footer.html");

fn markdown_to_html<P: AsRef<Path>>(input_path: P) -> io::Result<String> {
    let markdown_content = fs::read_to_string(input_path)?;
    let mut html_content = String::new();

    let parser = Parser::new(&markdown_content);
    html::push_html(&mut html_content, parser);

    Ok(html_content)
}

fn process_articles<P>(input_dir: P, output_dir: P) -> io::Result<String>
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
            let html = markdown_to_html(&article_path)?;
            let html_page = format!("{}\n{}\n{}", HEADER, html, FOOTER);

            let article_name = article_path.file_stem().unwrap().to_string_lossy();
            let output_file = format!("{}/{}.html", output_dir, article_name);

            fs::write(&output_file, &html_page)?;

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

pub(crate) fn static_pages<P>(input_dir: P, styles_path: P, output_dir: P) -> io::Result<()>
where
    P: AsRef<Path> + Display + Copy,
{
    fs::create_dir_all(output_dir)?;

    let index_path = format!("./{input_dir}/index.md");
    let mut index_html = markdown_to_html(index_path)?;

    let articles_list = process_articles(input_dir, output_dir)?;
    index_html.push_str(&articles_list);

    let styles_output = format!("./{output_dir}/styles.css");
    fs::copy(styles_path, &styles_output)?;

    let index_output = format!("./{output_dir}/index.html");
    let html_page = format!("{}\n{}\n{}", HEADER, index_html, FOOTER);
    fs::write(index_output, &html_page)
}
