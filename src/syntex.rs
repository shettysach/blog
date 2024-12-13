// Copyright (C) 2023 Enrico Guiraud
//
// This code is modified from highlight-pulldown
// https://gitlab.com/eguiraud/highlight-pulldown
//
// Modifications -
// 1. Uses ClassedHTMLGenerator to generate
// syntax highlighting with CSS classes,
// instead of predefined themes.
// This enables me to use my own generated
// CSS file, `tokyonight.css`.
// 2. Updated dependencies to make it
// compatible with the latest versions.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
};

pub struct PulldownHighlighter {
    syntaxset: SyntaxSet,
}

impl PulldownHighlighter {
    pub fn new() -> PulldownHighlighter {
        PulldownHighlighter {
            syntaxset: SyntaxSet::load_defaults_newlines(),
        }
    }

    // TODO: better error handling
    pub fn highlight<'a, It>(&self, events: It) -> Vec<Event<'a>>
    where
        It: Iterator<Item = Event<'a>>,
    {
        let mut in_code_block = false;

        let mut syntax = self.syntaxset.find_syntax_plain_text();

        let mut to_highlight = String::new();
        let mut out_events = Vec::new();

        for event in events {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    match kind {
                        CodeBlockKind::Fenced(lang) => {
                            syntax = self.syntaxset.find_syntax_by_token(&lang).unwrap_or(syntax)
                        }
                        CodeBlockKind::Indented => {}
                    }
                    in_code_block = true;
                }
                Event::End(TagEnd::CodeBlock) => {
                    if !in_code_block {
                        panic!("this should never happen");
                    }

                    let mut class_generator = ClassedHTMLGenerator::new_with_class_style(
                        syntax,
                        &self.syntaxset,
                        ClassStyle::Spaced,
                    );

                    for line in to_highlight.split_inclusive("\n") {
                        class_generator
                            .parse_html_for_line_which_includes_newline(line)
                            .unwrap();
                    }

                    to_highlight.clear();
                    in_code_block = false;

                    let html = class_generator.finalize();
                    let html = format!("<pre>{html}</pre>");
                    out_events.push(Event::Html(CowStr::from(html)));
                }
                Event::Text(t) => {
                    if in_code_block {
                        to_highlight.push_str(&t);
                    } else {
                        out_events.push(Event::Text(t));
                    }
                }
                _ => {
                    out_events.push(event);
                }
            }
        }

        out_events
    }
}
