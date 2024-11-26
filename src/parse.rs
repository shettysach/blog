use pulldown_cmark::{html, Parser};
use std::{fmt::Display, fs, io::Write, path::Path};

const HEADER: &str = include_str!("../template/header.txt");
const FOOTER: &str = include_str!("../template/footer.txt");

fn generate_page(content: &str) -> String {
    format!("{}\n{}\n{}", HEADER, content, FOOTER)
}

fn markdown_to_html<P: AsRef<Path>>(input_path: P) -> String {
    let markdown = fs::read_to_string(&input_path).expect("Cannot find `index.md`.");
    let parser = Parser::new(&markdown);

    let mut html = String::new();
    html::push_html(&mut html, parser);

    html
}

fn save_html<P: AsRef<Path>>(content: &str, output_path: P) {
    let mut file = fs::File::create(&output_path).unwrap();
    file.write_all(content.as_bytes()).unwrap()
}

fn process_articles<P: AsRef<Path> + Display>(input_dir: P, output_dir: P) -> String {
    let article_list = fs::read_dir(input_dir)
        .unwrap()
        .flatten()
        .filter_map(|article| {
            let article_path = article.path();
            let is_markdown = article_path.extension() == Some("md".as_ref());
            let is_index = article_path.file_stem() == Some("index".as_ref());

            (is_markdown && !is_index).then_some(article_path)
        })
        .map(|article_path| {
            let article_name = article_path.file_stem().unwrap().to_string_lossy();

            let html_content = markdown_to_html(&article_path);
            let page_content = generate_page(&html_content);
            let output_file = format!("{}/{}.html", output_dir, article_name);

            save_html(&page_content, &output_file);

            format!(
                "<li><a href=\"./{}.html\">{}</a></li>",
                article_name, article_name
            )
        })
        .collect::<Vec<String>>();

    format!("<h2>Articles</h2><ul>\n{}\n</ul>", article_list.join("\n"))
}

pub(crate) fn process_index<P: AsRef<Path> + Display + Copy>(input_dir: P, output_dir: P) {
    let index_path = format!("{input_dir}/index.md");
    let mut index_html = markdown_to_html(index_path);

    let blog_list = process_articles(input_dir, output_dir);
    index_html.push_str(&format!("<div>\n{}\n</div>\n", blog_list));

    let styles_input = "./styles/styles.css";
    let styles_output = format!("./{output_dir}/styles.css");
    fs::copy(styles_input, &styles_output).unwrap();

    let html_content = generate_page(&index_html);
    save_html(&html_content, format!("./{output_dir}/index.html"));
}
