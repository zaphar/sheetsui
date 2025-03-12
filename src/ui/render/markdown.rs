use std::collections::BTreeSet;

use crossterm::event::KeyCode;
use ratatui::{text::Text, widgets::Widget};

use pulldown_cmark::{Event,LinkType, Parser, Tag, TextMergeStream};

//enum State {
//    Para,
//    NumberList,
//    BulletList,
//    Heading,
//    BlockQuote,
//}

#[derive(Debug, Clone, PartialEq)]
pub struct Markdown {
    input: String,
    links: BTreeSet<String>,
}

impl Markdown {
    pub fn from_str(input: &str) -> Self {
        let mut me = Self {
            input: input.to_owned(),
            links: Default::default(),
        };
        me.parse();
        me
    }

    fn parse(&mut self) {
        let input = self.input.clone();
        let iter = TextMergeStream::new(Parser::new(&input));
        for event in iter {
            match event {
                Event::Start(tag) => {
                    self.start_tag(&tag);
                }
                _ => { /* noop */ }
            }
        }
    }

    fn start_tag(&mut self, tag: &Tag<'_>) {
        match tag {
            Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            } => {
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
                self.links.insert(dest);
            }
            _ => { /* noop */ }
        }
    }

    pub fn handle_input(&self, code: KeyCode) -> Option<String> {
        let num = match code {
            KeyCode::Char('0') => 0,
            KeyCode::Char('1') => 1,
            KeyCode::Char('2') => 2,
            KeyCode::Char('3') => 3,
            KeyCode::Char('4') => 4,
            KeyCode::Char('5') => 5,
            KeyCode::Char('6') => 6,
            KeyCode::Char('7') => 7,
            KeyCode::Char('8') => 8,
            KeyCode::Char('9') => 9,
            _ => return None,
        };
        self.links.iter().nth(num).cloned()
    }

    pub fn get_text<'w>(&'w self) -> Text<'_> {
        Text::raw(&self.input)
    }
}

// TODO(jwall): We need this to be lines instead of just a render.
impl Widget for Markdown {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let text = Text::raw(self.input);
        text.render(area, buf);
    }
}
