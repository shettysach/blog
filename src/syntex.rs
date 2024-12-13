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

pub struct SynTex {
    syntax_set: SyntaxSet,
}

impl SynTex {
    pub fn new() -> SynTex {
        SynTex {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }

    pub fn highlight<'a, It>(&self, events: It) -> io::Result<Vec<Event<'a>>>
    where
        It: Iterator<Item = Event<'a>>,
    {
        let mut is_latex = false;
        let mut in_code_block = false;

        let mut syntax = self.syntax_set.find_syntax_plain_text();
        let mut storage = Storage::new();
        let config = RenderConfig::<'_> {
            display_mode: DisplayMode::Block,
            xml: true,
            ..Default::default()
        };

        let mut inside_text = String::new();
        let mut out_events = Vec::new();

        for event in events {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    match kind {
                        CodeBlockKind::Fenced(CowStr::Borrowed("math")) => is_latex = true,
                        CodeBlockKind::Fenced(language) => {
                            syntax = self
                                .syntax_set
                                .find_syntax_by_token(&language)
                                .unwrap_or(syntax)
                        }
                        CodeBlockKind::Indented => {}
                    }
                    in_code_block = true;
                }
                Event::End(TagEnd::CodeBlock) => {
                    if !in_code_block {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "Code block has not been closed correctly.",
                        ));
                    }

                    let push_event = if is_latex {
                        let parser = Parser::new(&inside_text, &storage);

                        let mut mathml = String::new();
                        push_mathml(&mut mathml, parser, config).unwrap();

                        storage.reset();
                        is_latex = false;

                        CowStr::from(mathml)
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

                        CowStr::from(html)
                    };

                    out_events.push(Event::Html(push_event));
                    inside_text.clear();
                    in_code_block = false;
                }
                Event::Text(t) => {
                    if in_code_block {
                        inside_text.push_str(&t);
                    } else {
                        out_events.push(Event::Text(t));
                    }
                }
                _ => {
                    out_events.push(event);
                }
            }
        }

        Ok(out_events)
    }
}
