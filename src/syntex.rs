// Copyright (C) 2023 Enrico Guiraud
// This code is modified from highlight-pulldown
// https://gitlab.com/eguiraud/highlight-pulldown
//
// Modifications -
// 1. Added Latex to MathML, using the crate,
// `pulldown_latex` by `carloskiki`.
// 2. Uses ClassedHTMLGenerator to generate
// syntax highlighting with CSS classes,
// instead of predefined themes.
// This enables me to use my own generated
// CSS file, `tokyonight.css`.
// 3. Updated dependencies to make it
// compatible with the latest versions.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use pulldown_latex::{mathml::push_mathml, Parser, Storage};
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
        let mut is_latex = false;
        let mut in_code_block = false;

        let mut syntax = self.syntaxset.find_syntax_plain_text();

        let mut inside_text = String::new();
        let mut out_events = Vec::new();

        for event in events {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    match kind {
                        CodeBlockKind::Fenced(CowStr::Borrowed("math")) => is_latex = true,
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

                    if is_latex {
                        let storage = Storage::new();
                        let parser = Parser::new(&inside_text, &storage);
                        let config = Default::default();

                        let mut mathml = String::new();
                        push_mathml(&mut mathml, parser, config).unwrap();

                        let mathml = format!(
                            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\">{mathml}</math>"
                        );

                        out_events.push(Event::Html(CowStr::from(mathml)));
                        is_latex = false
                    } else {
                        let mut class_generator = ClassedHTMLGenerator::new_with_class_style(
                            syntax,
                            &self.syntaxset,
                            ClassStyle::Spaced,
                        );

                        for line in inside_text.split_inclusive("\n") {
                            class_generator
                                .parse_html_for_line_which_includes_newline(line)
                                .unwrap();
                        }

                        let html = class_generator.finalize();
                        let html = format!("<pre>{html}</pre>");
                        let html = CowStr::from(html);
                        out_events.push(Event::Html(html));
                    }

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

        out_events
    }
}
