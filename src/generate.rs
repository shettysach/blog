use comrak::{markdown_to_html_with_plugins, plugins, Options, Plugins};
use std::{fmt::Display, fs, io, path::Path};

const HEADER: &str = include_str!("../layout/header.html");
const FOOTER: &str = include_str!("../layout/footer.html");

fn md_to_html(markdown: &str) -> String {
    let mut opts = Options::default();
    opts.extension.underline = true;
    opts.extension.math_dollars = true;
    opts.extension.math_code = true;

    let adapter = plugins::syntect::SyntectAdapterBuilder::new().css().build();
    let mut plugs = Plugins::default();
    plugs.render.codefence_syntax_highlighter = Some(&adapter);

    markdown_to_html_with_plugins(markdown, &opts, &plugs)
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
            let file_string = fs::read_to_string(&article_path)?;
            let html = md_to_html(&file_string);
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

pub(crate) fn static_pages<P>(input_dir: P, styles_dir: P, output_dir: P) -> io::Result<()>
where
    P: AsRef<Path> + Display + Copy,
{
    fs::create_dir_all(output_dir)?;

    let index_path = format!("./{input_dir}/index.md");
    let file_string = fs::read_to_string(&index_path)?;
    let mut index_html = md_to_html(&file_string);

    let articles_list = process_articles(input_dir, output_dir)?;
    index_html.push_str(&articles_list);

    for entry in fs::read_dir(styles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let destination = output_dir.as_ref().join(file_name);
            fs::copy(&path, destination)?;
        }
    }

    let index_output = format!("./{output_dir}/index.html");
    let html_page = format!("{}\n{}\n{}", HEADER, index_html, FOOTER);
    fs::write(index_output, &html_page)?;

    Ok(())
}
