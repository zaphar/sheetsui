use std::process::ExitCode;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::book;
use crate::ui::cmd::parse_color;
use crate::ui::{Address, Modality};

use super::cmd::{parse, Cmd};
use super::Workspace;

#[derive(Default)]
pub struct InputScript {
    events: Vec<Event>,
}

impl InputScript {
    pub fn char(self, c: char) -> Self {
        self.event(construct_key_event(KeyCode::Char(c)))
    }

    pub fn chars(self, cs: &str) -> Self {
        cs.chars().fold(self, |s, c| s.char(c))
    }

    pub fn ctrl(self, c: char) -> Self {
        self.modified_char(c, KeyModifiers::CONTROL)
    }

    pub fn alt(self, c: char) -> Self {
        self.modified_char(c, KeyModifiers::ALT)
    }

    pub fn tab(self) -> Self {
        self.event(construct_key_event(KeyCode::Tab))
    }

    pub fn enter(self) -> Self {
        self.event(construct_key_event(KeyCode::Enter))
    }

    pub fn modified_char(self, c: char, mods: KeyModifiers) -> Self {
        self.event(construct_modified_key_event(KeyCode::Char(c), mods))
    }

    pub fn event(mut self, evt: Event) -> Self {
        self.events.push(evt);
        self
    }

    pub fn esc(self) -> Self {
        self.event(construct_key_event(KeyCode::Esc))
    }

    pub fn run(self, ws: &mut Workspace) -> anyhow::Result<Option<ExitCode>> {
        for evt in self.events {
            if let Some(e) = ws.handle_input(evt)? {
                return Ok(Some(e));
            }
        }
        Ok(None)
    }
}

fn script() -> InputScript {
    InputScript::default()
}

fn construct_key_event(code: KeyCode) -> Event {
    construct_modified_key_event(code, KeyModifiers::empty())
}

fn construct_modified_key_event(code: KeyCode, mods: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, mods))
}

#[test]
fn test_write_cmd() {
    let input = "write foo.xlsx";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Write(Some("foo.xlsx")));
}

#[test]
fn test_short_write_cmd() {
    let input = "w foo.xlsx";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Write(Some("foo.xlsx")));
}

#[test]
fn test_insert_rows_cmd() {
    let input = "insert-rows 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertRows(1));
}

#[test]
fn test_insert_rows_cmd_short() {
    let input = "ir 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertRows(1));
}

#[test]
fn test_insert_cols_cmd() {
    let input = "insert-cols 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertColumns(1));
}

#[test]
fn test_insert_cols_cmd_short() {
    let input = "ic 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertColumns(1));
}

#[test]
fn test_edit_cmd() {
    let input = "edit path.txt";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Edit("path.txt"));
}

#[test]
fn test_edit_cmd_short() {
    let input = "e path.txt";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Edit("path.txt"));
}

#[test]
fn test_help_cmd() {
    let input = "help topic";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Help(Some("topic")));
}

#[test]
fn test_help_cmd_short() {
    let input = "? topic";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Help(Some("topic")));
}

#[test]
fn test_quit_cmd_short() {
    let input = "q";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Quit);
}

#[test]
fn test_quit_cmd() {
    let input = "quit";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Quit);
}

#[test]
fn test_cmd_new_sheet_with_name() {
    let input = "new-sheet test";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::NewSheet(Some("test")));
}

#[test]
fn test_cmd_new_sheet_no_name() {
    let input = "new-sheet";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::NewSheet(None));
}

#[test]
fn test_cmd_select_sheet_with_name() {
    let input = "select-sheet test";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::SelectSheet("test"));
}

#[test]
fn test_cmd_rename_sheet_with_name() {
    let input = "rename-sheet test";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::RenameSheet(None, "test"));
}

#[test]
fn test_cmd_rename_sheet_with_idx_and_name() {
    let input = "rename-sheet 0 test";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::RenameSheet(Some(0), "test"));
}

#[test]
fn test_cmd_color_rows_with_color() {
    let input = "color-rows red";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::ColorRows(None, parse_color("red").unwrap()));
}

#[test]
fn test_cmd_color_rows_with_idx_and_color() {
    let input = "color-rows 1 red";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::ColorRows(Some(1), parse_color("red").unwrap()));
}

#[test]
fn test_cmd_color_columns_with_color() {
    let input = "color-columns red";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::ColorColumns(None, parse_color("red").unwrap()));
}

#[test]
fn test_cmd_color_columns_with_idx_and_color() {
    let input = "color-columns 1 red";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::ColorColumns(Some(1), parse_color("red").unwrap()));
}

#[test]
fn test_input_navitation_enter_key() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .enter()
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(row + 1, ws.book.location.row);
}

#[test]
fn test_input_navitation_tab_key() {
    let mut ws = new_workspace();
    let col = ws.book.location.col;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .tab()
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(col + 1, ws.book.location.col);
}

#[test]
fn test_input_navitation_shift_enter_key() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .enter()
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(row + 1, ws.book.location.row);
    script()
        .event(construct_modified_key_event(
            KeyCode::Enter,
            KeyModifiers::SHIFT,
        ))
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(row, ws.book.location.row);
}

#[test]
fn test_input_navitation_shift_tab_key() {
    let mut ws = new_workspace();
    let col = ws.book.location.col;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .tab()
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(col + 1, ws.book.location.col);
    script()
        .event(construct_modified_key_event(
            KeyCode::Tab,
            KeyModifiers::SHIFT,
        ))
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(col, ws.book.location.col);
}

macro_rules! assert_help_dialog {
    ($exit : expr) => {{
        let mut ws = new_workspace();
        assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
        script()
            .char('i')
            .run(&mut ws)
            .expect("Failed to handle 'i' key");
        assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
        let edit_help = ws.render_help_text();
        script()
            .alt('h')
            .run(&mut ws)
            .expect("Failed to handle 'alt-h' key event");
        assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
        assert_eq!(edit_help, ws.state.popup);
        $exit.run(&mut ws).expect("Failed to handle key event");
        assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    }};
}

#[test]
fn test_edit_mode_help_keycode_esc() {
    assert_help_dialog!(script().esc());
}

#[test]
fn test_edit_mode_help_keycode_enter() {
    assert_help_dialog!(script().enter());
}

#[test]
fn test_edit_mode_help_keycode_q() {
    assert_help_dialog!(script().char('q'));
}

#[test]
fn test_edit_mode_help_keycode_alt_h() {
    assert_help_dialog!(script().alt('h'));
}

#[test]
fn test_navigation_mode_help_keycode() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    let help_text = ws.render_help_text();
    script()
        .alt('h')
        .run(&mut ws)
        .expect("Failed to handle 'alt-h' key event");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!(help_text, ws.state.popup);
}

#[test]
fn test_command_mode_help_keycode() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char(':')
        .run(&mut ws)
        .expect("Failed to handle ':' key");
    assert_eq!(Some(&Modality::Command), ws.state.modality_stack.last());
    let edit_help = ws.render_help_text();
    script()
        .alt('h')
        .run(&mut ws)
        .expect("Failed to handle 'alt-h' key event");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!(edit_help, ws.state.popup);
}

#[test]
fn test_edit_mode_esc_keycode() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('i')
        .run(&mut ws)
        .expect("Failed to handle 'i' key");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    script()
        .char('a')
        .esc()
        .run(&mut ws)
        .expect("Failed to handle key squence");
    assert_eq!(
        "",
        ws.book
            .get_current_cell_contents()
            .expect("Failed to get current cell contents")
    );
    assert_eq!("", ws.text_area.lines().join("\n"));
}

#[test]
fn test_navigation_numeric_prefix() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.book
        .new_sheet(Some("Sheet2"))
        .expect("failed to create sheet2");
    ws.book
        .new_sheet(Some("Sheet3"))
        .expect("failed to create sheet3");
    script()
        .char('2')
        .char('3')
        .char('9')
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!(239, ws.state.get_n_prefix());
}

#[test]
fn test_navigation_numeric_prefix_cancel() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.book
        .new_sheet(Some("Sheet2"))
        .expect("failed to create sheet2");
    ws.book
        .new_sheet(Some("Sheet3"))
        .expect("failed to create sheet3");
    script()
        .char('2')
        .char('3')
        .char('9')
        .esc()
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!(1, ws.state.get_n_prefix());
}

#[test]
fn test_navigation_tab_next_numeric_prefix() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.book
        .new_sheet(Some("Sheet2"))
        .expect("failed to create sheet2");
    ws.book
        .new_sheet(Some("Sheet3"))
        .expect("failed to create sheet3");
    script()
        .char('2')
        .run(&mut ws)
        .expect("Failed to handle '2' key event");
    assert_eq!(2, ws.state.get_n_prefix());
    script()
        .ctrl('n')
        .run(&mut ws)
        .expect("Failed to handle 'Ctrl-n' key event");
    assert_eq!(
        "Sheet3",
        ws.book.get_sheet_name().expect("Failed to get sheet name")
    );
    assert_eq!(1, ws.state.get_n_prefix());
    script()
        .ctrl('n')
        .run(&mut ws)
        .expect("Failed to handle 'Ctrl-n' key event");
    assert_eq!(
        "Sheet1",
        ws.book.get_sheet_name().expect("Failed to get sheet name")
    );
}

#[test]
fn test_range_copy() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());

    let address = Address::default();
    ws.book
        .move_to(&address)
        .expect("Failed to move to row");
    let original_loc = ws.book.location.clone();
    script()
        .ctrl('r')
        .run(&mut ws)
        .expect("Failed to handle 'Ctrl-r' key event");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(
        Some(original_loc.clone()),
        ws.state.range_select.original_location
    );
    assert!(ws.state.range_select.start.is_none());
    assert!(ws.state.range_select.end.is_none());

    script()
        .char('l')
        .char(' ')
        .run(&mut ws)
        .expect("Failed to handle key sequence");
    assert_eq!(
        Some(Address { sheet: 0, row: 1, col: 2 }),
        ws.state.range_select.start
    );

    script()
        .char('j')
        .char(' ')
        .run(&mut ws)
        .expect("Failed to handle key sequence");

    assert!(ws.state.range_select.original_location.is_none());
    assert_eq!(
        Some(Address { sheet: 0, row: 1, col: 2 }),
        ws.state.range_select.start
    );
    assert_eq!(Some(Address { sheet: 0, row: 2, col: 2 }), ws.state.range_select.end);
    assert_eq!(original_loc, ws.book.location);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());

    ws.book
        .move_to(&Address { sheet: 0, row: 5, col: 5 })
        .expect("Failed to move to row");
    let original_loc_2 = ws.book.location.clone();
    assert_eq!(Address { sheet: 0, row: 5, col: 5 }, original_loc_2);

    script()
        .char('v')
        .run(&mut ws)
        .expect("Failed to handle 'v' key event");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(
        Some(original_loc_2.clone()),
        ws.state.range_select.original_location
    );
    assert!(ws.state.range_select.start.is_some());
    assert!(ws.state.range_select.end.is_none());

    script()
        .char('h')
        .char(' ')
        .run(&mut ws)
        .expect("Failed to handle key sequence");
    assert_eq!(
        Some(Address { sheet: 0, row: 5, col: 5 }),
        ws.state.range_select.start
    );

    script()
        .char('k')
        .char(' ')
        .run(&mut ws)
        .expect("Failed to handle key sequence");

    assert!(ws.state.range_select.original_location.is_none());
    assert_eq!(
        Some(Address { sheet: 0, row: 5, col: 5 }),
        ws.state.range_select.start
    );
    assert_eq!(Some(Address { sheet: 0, row: 5, col: 4 }), ws.state.range_select.end);
    assert_eq!(Address { sheet: 0, row: 4, col: 5 }, ws.book.location);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_range_copy_mode_from_edit_mode() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('e')
        .run(&mut ws)
        .expect("Failed to handle 'e' key event");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    script()
        .ctrl('r')
        .run(&mut ws)
        .expect("Failed to handle 'Ctrl-r' key event");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
}

#[test]
fn test_gg_movement() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('j')
        .char('j')
        .run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(ws.book.location, Address { sheet: 0, row: 3, col: 1 });
    script()
        .char('l')
        .char('g')
        .char('g')
        .run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(ws.book.location, Address { sheet: 0, row: 1, col: 2 });
}

#[test]
fn test_h_j_k_l_movement() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('2')
        .char('j')
        .char('l')
        .run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(ws.book.location, Address { sheet: 0, row: 3, col: 2 });
    script()
        .char('h')
        .char('2')
        .char('k')
        .run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(ws.book.location, Address { sheet: 0, row: 1, col: 1 });
}

macro_rules! assert_copy_paste {
    ($c: expr, $p: expr, $source: expr,) => {
        assert_copy_paste!($c, $p, $source, $source)
    };
    ($c: expr, $p: expr, $source: expr) => {
        assert_copy_paste!($c, $p, $source, $source)
    };
    ($c: expr, $p: expr, $source: expr, $expected: expr,) => {
        assert_copy_paste!($c, $p, $source, $expected)
    };
    ($c: expr, $p: expr, $source: expr, $expected: expr) => {{
        let mut ws = new_workspace();
        script()
            .char('j')
            .char('l')
            .run(&mut ws)
            .expect("Failed to run script");
        ws.book
            .edit_current_cell($source)
            .expect("Failed to edit cell");
        ws.book.evaluate();
        script()
            .event($c)
            .char('l')
            .char('j')
            .event($p)
            .run(&mut ws)
            .expect("Failed to run script");
        let copy = ws
            .book
            .get_current_cell_contents()
            .expect("Failed to get cell contents");
        assert_eq!(copy, $expected);
    }};
}

#[test]
fn test_y_p_copy_paste() {
    assert_copy_paste!(
        construct_key_event(KeyCode::Char('y')),
        construct_key_event(KeyCode::Char('p')),
        "foo",
    );
}

#[test]
fn test_traditional_copy_paste() {
    assert_copy_paste!(
        construct_modified_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL),
        construct_modified_key_event(KeyCode::Char('v'), KeyModifiers::CONTROL),
        "foo",
    );
}

#[test]
fn test_y_p_copy_paste_rendered() {
    assert_copy_paste!(
        construct_key_event(KeyCode::Char('Y')),
        construct_key_event(KeyCode::Char('p')),
        "=1+2",
        "3",
    );
}

#[test]
fn test_traditional_copy_paste_rendered() {
    assert_copy_paste!(
        construct_modified_key_event(KeyCode::Char('C'), KeyModifiers::CONTROL),
        construct_modified_key_event(KeyCode::Char('v'), KeyModifiers::CONTROL),
        "=1+2",
        "3",
    );
}

#[test]
fn test_clear_cell() {
    let mut ws = new_workspace();
    ws.book
        .edit_current_cell("foo")
        .expect("failed to edit cell");
    ws.book.evaluate();
    assert_eq!(
        "foo",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
    );
    script()
        .char('d')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
    );
}

#[test]
fn test_clear_cell_all() {
    let mut ws = new_workspace();
    ws.book
        .edit_current_cell("foo")
        .expect("failed to edit cell");
    ws.book.evaluate();
    assert_eq!(
        "foo",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
    );
    script()
        .char('D')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
    );
}

#[test]
fn test_sheet_navigation() {
    let mut ws = new_workspace();
    ws.book
        .new_sheet(Some("sheet 2"))
        .expect("Failed to set sheet name");
    ws.book
        .new_sheet(Some("sheet 3"))
        .expect("Failed to set sheet name");
    ws.book
        .new_sheet(Some("sheet 4"))
        .expect("Failed to set sheet name");
    script()
        .ctrl('n')
        .ctrl('n')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "sheet 3",
        ws.book.get_sheet_name().expect("Failed to get sheet name")
    );
    script()
        .ctrl('p')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "sheet 2",
        ws.book.get_sheet_name().expect("Failed to get sheet name")
    );
}

#[test]
fn test_sheet_column_sizing() {
    let mut ws = new_workspace();
    script()
        .char('3')
        .ctrl('l')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        28,
        ws.book.get_col_size(1).expect("Failed to get column size")
    );
    script()
        .char('1')
        .ctrl('h')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        27,
        ws.book.get_col_size(1).expect("Failed to get column size")
    );
}

#[test]
fn test_quit() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    let result = script()
        .char('q')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert!(result.is_some());
}

#[test]
fn test_cell_replace() {
    let mut ws = new_workspace();
    ws.book
        .edit_current_cell("foo")
        .expect("Failed to edit current cell");
    assert_eq!(
        "foo",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
            .as_str()
    );
    script()
        .char('s')
        .char('b')
        .char('a')
        .char('r')
        .enter()
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "bar",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
            .as_str()
    );
}

macro_rules! assert_command_finish {
    ($script : expr) => {
        let mut ws = new_workspace();
        assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
        script()
            .char(':')
            .run(&mut ws)
            .expect("Failed to handle ':' key");
        assert_eq!(Some(&Modality::Command), ws.state.modality_stack.last());
        $script.run(&mut ws).expect("Failed to handle script");
        assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    };
}

#[test]
fn test_command_mode_esc() {
    assert_command_finish!(script().esc());
}

#[test]
fn test_command_mode_enter() {
    assert_command_finish!(script().enter());
}

#[test]
fn test_edit_mode_paste() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.state.range_select.start = Some(Address { sheet: 0, row: 1, col: 1 });
    ws.state.range_select.end = Some(Address { sheet: 0, row: 2, col: 2 });
    script()
        .char('e')
        .ctrl('p')
        .run(&mut ws)
        .expect("Failed to handle input script");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    assert_eq!(vec!["A1:B2".to_string()], ws.text_area.into_lines());
}

#[test]
fn test_range_select_esc() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('v')
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    script()
        .esc()
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('v')
        .chars("123")
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(3, ws.state.numeric_prefix.len());
    script()
        .esc()
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(0, ws.state.numeric_prefix.len());
    script()
        .esc()
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

macro_rules! assert_range_clear {
    ($script : expr) => {{
        let mut ws = new_workspace();
        assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
        let first_corner = Address { sheet: 0, row: 1, col: 1 };
        let second_corner = Address { sheet: 0, row: 2, col: 2 };
        ws.book
            .update_cell(&first_corner, "foo")
            .expect("Failed to update cell");
        ws.book
            .update_cell(&second_corner, "bar")
            .expect("Failed to update cell");
        assert_eq!(
            "foo".to_string(),
            ws.book
                .get_cell_addr_contents(&first_corner)
                .expect("failed to get cell contents")
        );
        assert_eq!(
            "bar".to_string(),
            ws.book
                .get_cell_addr_contents(&second_corner)
                .expect("failed to get cell contents")
        );
        script()
            .char('v')
            .run(&mut ws)
            .expect("Failed to handle script");
        assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
        $script.run(&mut ws).expect("Failed to handle script");
        assert_eq!(
            "".to_string(),
            ws.book
                .get_cell_addr_contents(&first_corner)
                .expect("failed to get cell contents")
        );
        assert_eq!(
            "".to_string(),
            ws.book
                .get_cell_addr_contents(&second_corner)
                .expect("failed to get cell contents")
        );
    }};
}

#[test]
fn test_range_select_clear_upper_d() {
    assert_range_clear!(script().char('j').char('l').char('D'));
}

#[test]
fn test_range_select_movement() {
    let mut ws = new_workspace();
    ws.book
        .new_sheet(Some("s2"))
        .expect("Unable create s2 sheet");
    ws.book
        .new_sheet(Some("s3"))
        .expect("Unable create s3 sheet");
    script()
        .ctrl('r')
        .run(&mut ws)
        .expect("failed to run script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    script()
        .char('3')
        .char('j')
        .char('3')
        .char('l')
        .char('1')
        .char('h')
        .char('1')
        .char('k')
        .run(&mut ws)
        .expect("failed to run script");
    assert_eq!(&Address { sheet: 0, row: 3, col: 3 }, &ws.book.location);
    script()
        .ctrl('n')
        .run(&mut ws)
        .expect("Unable to run script");
    assert_eq!(1, ws.book.location.sheet);
    script()
        .ctrl('p')
        .run(&mut ws)
        .expect("Unable to run script");
    assert_eq!(0, ws.book.location.sheet);
}

#[test]
fn test_range_select_clear_lower_d() {
    assert_range_clear!(script().char('j').char('l').char('d'));
}

macro_rules! assert_range_copy {
    ($script: expr) => {{
        let mut ws = new_workspace();
        let top_left_addr = Address { sheet: 0, row: 2, col: 2 };
        let bot_right_addr = Address { sheet: 0, row: 4, col: 4 };
        ws.book
            .update_cell(&top_left_addr, "top_left")
            .expect("Failed to update top left");
        ws.book
            .update_cell(&bot_right_addr, "bot_right")
            .expect("Failed to update top left");
        assert!(ws.state.clipboard.is_none());
        script()
            .ctrl('r')
            .char('j')
            .char('l')
            .char(' ')
            .run(&mut ws)
            .expect("failed to run script");
        assert_eq!(
            &top_left_addr,
            ws.state
                .range_select
                .start
                .as_ref()
                .expect("Didn't find a start of range")
        );
        script()
            .char('2')
            .char('j')
            .char('2')
            .char('l')
            .run(&mut ws)
            .expect("failed to run script");
        assert_eq!(
            &bot_right_addr,
            ws.state
                .range_select
                .end
                .as_ref()
                .expect("Didn't find a start of range")
        );
        assert_eq!(
            &Address { sheet: 0, row: 1, col: 1 },
            ws.state
                .range_select
                .original_location
                .as_ref()
                .expect("Expected an original location")
        );
        assert_eq!(
            Some(&Modality::RangeSelect),
            ws.state.modality_stack.iter().last()
        );
        $script.run(&mut ws).expect("failed to run script");
        assert!(ws.state.clipboard.is_some());
        match ws.state.clipboard.unwrap() {
            crate::ui::ClipboardContents::Cell(_) => assert!(false, "Not rows in Clipboard"),
            crate::ui::ClipboardContents::Range(rows) => {
                assert_eq!(
                    vec![
                        vec!["top_left".to_string(), "".to_string(), "".to_string()],
                        vec!["".to_string(), "".to_string(), "".to_string()],
                        vec!["".to_string(), "".to_string(), "bot_right".to_string()],
                    ],
                    rows
                );
            }
        }
        assert_eq!(
            Some(&Modality::Navigate),
            ws.state.modality_stack.iter().last()
        );
    }};
}

#[test]
fn test_range_select_copy_c() {
    assert_range_copy!(script().ctrl('c'));
}

#[test]
fn test_range_select_copy_y() {
    assert_range_copy!(script().char('y'));
}

#[test]
fn test_range_select_copy_capital_y() {
    assert_range_copy!(script().char('Y'));
}

#[test]
fn test_range_select_copy_capital_c() {
    assert_range_copy!(script().ctrl('C'));
}

#[test]
fn test_extend_to_range() {
    let mut ws = new_workspace();
    ws.book
        .edit_current_cell("=B1+1")
        .expect("Failed to edit cell");
    ws.book.evaluate();
    script()
        .char('v')
        .char('j')
        .char('x')
        .run(&mut ws)
        .expect("Unable to run script");
    let extended_cell = ws
        .book
        .get_cell_addr_contents(&Address { sheet: 0, row: 2, col: 1 })
        .expect("Failed to get cell contents");
    assert_eq!("=B2+1".to_string(), extended_cell);
}

#[test]
fn test_color_cells() {
    let mut ws = new_workspace();
    script()
        .char('v')
        .chars("jjll")
        .char(':')
        .chars("color-cell red")
        .enter()
        .run(&mut ws)
        .expect("Unable to run script");
    for ri in 1..=3 {
        for ci in 1..=3 {
            let style = ws
                .book
                .get_cell_style(&Address { sheet: ws.book.location.sheet, row: ri, col: ci })
                .expect("failed to get style");
            assert_eq!(
                "#800000",
                style
                    .fill
                    .bg_color
                    .expect(&format!("No background color set for {}:{}", ri, ci))
                    .as_str()
            );
        }
    }
}

#[test]
fn test_color_row() {
    let mut ws = new_workspace();
    script()
        .char(':')
        .chars("color-rows red")
        .enter()
        .run(&mut ws)
        .expect("Unable to run script");
    for ci in [1, book::LAST_COLUMN] {
        let style = ws
            .book
            .get_cell_style(&Address { sheet: ws.book.location.sheet, row: 1, col: ci as usize })
            .expect("failed to get style");
        assert_eq!(
            "#800000",
            style
                .fill
                .bg_color
                .expect(&format!("No background color set for {}:{}", 1, ci))
                .as_str()
        );
    }
}

#[test]
fn test_color_col() {
    let mut ws = new_workspace();
    script()
        .char(':')
        .chars("color-columns red")
        .enter()
        .run(&mut ws)
        .expect("Unable to run script");
    for ri in [1, book::LAST_ROW] {
        let style = ws
            .book
            .get_cell_style(&Address { sheet: ws.book.location.sheet, row: ri as usize, col: 1 })
            .expect("failed to get style");
        assert_eq!(
            "#800000",
            style
                .fill
                .bg_color
                .expect(&format!("No background color set for {}:{}", ri, 1))
                .as_str()
        );
    }
}

#[test]
fn test_bold_text() {
    let mut ws = new_workspace();
    let before_style = ws
        .book
        .get_cell_style(&Address { sheet: 0, row: 1, col: 1 })
        .expect("Failed to get style");
    assert!(!before_style.font.b);
    script()
        .char('B')
        .run(&mut ws)
        .expect("Unable to run script");
    let style = ws
        .book
        .get_cell_style(&Address { sheet: 0, row: 1, col: 1 })
        .expect("Failed to get style");
    assert!(style.font.b);
    script()
        .char('B')
        .run(&mut ws)
        .expect("Unable to run script");
    assert!(!before_style.font.b);
}

#[test]
fn test_italic_text() {
    let mut ws = new_workspace();
    let before_style = ws
        .book
        .get_cell_style(&Address { sheet: 0, row: 1, col: 1 })
        .expect("Failed to get style");
    assert!(!before_style.font.i);
    script()
        .char('I')
        .run(&mut ws)
        .expect("Unable to run script");
    let style = ws
        .book
        .get_cell_style(&Address { sheet: 0, row: 1, col: 1 })
        .expect("Failed to get style");
    assert!(style.font.i);
    script()
        .char('I')
        .run(&mut ws)
        .expect("Unable to run script");
    assert!(!before_style.font.i);
}

fn new_workspace<'a>() -> Workspace<'a> {
    Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook")
}
