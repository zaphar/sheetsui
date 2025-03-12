use core::panic;
use std::collections::BTreeSet;

use ratatui::{
    text::{Line, Span, Text},
    widgets::Widget,
};

use pulldown_cmark::{Event, HeadingLevel, LinkType, Parser, Tag, TagEnd, TextMergeStream};

enum State {
    Para,
    NumberList,
    BulletList,
    Heading,
    BlockQuote,
}

struct WidgetWriter<'i> {
    input: &'i str,
    state_stack: Vec<State>,
    heading_stack: Vec<&'static str>,
    list_stack: Vec<u64>,
    accumulator: String,
    lines: Vec<String>,
    links: BTreeSet<String>,
}

impl<'i> WidgetWriter<'i>
{
    pub fn from_str(input: &'i str) -> Self {
        Self {
            input,
            state_stack: Default::default(),
            heading_stack: Default::default(),
            list_stack: Default::default(),
            accumulator: Default::default(),
            lines: Default::default(),
            links: Default::default(),
        }
    }

    pub fn parse(&mut self) {
        let iter = TextMergeStream::new(Parser::new(self.input));
        for event in iter {
            match event {
                Event::Start(tag) => {
                    self.start_tag(&tag);
                },
                Event::End(tag) => {
                    self.end_tag(tag);
                },
                Event::Text(txt)
                | Event::Code(txt)
                | Event::InlineHtml(txt)
                | Event::Html(txt) => {
                    let prefix = if let Some(State::BlockQuote) = self.state_stack.first() {
                        "| "
                    } else {
                        ""
                    };
                    for ln in txt.lines() {
                        self.accumulator.push_str(prefix);
                        self.accumulator.push_str(ln);
                    }
                },
                Event::Rule => { /* noop */ },
                Event::SoftBreak => { /* noop */ },
                Event::HardBreak => { /* noop */ },
                // We don't support these
                Event::InlineMath(_) => todo!(),
                Event::DisplayMath(_) => todo!(),
                Event::FootnoteReference(_) => todo!(),
                Event::TaskListMarker(_) => todo!(),
            }
        }
    }

    fn start_tag(&mut self, tag: &Tag<'i>) {
        match tag {
            Tag::Paragraph => {
                self.state_stack.push(State::Para);
            },
            Tag::Heading { level, id: _id, classes: _classes, attrs: _attrs } => {
                self.heading_stack.push(match level {
                    HeadingLevel::H1 => "1",
                    HeadingLevel::H2 => "2",
                    HeadingLevel::H3 => "3",
                    HeadingLevel::H4 => "4",
                    HeadingLevel::H5 => "5",
                    HeadingLevel::H6 => "6",
                });
                self.state_stack.push(State::Heading);
                let prefix = self.heading_stack.join(".");
                self.accumulator.push_str(&prefix);
                self.accumulator.push_str(" ");
            },
            Tag::List(Some(first)) => {
                self.list_stack.push(*first);
                self.state_stack.push(State::NumberList);
            },
            Tag::List(None) => {
                self.state_stack.push(State::BulletList);
            },
            Tag::Item => {
                if let Some(State::BulletList) = self.state_stack.first() {
                    self.accumulator.push_str("- ");
                } else if let Some(State::NumberList) = self.state_stack.first() {
                    let num = self.list_stack.pop().unwrap_or(1);
                    self.accumulator.push_str(&format!("{}. ", num));
                    self.list_stack.push(num + 1);
                }
                panic!("No list type in our state stack");
            },
            Tag::Emphasis => {
                self.accumulator.push_str("*");
            },
            Tag::Strong => {
                self.accumulator.push_str("**");
            },
            Tag::Link { link_type, dest_url, title, id } => {
                let dest = match link_type {
                    // [foo](bar)
                    LinkType::Inline => format!("({})", dest_url),
                    // [foo][bar]
                    LinkType::Reference => format!("[{}]", id),
                    // [foo]
                    LinkType::Shortcut => format!("[{}]", title),
                    // These are unsupported right now
                    LinkType::ReferenceUnknown => todo!(),
                    LinkType::Collapsed => todo!(),
                    LinkType::CollapsedUnknown => todo!(),
                    LinkType::ShortcutUnknown => todo!(),
                    LinkType::Autolink => todo!(),
                    LinkType::Email => todo!(),
                    LinkType::WikiLink { has_pothole: _ } => todo!(),
                };
                self.accumulator.push_str(&format!("[{}]{}", title, dest));
                self.links.insert(dest);
            },
            Tag::BlockQuote(_) => {
                self.state_stack.push(State::BlockQuote);
            },
            // these are all noops
            Tag::CodeBlock(_) => {},
            Tag::HtmlBlock => {},
            Tag::FootnoteDefinition(_) => {},
            Tag::DefinitionList => {},
            Tag::DefinitionListTitle => {},
            Tag::DefinitionListDefinition => {},
            Tag::Table(_) => {},
            Tag::TableHead => {},
            Tag::TableRow => {},
            Tag::TableCell => {},
            Tag::Strikethrough => {},
            Tag::Superscript => {},
            Tag::Subscript => {}
            Tag::Image { link_type: _link_type, dest_url: _dest_url, title: _title, id: _id } => {},
            Tag::MetadataBlock(_) => {},
        }
    }
    
    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                self.state_stack.pop();
                self.lines.push("\n".to_owned());
            },
            TagEnd::Heading(_level) => {
                self.heading_stack.pop();
                self.state_stack.pop();
                self.lines.extend(self.accumulator.lines().map(|s| s.to_owned()));
                self.accumulator.clear();
            },
            TagEnd::List(_ordered) => {
                self.state_stack.pop();
            },
            TagEnd::BlockQuote(_kind) => {
                self.state_stack.pop();
            },
            TagEnd::CodeBlock => {
                todo!()
            },
            TagEnd::HtmlBlock => {
                todo!()
            },
            TagEnd::Item => { /* noop */ },
            TagEnd::Link => { /* noop */ },
            // We don't support these
            TagEnd::FootnoteDefinition => todo!(),
            TagEnd::DefinitionList => todo!(),
            TagEnd::DefinitionListTitle => todo!(),
            TagEnd::DefinitionListDefinition => todo!(),
            TagEnd::Table => todo!(),
            TagEnd::TableHead => todo!(),
            TagEnd::TableRow => todo!(),
            TagEnd::TableCell => todo!(),
            TagEnd::Emphasis => {
                self.accumulator.push_str("*");
            },
            TagEnd::Strong => {
                self.accumulator.push_str("**");
            },
            TagEnd::Strikethrough => todo!(),
            TagEnd::Superscript => todo!(),
            TagEnd::Subscript => todo!(),
            TagEnd::Image => todo!(),
            TagEnd::MetadataBlock(_) => todo!(),
        }
    }
}
