use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use pulldown_latex::{config::DisplayMode, mathml::push_mathml, Parser, RenderConfig, Storage};
use std::io;
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::{SyntaxReference, SyntaxSet},
};

pub fn process<'a, Iter>(events: Iter, syntax_set: &SyntaxSet) -> io::Result<Vec<Event<'a>>>
where
    Iter: Iterator<Item = Event<'a>>,
{
    let mut is_latex = false;
    let mut in_code_block = false;
    let mut heading_level = 0;

    let plain_text = syntax_set.find_syntax_plain_text();
    let mut syntax = plain_text;

    let mut storage = Storage::new();
    let mut code_block = String::new();
    let mut out_events = Vec::new();

    for event in events {
        match event {
            Event::Text(t) => {
                if heading_level != 0 {
                    let heading_start = anchorize(&t, heading_level);
                    heading_level = 0;
                    out_events.push(Event::Html(CowStr::from(heading_start)));
                } else if in_code_block {
                    code_block.push_str(&t)
                } else {
                    out_events.push(Event::Html(t))
                }
            }

            Event::Start(Tag::Heading { level, .. }) => heading_level = level as u8,
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;

                match kind {
                    CodeBlockKind::Fenced(lang) => {
                        if lang.as_ref() == "math" {
                            is_latex = true
                        } else {
                            syntax = syntax_set.find_syntax_by_token(&lang).unwrap_or(plain_text)
                        }
                    }
                    CodeBlockKind::Indented => syntax = plain_text,
                }
            }

            Event::End(TagEnd::CodeBlock) => {
                let out_event = if is_latex {
                    latex_to_mathml(&code_block, &mut storage, DisplayMode::Block)?
                } else {
                    highlight_code(&code_block, syntax, syntax_set)?
                };

                code_block.clear();
                is_latex = false;
                in_code_block = false;

                out_events.push(Event::Html(CowStr::from(out_event)));
            }

            Event::DisplayMath(latex) => {
                let mathml = latex_to_mathml(&latex, &mut storage, DisplayMode::Block)?;
                out_events.push(Event::Html(CowStr::from(mathml)));
            }

            Event::InlineMath(latex) => {
                let mathml = latex_to_mathml(&latex, &mut storage, DisplayMode::Inline)?;
                out_events.push(Event::InlineHtml(CowStr::from(mathml)));
            }

            _ => out_events.push(event),
        }
    }

    Ok(out_events)
}

fn latex_to_mathml(
    text: &str,
    storage: &mut Storage,
    display_mode: DisplayMode,
) -> io::Result<String> {
    let mut mathml = String::new();
    let parser = Parser::new(text, storage);
    let config = RenderConfig {
        display_mode,
        ..Default::default()
    };

    push_mathml(&mut mathml, parser, config)?;

    storage.reset();
    Ok(mathml)
}

fn highlight_code(
    text: &str,
    syntax: &SyntaxReference,
    syntax_set: &SyntaxSet,
) -> io::Result<String> {
    let mut class_generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, syntax_set, ClassStyle::Spaced);

    for line in text.split_inclusive('\n') {
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
        "<h{} id=\"{}\">{} <a href=\"#{}\" class=\"anchor\">¶</a>",
        heading_level, anchor, text, anchor
    )
}
