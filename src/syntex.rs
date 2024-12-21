// This code is modified from highlight-pulldown
// https://gitlab.com/eguiraud/highlight-pulldown
// Copyright (C) 2023 Enrico Guiraud

// Modifications -
// 1. Added LaTeX to MathML, using the crate,
// `pulldown_latex` by `carloskiki`.
// 2. Uses `syntect`'s ClassedHTMLGenerator to
// generate syntax highlighting with CSS classes,
// instead of predefined themes.
// This enables me to use my own generated
// CSS file, `code.css`.
// 3. Updated dependencies to make it
// compatible with the latest versions.

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

pub struct SynTex {
    syntax_set: SyntaxSet,
}

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

impl SynTex {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }

    pub fn process<'a, It>(&self, events: It) -> io::Result<Vec<Event<'a>>>
    where
        It: Iterator<Item = Event<'a>>,
    {
        let mut is_latex = false;
        let mut in_code_block = false;

        let plain_text = self.syntax_set.find_syntax_plain_text();
        let mut syntax = plain_text;

        let mut storage = Storage::new();
        let mut code_block = String::new();
        let mut out_events = Vec::new();

        for event in events {
            match event {
                Event::Text(t) if in_code_block => code_block.push_str(&t),
                Event::Text(t) => out_events.push(Event::Html(t)),

                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;

                    match kind {
                        CodeBlockKind::Fenced(lang) if lang.as_ref() == "math" => is_latex = true,
                        CodeBlockKind::Fenced(lang) => {
                            syntax = self
                                .syntax_set
                                .find_syntax_by_token(&lang)
                                .unwrap_or(plain_text)
                        }
                        CodeBlockKind::Indented => syntax = plain_text,
                    }
                }

                Event::End(TagEnd::CodeBlock) => {
                    let out_event = if is_latex {
                        self.latex_to_mathml(&code_block, &mut storage, true)?
                    } else {
                        self.highlight_code(&code_block, syntax)?
                    };

                    code_block.clear();
                    out_events.push(Event::Html(CowStr::from(out_event)));

                    is_latex = false;
                    in_code_block = false;
                }

                Event::InlineMath(latex) => {
                    let mathml = self.latex_to_mathml(&latex, &mut storage, false)?;
                    out_events.push(Event::Html(CowStr::from(mathml)));
                }

                Event::DisplayMath(latex) => {
                    let mathml = self.latex_to_mathml(&latex, &mut storage, true)?;
                    out_events.push(Event::Html(CowStr::from(mathml)));
                }

                _ => out_events.push(event),
            }
        }

        Ok(out_events)
    }

    fn latex_to_mathml(
        &self,
        text: &str,
        storage: &mut Storage,
        is_block: bool,
    ) -> io::Result<String> {
        let parser = Parser::new(text, storage);
        let mut mathml = String::new();

        let mathml = if is_block {
            push_mathml(&mut mathml, parser, MATH_BLOCK)?;
            mathml
        } else {
            push_mathml(&mut mathml, parser, MATH_INLINE)?;
            format!("<inline>{mathml}</inline>")
        };

        storage.reset();
        Ok(mathml)
    }

    fn highlight_code(&self, text: &str, syntax: &SyntaxReference) -> io::Result<String> {
        let mut class_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            ClassStyle::Spaced,
        );

        for line in text.split_inclusive('\n') {
            class_generator
                .parse_html_for_line_which_includes_newline(line)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }

        let html = class_generator.finalize();
        let html = format!("<pre><code>{html}</code></pre>");

        Ok(html)
    }
}
