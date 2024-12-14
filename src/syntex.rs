// Copyright (C) 2023 Enrico Guiraud
// This code is modified from highlight-pulldown
// https://gitlab.com/eguiraud/highlight-pulldown

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use pulldown_latex::{config::DisplayMode, mathml::push_mathml, Parser, RenderConfig, Storage};
use std::io;
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
};

// Modifications -
// 1. Added LaTeX to MathML, using the crate,
// `pulldown_latex` by `carloskiki`.
// 2. Uses ClassedHTMLGenerator to generate
// syntax highlighting with CSS classes,
// instead of predefined themes.
// This enables me to use my own generated
// CSS file, `tokyonight.css`.
// 3. Updated dependencies to make it
// compatible with the latest versions.

pub struct SynTex<'a> {
    syntax_set: SyntaxSet,
    render_config: RenderConfig<'a>,
}

impl<'a> SynTex<'a> {
    pub fn new() -> SynTex<'a> {
        SynTex {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            render_config: RenderConfig::<'a> {
                display_mode: DisplayMode::Block,
                xml: true,
                ..Default::default()
            },
        }
    }

    pub fn highlight<It>(&self, events: It) -> io::Result<Vec<Event<'a>>>
    where
        It: Iterator<Item = Event<'a>>,
    {
        let mut is_latex = false;
        let mut in_code_block = false;

        let mut syntax = self.syntax_set.find_syntax_plain_text();
        let mut storage = Storage::new();

        let mut inside_text = String::new();
        let mut out_events = Vec::new();

        for event in events {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;

                    match kind {
                        CodeBlockKind::Fenced(CowStr::Borrowed("math")) => is_latex = true,
                        CodeBlockKind::Fenced(lang) => {
                            syntax = self
                                .syntax_set
                                .find_syntax_by_token(&lang)
                                .unwrap_or(syntax)
                        }
                        CodeBlockKind::Indented => {}
                    }
                }
                Event::End(TagEnd::CodeBlock) => {
                    if !in_code_block {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "Code block has not been closed correctly.",
                        ));
                    }

                    in_code_block = false;

                    let out_event = if is_latex {
                        let parser = Parser::new(&inside_text, &storage);
                        let mut mathml = String::new();
                        push_mathml(&mut mathml, parser, self.render_config)?;

                        storage.reset();
                        is_latex = false;

                        mathml
                    } else {
                        let mut class_generator = ClassedHTMLGenerator::new_with_class_style(
                            syntax,
                            &self.syntax_set,
                            ClassStyle::Spaced,
                        );

                        for line in inside_text.split_inclusive("\n") {
                            class_generator
                                .parse_html_for_line_which_includes_newline(line)
                                .unwrap();
                        }

                        let html = class_generator.finalize();
                        let html = format!("<pre><code>{html}</code></pre>");

                        html
                    };

                    inside_text.clear();

                    let out_event = Event::Html(CowStr::from(out_event));
                    out_events.push(out_event);
                }
                Event::Text(t) => {
                    if in_code_block {
                        inside_text.push_str(&t);
                    } else {
                        out_events.push(Event::Text(t));
                    }
                }
                _ => out_events.push(event),
            }
        }

        Ok(out_events)
    }
}
