use std::collections::BTreeSet;

use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Widget,
};

use pulldown_cmark::{Event, LinkType, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq)]
pub struct Markdown {
    input: String,
    links: BTreeSet<String>,
    parsed_text: Option<Text<'static>>,
}

/// Define the different states a markdown parser can be in
#[derive(Debug, Clone, PartialEq)]
enum MarkdownState {
    Normal,
    Heading(pulldown_cmark::HeadingLevel),
    Strong,
    Emphasis,
    Code,
    List(ListState),
}

/// Track list state including nesting level and type
#[derive(Debug, Clone, PartialEq)]
struct ListState {
    list_type: ListType,
    nesting_level: usize,
    item_number: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum ListType {
    Ordered,
    Unordered,
}

impl Markdown {
    pub fn from_str(input: &str) -> Self {
        let mut me = Self {
            input: input.to_owned(),
            links: Default::default(),
            parsed_text: None,
        };
        me.parse();
        me
    }

    fn parse(&mut self) {
        let input = self.input.clone();
        
        let parser = pulldown_cmark::TextMergeStream::new(Parser::new(&input));

        let mut current_line = Line::default();
        let mut lines: Vec<Line> = Vec::new();
        let mut state_stack: Vec<MarkdownState> = vec![MarkdownState::Normal];

        for event in parser {
            match event {
                Event::Start(tag) => {
                    match &tag {
                        Tag::Heading { level, .. } => {
                            if !current_line.spans.is_empty() {
                                lines.push(current_line);
                            }

                            // Add heading style based on level
                            let heading_style = match level {
                                pulldown_cmark::HeadingLevel::H1 => {
                                    Style::default().add_modifier(Modifier::BOLD)
                                }
                                pulldown_cmark::HeadingLevel::H2 => {
                                    Style::default().add_modifier(Modifier::ITALIC)
                                }
                                _ => Style::default().fg(Color::Blue),
                            };
                            current_line = Line::styled("", heading_style);
                            state_stack.push(MarkdownState::Heading(*level));
                        }
                        Tag::Paragraph => {
                            if !current_line.spans.is_empty() {
                                lines.push(current_line);
                                current_line = Line::default();
                            }
                        }
                        Tag::Strong => {
                            state_stack.push(MarkdownState::Strong);
                        }
                        Tag::Emphasis => {
                            state_stack.push(MarkdownState::Emphasis);
                        }
                        Tag::CodeBlock(_) => {
                            state_stack.push(MarkdownState::Code);
                        }
                        Tag::List(list_type) => {
                            if !current_line.spans.is_empty() {
                                lines.push(current_line);
                                current_line = Line::default();
                            }

                            // Determine list type and nesting level
                            let list_type = match list_type {
                                Some(_) => ListType::Ordered,
                                None => ListType::Unordered,
                            };

                            // Calculate nesting level based on existing lists in the stack
                            let nesting_level = state_stack
                                .iter()
                                .filter(|state| matches!(state, MarkdownState::List(_)))
                                .count();

                            state_stack.push(MarkdownState::List(ListState {
                                list_type,
                                nesting_level,
                                item_number: 0,
                            }));
                        }
                        Tag::Item => {
                            if !current_line.spans.is_empty() {
                                lines.push(current_line);
                                current_line = Line::default();
                            }

                            // Find the current list state and increment its item number
                            for state in state_stack.iter_mut().rev() {
                                if let MarkdownState::List(list_state) = state {
                                    list_state.item_number += 1;

                                    // Add appropriate indentation based on nesting level
                                    let indent = "  ".repeat(list_state.nesting_level);

                                    // Add appropriate marker based on list type
                                    let marker = match list_state.list_type {
                                        ListType::Unordered => "* ".to_string(),
                                        ListType::Ordered => {
                                            format!("{}. ", list_state.item_number)
                                        }
                                    };

                                    current_line
                                        .spans
                                        .push(Span::raw(format!("{}{}", indent, marker)));
                                    break;
                                }
                            }
                        }
                        Tag::Link {
                            link_type: _,
                            dest_url: _,
                            title: _,
                            id: _,
                        } => {
                            self.handle_link_tag(&tag);
                        }
                        Tag::BlockQuote(_) => todo!(),
                        Tag::Strikethrough => todo!(),
                        Tag::Superscript => todo!(),
                        Tag::Subscript => todo!(),
                        _ => {
                            // noop
                        }
                    }
                }
                Event::End(tag) => {
                    match tag {
                        TagEnd::Heading { .. } => {
                            lines.push(current_line);
                            current_line = Line::default();
                            state_stack.pop();
                        }
                        TagEnd::Paragraph => {
                            lines.push(current_line);
                            lines.push(Line::default()); // Add empty line after paragraph
                            current_line = Line::default();
                        }
                        TagEnd::Strong => {
                            state_stack.pop();
                        }
                        TagEnd::Emphasis => {
                            state_stack.pop();
                        }
                        TagEnd::CodeBlock => {
                            state_stack.pop();
                        }
                       TagEnd::Item => {
                            // Push the current line to preserve the list item
                            if !current_line.spans.is_empty() {
                                lines.push(current_line);
                                current_line = Line::default();
                            }
                        }
                        TagEnd::List(_) => {
                            state_stack.pop();

                            // Only add an empty line if we're back to the root level
                            if state_stack
                                .iter()
                                .filter(|state| matches!(state, MarkdownState::List(_))).count() == 0
                            {
                                //lines.push(Line::default()); // Add empty line after list
                            }
                        }
                        _ => {}
                    }
                }
                Event::InlineMath(text) 
                | Event::Code(text)
                | Event::InlineHtml(text)
                | Event::DisplayMath(text)
                | Event::Html(text)
                | Event::Text(text) => {
                    let mut style = Style::default();

                    // Apply style based on current state
                    for state in state_stack.iter().rev() {
                        match state {
                            MarkdownState::Heading(_) => {
                                // Style already applied to the line
                                break;
                            }
                            MarkdownState::Strong => {
                                style = style.add_modifier(Modifier::BOLD);
                            }
                            MarkdownState::Emphasis => {
                                style = style.add_modifier(Modifier::ITALIC);
                            }
                            //MarkdownState::Code => {
                            //    style = style.fg(Color::Yellow);
                            //}
                            _ => {
                            }
                        }
                    }

                    // Add the text with appropriate styling
                    current_line
                        .spans
                        .push(Span::styled(text.to_string(), style));
                }
                Event::SoftBreak => {
                    current_line.spans.push(Span::raw(" "));
                }
                Event::HardBreak => {
                    lines.push(current_line);
                    current_line = Line::default();
                }
                Event::FootnoteReference(_) => {},
                Event::Rule => {},
                Event::TaskListMarker(_) => {},
            }
        }

        // Add any remaining content
        if !current_line.spans.is_empty() {
            lines.push(current_line);
        }

        self.parsed_text = Some(Text::from(lines));
    }

    fn handle_link_tag(&mut self, tag: &Tag<'_>) {
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
                    LinkType::ReferenceUnknown => String::from("[unknown]"),
                    LinkType::Collapsed => String::from("[collapsed]"),
                    LinkType::CollapsedUnknown => String::from("[collapsed unknown]"),
                    LinkType::ShortcutUnknown => String::from("[shortcut unknown]"),
                    LinkType::Autolink => dest_url.to_string(),
                    LinkType::Email => dest_url.to_string(),
                    LinkType::WikiLink { has_pothole: _ } => String::from("[wiki]"),
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

    pub fn get_text(&self) -> Text {
        if let Some(ref parsed) = self.parsed_text {
            parsed.clone()
        } else {
            Text::raw(&self.input)
        }
    }
}

impl Widget for Markdown {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if let Some(parsed) = self.parsed_text {
            parsed.render(area, buf);
        } else {
            let text = Text::raw(self.input);
            text.render(area, buf);
        }
    }
}

// TODO(zaphar): Move this into a proper test file.
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Text;

    #[test]
    fn test_empty_markdown() {
        let md = Markdown::from_str("");
        let text = md.get_text();
        assert_eq!(text.lines.len(), 0);
    }

    #[test]
    fn test_simple_paragraph() {
        let md = Markdown::from_str("This is a simple paragraph.");
        let text = md.get_text();
        assert_eq!(text.lines.len(), 2); // Paragraph + empty line
        assert_eq!(text.lines[0].spans[0].content, "This is a simple paragraph.");
    }

    #[test]
    fn test_headings() {
        let md = Markdown::from_str("# Heading 1\n## Heading 2\n### Heading 3");
        let text = md.get_text();
        
        // Should have 3 headings
        assert_eq!(text.lines.len(), 3);
        
        // Check content
        assert_eq!(text.lines[0].spans[0].content, "Heading 1");
        assert_eq!(text.lines[1].spans[0].content, "Heading 2");
        assert_eq!(text.lines[2].spans[0].content, "Heading 3");
        
        // Check styling (we can't directly check the style, but we can verify it's different)
        assert!(text.lines[0].style != text.lines[1].style);
    }

    #[test]
    fn test_emphasis() {
        let md = Markdown::from_str("Normal *italic* **bold** text");
        let text = md.get_text();
        
        assert_eq!(text.lines.len(), 2); // Paragraph + empty line
        
        // Check spans - should have 4 spans: normal, italic, bold, normal
        assert_eq!(text.lines[0].spans.len(), 5);
        assert_eq!(text.lines[0].spans[0].content, "Normal ");
        assert_eq!(text.lines[0].spans[1].content, "italic");
        assert_eq!(text.lines[0].spans[2].content, " ");
        assert_eq!(text.lines[0].spans[3].content, "bold");
        assert_eq!(text.lines[0].spans[4].content, " text");
        
        // Check that styles are different
        assert!(text.lines[0].spans[0].style != text.lines[0].spans[1].style);
        assert!(text.lines[0].spans[1].style != text.lines[0].spans[2].style);
    }

    #[test]
    fn test_unordered_list() {
        let md = Markdown::from_str("* Item 1\n* Item 2\n* Item 3");
        let text = md.get_text();
        
        // Should have 4 lines: 3 items + empty line after list
        assert_eq!(text.lines.len(), 3);
        
        // Check content with markers
        assert_eq!(text.lines[0].spans[0].content, "* ");
        assert_eq!(text.lines[0].spans[1].content, "Item 1");
        
        assert_eq!(text.lines[1].spans[0].content, "* ");
        assert_eq!(text.lines[1].spans[1].content, "Item 2");
        
        assert_eq!(text.lines[2].spans[0].content, "* ");
        assert_eq!(text.lines[2].spans[1].content, "Item 3");
    }

    #[test]
    fn test_ordered_list() {
        let md = Markdown::from_str("1. First item\n2. Second item\n3. Third item");
        let text = md.get_text();
        
        // Should have 4 lines: 3 items + empty line after list
        assert_eq!(text.lines.len(), 3);
        
        // Check content with markers
        assert_eq!(text.lines[0].spans[0].content, "1. ");
        assert_eq!(text.lines[0].spans[1].content, "First item");
        
        assert_eq!(text.lines[1].spans[0].content, "2. ");
        assert_eq!(text.lines[1].spans[1].content, "Second item");
        
        assert_eq!(text.lines[2].spans[0].content, "3. ");
        assert_eq!(text.lines[2].spans[1].content, "Third item");
    }

    #[test]
    fn test_nested_lists() {
        let md = Markdown::from_str("* Item 1\n  * Nested 1\n  * Nested 2\n* Item 2");
        let text = md.get_text();
        
        // Should have 5 lines: 4 items + empty line after list
        assert_eq!(text.lines.len(), 4);
        
        // Check indentation and markers
        assert_eq!(text.lines[0].spans[0].content, "* ");
        assert_eq!(text.lines[0].spans[1].content, "Item 1");
        
        assert_eq!(text.lines[1].spans[0].content, "  * ");
        assert_eq!(text.lines[1].spans[1].content, "Nested 1");
        
        assert_eq!(text.lines[2].spans[0].content, "  * ");
        assert_eq!(text.lines[2].spans[1].content, "Nested 2");
        
        assert_eq!(text.lines[3].spans[0].content, "* ");
        assert_eq!(text.lines[3].spans[1].content, "Item 2");
    }

    #[test]
    fn test_mixed_list_types() {
        let md = Markdown::from_str("1. First\n   * Nested bullet\n2. Second");
        let text = md.get_text();
        
        // Should have 4 lines: 3 items + empty line after list
        assert_eq!(text.lines.len(), 3);
        
        assert_eq!(text.lines[0].spans[0].content, "1. ");
        assert_eq!(text.lines[0].spans[1].content, "First");
        
        assert_eq!(text.lines[1].spans[0].content, "  * ");
        assert_eq!(text.lines[1].spans[1].content, "Nested bullet");
        
        assert_eq!(text.lines[2].spans[0].content, "2. ");
        assert_eq!(text.lines[2].spans[1].content, "Second");
    }

    #[test]
    fn test_links() {
        let md = Markdown::from_str("[Link text](https://example.com)");
        let text = md.get_text();
        
        // Should have 2 lines: paragraph + empty line
        assert_eq!(text.lines.len(), 2);
        
        // Check link text is rendered
        assert_eq!(text.lines[0].spans[0].content, "Link text");
        
        // Check link is stored
        assert!(md.links.contains(&String::from("(https://example.com)")));
    }

    #[test]
    fn test_handle_input() {
        let md = Markdown::from_str("[Link 1](https://example1.com)\n[Link 2](https://example2.com)");
        
        // Test valid key input
        let link1 = md.handle_input(KeyCode::Char('0'));
        let link2 = md.handle_input(KeyCode::Char('1'));
        
        assert!(link1.is_some());
        assert!(link2.is_some());
        
        // Test invalid key input
        let invalid = md.handle_input(KeyCode::Enter);
        assert!(invalid.is_none());
    }

    #[test]
    fn test_complex_document() {
        let markdown = r#"
# Main Heading

This is a paragraph with *italic* and **bold** text.

## Subheading

* List item 1
* List item 2
  * Nested item 1
  * Nested item 2
* List item 3

1. Ordered item 1
2. Ordered item 2

[Link to example](https://example.com)
"#;
        
        let md = Markdown::from_str(markdown);
        let text = md.get_text();
        
        // Basic validation that parsing worked
        assert!(text.lines.len() > 10);
        
        // Check link is stored
        assert!(md.links.contains(&String::from("(https://example.com)")));
    }
}
