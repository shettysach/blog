use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use pulldown_latex::{config::DisplayMode, mathml::push_mathml, Parser, RenderConfig, Storage};
use std::io;
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::{SyntaxReference, SyntaxSet},
};

pub(crate) fn process<'a, I>(events: I, syntax_set: &SyntaxSet) -> io::Result<Vec<Event<'a>>>
where
    I: Iterator<Item = Event<'a>>,
{
    let plain_text = syntax_set.find_syntax_plain_text();
    let mut syntax = plain_text;

    let mut storage = Storage::new();
    let mut code_block = String::new();

    let mut in_code_block = false;
    let mut heading_level = 0;

    events
        .map(|event| {
            Ok(match event {
                Event::Text(t) if heading_level != 0 => {
                    let heading_start = anchorize(&t, heading_level);
                    heading_level = 0;
                    Some(Event::Html(CowStr::from(heading_start)))
                }

                Event::Text(t) if in_code_block => {
                    code_block.push_str(&t);
                    None
                }

                Event::Text(t) => Some(Event::Html(t)), // NOTE: needed?

                Event::Start(Tag::Heading { level, .. }) => {
                    heading_level = level as u8;
                    None
                }

                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    syntax = match kind {
                        CodeBlockKind::Fenced(lang) => {
                            syntax_set.find_syntax_by_token(&lang).unwrap_or(plain_text)
                        }
                        CodeBlockKind::Indented => plain_text,
                    };

                    None
                }

                Event::End(TagEnd::CodeBlock) => {
                    let highlighted = highlight_code(&code_block, syntax, syntax_set)?;
                    in_code_block = false;
                    code_block.clear();

                    Some(Event::Html(CowStr::from(highlighted)))
                }

                Event::DisplayMath(latex) => {
                    let mathml = latex_to_mathml(&latex, &mut storage, DisplayMode::Block)?;
                    Some(Event::Html(CowStr::from(mathml)))
                }

                Event::InlineMath(latex) => {
                    let mathml = latex_to_mathml(&latex, &mut storage, DisplayMode::Inline)?;
                    Some(Event::InlineHtml(CowStr::from(mathml)))
                }

                _ => Some(event),
            })
        })
        .filter_map(|e| e.transpose())
        .collect()
}

fn latex_to_mathml(
    latex: &str,
    storage: &mut Storage,
    display_mode: DisplayMode,
) -> io::Result<String> {
    let mut mathml = String::new();
    let parser = Parser::new(latex, storage);
    let config = RenderConfig {
        display_mode,
        ..Default::default()
    };

    push_mathml(&mut mathml, parser, config)?;
    storage.reset();
    Ok(mathml)
}

fn highlight_code(
    code: &str,
    syntax: &SyntaxReference,
    syntax_set: &SyntaxSet,
) -> io::Result<String> {
    let mut class_generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, syntax_set, ClassStyle::Spaced);

    for line in code.split_inclusive('\n') {
        class_generator
            .parse_html_for_line_which_includes_newline(line)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    let html = class_generator.finalize();
    Ok(format!("<pre><code>{}</code></pre>", html))
}

fn anchorize(text: &str, heading_level: u8) -> String {
    let anchor: String = text
        .to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if c.is_whitespace() {
                Some('_')
            } else {
                None
            }
        })
        .collect();

    format!(
        "<h{} id=\"{}\">{} <a href=\"#{}\" class=\"anchor\">h{}</a>",
        heading_level, anchor, text, anchor, heading_level
    )
}
