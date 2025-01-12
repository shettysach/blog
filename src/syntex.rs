use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use pulldown_latex::{
    config::{DisplayMode, MathStyle},
    mathml::push_mathml,
    Parser, RenderConfig, Storage,
};
use std::io;
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::{SyntaxReference, SyntaxSet},
};

const MATH_BLOCK: RenderConfig = RenderConfig {
    display_mode: DisplayMode::Block,
    math_style: MathStyle::TeX,
    xml: true,
    annotation: None,
    error_color: (255, 255, 255),
};

const MATH_INLINE: RenderConfig = RenderConfig {
    display_mode: DisplayMode::Inline,
    math_style: MathStyle::TeX,
    xml: true,
    annotation: None,
    error_color: (255, 255, 255),
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
            Event::Text(t) if in_code_block => code_block.push_str(&t),
            Event::Text(t) if heading_level != 0 => {
                let h_start = anchorize(&t, heading_level);
                heading_level = 0;
                out_events.push(Event::Html(CowStr::from(h_start)));
            }
            Event::Text(t) => out_events.push(Event::Html(t)),

            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = level as u8;
            }

            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;

                match kind {
                    CodeBlockKind::Fenced(lang) if lang.as_ref() == "math" => is_latex = true,
                    CodeBlockKind::Fenced(lang) => {
                        syntax = syntax_set.find_syntax_by_token(&lang).unwrap_or(plain_text)
                    }
                    CodeBlockKind::Indented => syntax = plain_text,
                }
            }

            Event::End(TagEnd::CodeBlock) => {
                let out_event = if is_latex {
                    latex_to_mathml(&code_block, &mut storage, true)?
                } else {
                    highlight_code(&code_block, syntax, syntax_set)?
                };

                code_block.clear();
                is_latex = false;
                in_code_block = false;

                out_events.push(Event::Html(CowStr::from(out_event)));
            }

            Event::InlineMath(latex) => {
                let mathml = latex_to_mathml(&latex, &mut storage, false)?;
                out_events.push(Event::InlineHtml(CowStr::from(mathml)));
            }

            Event::DisplayMath(latex) => {
                let mathml = latex_to_mathml(&latex, &mut storage, true)?;
                out_events.push(Event::Html(CowStr::from(mathml)));
            }

            _ => out_events.push(event),
        }
    }

    Ok(out_events)
}

fn latex_to_mathml(text: &str, storage: &mut Storage, is_block: bool) -> io::Result<String> {
    let parser = Parser::new(text, storage);
    let mut mathml = String::new();

    push_mathml(
        &mut mathml,
        parser,
        if is_block { MATH_BLOCK } else { MATH_INLINE },
    )?;

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
    let html = format!("<pre><code>{html}</code></pre>");

    Ok(html)
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
