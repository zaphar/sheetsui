use ironcalc::base::Model;

use super::{Address, Book, Viewport, ViewportState, COLNAMES};

#[test]
fn test_viewport_get_visible_columns() {
    let mut state = ViewportState::default();
    let book = Book::new(Model::new_empty("test", "en", "America/New_York").expect("Failed to make model"));
    let default_size = book.get_col_size(1).expect("Failed to get column size");
    let width = dbg!(dbg!(default_size) * 12 / 2);
    let viewport = Viewport::new(&book).with_selected(Address { row: 1, col: 17 });
    let cols = viewport.get_visible_columns((width + 5) as u16, &mut state).expect("Failed to get visible columns");
    assert_eq!(5, cols.len());
    assert_eq!(17, cols.last().expect("Failed to get last column").idx);
}
