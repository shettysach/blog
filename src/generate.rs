use crate::{syntex, utils::*};
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

fn process_article<P: AsRef<Path>>(dir_path: &Path, output_base: P) -> io::Result<Article> {
    let index_path = dir_path.join("index.md");
    let metadata_path = dir_path.join("metadata.txt");

    if !index_path.exists() || !metadata_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Incomplete - `index.md` or `metadata` does not exist.",
        ));
    }

    let dir_name = dir_path.file_stem().and_then(|s| s.to_str()).unwrap();

    let output_dir = output_base.as_ref().join(dir_name);
    fs::create_dir_all(&output_dir)?;

    copy_article_contents(dir_path, &output_dir)?;

    Ok(Article {
        name: fs::read_to_string(&metadata_path)?,
        dir: dir_name.to_string(),
        path: index_path,
    })
}

fn list_articles<P>(input_dir: P, output_dir: P, syntax_set: &SyntaxSet) -> io::Result<String>
where
    P: AsRef<Path> + Display,
{
    let mut article_list = fs::read_dir(&input_dir)?
        .flatten()
        .filter_map(|entry| {
            entry
                .path()
                .is_dir()
                .then_some(process_article(&entry.path(), &output_dir))
        })
        .collect::<io::Result<Vec<Article>>>()?;

    article_list.sort_unstable_by(|a, b| a.dir.cmp(&b.dir));

    let article_list = article_list
        .into_iter()
        .map(|article| {
            let article_contents = fs::read_to_string(&article.path)?;
            let html_contents = convert_to_html(&article_contents, syntax_set)?;
            let html_page = format!("{}\n{}\n{}", HEADER, html_contents, FOOTER);

            let output_path = output_dir.as_ref().join(&article.dir);
            let index_path = output_path.join("index.html");

            fs::write(&index_path, &html_page)?;

            let url_path = Path::new(&article.dir).join("index.html");

            Ok(format!(
                "<li><a href=\"./{}\">{}</a></li>",
                url_path.to_str().unwrap(),
                article.name,
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
    copy_directory(styles_dir, output_dir)?;
    let syntax_set = SyntaxSet::load_defaults_newlines();

    let index_path = input_dir.as_ref().join("index.md");
    let file_string = fs::read_to_string(&index_path)?;
    let mut index_html = convert_to_html(&file_string, &syntax_set)?;

    let articles_list = list_articles(input_dir, output_dir, &syntax_set)?;
    index_html.push_str(&articles_list);

    let index_page = format!("{}\n{}\n{}", INDEX_HEADER, index_html, FOOTER);

    let index_output = output_dir.as_ref().join("index.html");
    fs::write(index_output, &index_page)?;

    Ok(())
}
