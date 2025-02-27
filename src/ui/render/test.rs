use ironcalc::base::Model;
use ratatui::style::Color;

use crate::ui::AppState;

use super::{Address, Book, Viewport, ViewportState};

#[test]
fn test_viewport_get_visible_columns() {
    let mut state = ViewportState::default();
    let book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York").expect("Failed to make model"),
    );
    let default_size = book.get_col_size(1).expect("Failed to get column size");
    let width = dbg!(dbg!(default_size) * 12 / 2);
    let app_state = AppState::default();
    let viewport = Viewport::new(&book, Some(&app_state.range_select))
        .with_selected(Address { sheet: 0, row: 1, col: 17 });
    let cols = viewport
        .get_visible_columns((width + 5) as u16, &mut state)
        .expect("Failed to get visible columns");
    assert_eq!(5, cols.len());
    assert_eq!(17, cols.last().expect("Failed to get last column").idx);
}

#[test]
fn test_viewport_get_visible_rows() {
    let mut state = dbg!(ViewportState::default());
    let book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York").expect("Failed to make model"),
    );
    let height = 6;
    let app_state = AppState::default();
    let viewport = Viewport::new(&book, Some(&app_state.range_select))
        .with_selected(Address { sheet: 0, row: 17, col: 1 });
    let rows = dbg!(viewport.get_visible_rows(height as u16, &mut state));
    assert_eq!(height - 1, rows.len());
    assert_eq!(
        17 - (height - 2),
        *rows.first().expect("Failed to get first row")
    );
    assert_eq!(17, *rows.last().expect("Failed to get last row"));
}

#[test]
fn test_viewport_visible_columns_after_length_change() {
    let mut state = ViewportState::default();
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York").expect("Failed to make model"),
    );
    let default_size = book.get_col_size(1).expect("Failed to get column size");
    let width = dbg!(dbg!(default_size) * 12 / 2);
    {
        let app_state = AppState::default();
        let viewport = Viewport::new(&book, Some(&app_state.range_select))
            .with_selected(Address { sheet: 0, row: 1, col: 17 });
        let cols = viewport
            .get_visible_columns((width + 5) as u16, &mut state)
            .expect("Failed to get visible columns");
        assert_eq!(5, cols.len());
        assert_eq!(17, cols.last().expect("Failed to get last column").idx);
    }

    book.set_col_size(1, default_size * 6)
        .expect("Failed to set column size");
    {
        let app_state = AppState::default();
        let viewport = Viewport::new(&book, Some(&app_state.range_select))
            .with_selected(Address { sheet: 0, row: 1, col: 1 });
        let cols = viewport
            .get_visible_columns((width + 5) as u16, &mut state)
            .expect("Failed to get visible columns");
        assert_eq!(1, cols.len());
        assert_eq!(1, cols.last().expect("Failed to get last column").idx);
    }
}

#[test]
fn test_color_mapping() {
    for (s, c) in [
        ("red", Color::Red),
        ("blue", Color::Blue),
        ("green", Color::Green),
        ("magenta", Color::Magenta),
        ("cyan", Color::Cyan),
        ("white", Color::White),
        ("yellow", Color::Yellow),
        ("black", Color::Black),
        ("gray", Color::Gray),
        ("grey", Color::Gray),
        ("lightred", Color::LightRed),
        ("lightblue", Color::LightBlue),
        ("lightgreen", Color::LightGreen),
        ("lightmagenta", Color::LightMagenta),
        ("lightcyan", Color::LightCyan),
        ("lightyellow", Color::LightYellow),
        ("darkgrey", Color::DarkGray),
        ("darkgray", Color::DarkGray),
        ("#35f15b", Color::Rgb(53, 241, 91)),
    ]
    .map(|(s, c)| (Some(s.to_string()), c))
    {
        assert_eq!(super::viewport::map_color(s.as_ref(), Color::Gray), c);
    }
}
