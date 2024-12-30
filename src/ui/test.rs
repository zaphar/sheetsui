use std::process::ExitCode;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::ui::{Address, Modality};

use super::cmd::{parse, Cmd};
use super::Workspace;

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

#[derive(Default)]
pub struct InputScript{
    events: Vec<Event>
}

impl InputScript {
    pub fn char(self, c: char) -> Self {
        self.event(construct_key_event(KeyCode::Char(c)))
    }
    
    pub fn ctrl(self, c: char) -> Self {
        self.modified_char(c, KeyModifiers::CONTROL)
    }
    
    pub fn modified_char(self, c: char, mods: KeyModifiers) -> Self {
        self.event(construct_modified_key_event(KeyCode::Char(c), mods))
    }
    
    pub fn event(mut self, evt: Event) -> Self {
        self.events.push(evt);
        self
    }

    pub fn enter(self) -> Self {
        self.event(construct_key_event(KeyCode::Enter))
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

fn construct_key_event(code: KeyCode) -> Event {
    construct_modified_key_event(code, KeyModifiers::empty())
}

fn construct_modified_key_event(code: KeyCode, mods: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, mods))
}

#[test]
fn test_input_navitation_enter_key() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    let row = ws.book.location.row;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.handle_input(construct_key_event(KeyCode::Enter))
        .expect("Failed to handle enter key");
    assert_eq!(row + 1, ws.book.location.row);
}

#[test]
fn test_input_navitation_tab_key() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    let col = dbg!(ws.book.location.col);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.handle_input(construct_key_event(KeyCode::Tab))
        .expect("Failed to handle enter key");
    assert_eq!(col + 1, ws.book.location.col);
}

#[test]
fn test_input_navitation_shift_enter_key() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    let row = ws.book.location.row;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.handle_input(construct_key_event(KeyCode::Enter))
        .expect("Failed to handle enter key");
    assert_eq!(row + 1, ws.book.location.row);
    ws.handle_input(construct_modified_key_event(
        KeyCode::Enter,
        KeyModifiers::SHIFT,
    ))
    .expect("Failed to handle enter key");
    assert_eq!(row, ws.book.location.row);
}

#[test]
fn test_input_navitation_shift_tab_key() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    let col = dbg!(ws.book.location.col);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.handle_input(construct_key_event(KeyCode::Tab))
        .expect("Failed to handle enter key");
    assert_eq!(col + 1, ws.book.location.col);
    ws.handle_input(construct_modified_key_event(
        KeyCode::Tab,
        KeyModifiers::SHIFT,
    ))
    .expect("Failed to handle enter key");
    assert_eq!(col, ws.book.location.col);
}

#[test]
fn test_edit_mode_help_keycode() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.handle_input(construct_key_event(KeyCode::Char('i')))
        .expect("Failed to handle 'i' key");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    let edit_help = ws.render_help_text();
    ws.handle_input(construct_modified_key_event(KeyCode::Char('h'), KeyModifiers::ALT))
        .expect("Failed to handle 'ctrl-?' key event");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!(edit_help, ws.state.popup);
}

#[test]
fn test_navigation_mode_help_keycode() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    let help_text = ws.render_help_text();
    ws.handle_input(construct_modified_key_event(KeyCode::Char('h'), KeyModifiers::ALT))
        .expect("Failed to handle 'ctrl-?' key event");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!(help_text, ws.state.popup);
}

#[test]
fn test_command_mode_help_keycode() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.handle_input(construct_key_event(KeyCode::Char(':')))
        .expect("Failed to handle ':' key");
    assert_eq!(Some(&Modality::Command), ws.state.modality_stack.last());
    let edit_help = ws.render_help_text();
    ws.handle_input(construct_modified_key_event(KeyCode::Char('h'), KeyModifiers::ALT))
        .expect("Failed to handle 'ctrl-?' key event");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!(edit_help, ws.state.popup);
}

#[test]
fn test_edit_mode_esc_keycode() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.handle_input(construct_key_event(KeyCode::Char('i')))
        .expect("Failed to handle 'i' key");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    ws.handle_input(construct_key_event(KeyCode::Char('a')))
        .expect("Failed to handle 'a' key event");
    ws.handle_input(construct_key_event(KeyCode::Esc))
        .expect("Failed to handle 'esc' key event");
    assert_eq!("", ws.book.get_current_cell_contents().expect("Failed to get current cell contents"));
    assert_eq!("", ws.text_area.lines().join("\n"));
}

#[test]
fn test_navigation_numeric_prefix()
{
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.book.new_sheet(Some("Sheet2")).expect("failed to create sheet2");
    ws.book.new_sheet(Some("Sheet3")).expect("failed to create sheet3");
    InputScript::default()
        .char('2')
        .char('3')
        .char('9')
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!(239, ws.state.get_n_prefix());
}

#[test]
fn test_navigation_numeric_prefix_cancel()
{
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.book.new_sheet(Some("Sheet2")).expect("failed to create sheet2");
    ws.book.new_sheet(Some("Sheet3")).expect("failed to create sheet3");
    InputScript::default()
        .char('2')
        .char('3')
        .char('9')
        .esc()
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!(1, ws.state.get_n_prefix());
}

#[test]
fn test_navigation_tab_next_numeric_prefix()
{
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.book.new_sheet(Some("Sheet2")).expect("failed to create sheet2");
    ws.book.new_sheet(Some("Sheet3")).expect("failed to create sheet3");
    ws.handle_input(construct_key_event(KeyCode::Char('2')))
        .expect("Failed to handle '3' key event");
    assert_eq!(2, ws.state.get_n_prefix());
    ws.handle_input(construct_modified_key_event(KeyCode::Char('n'), KeyModifiers::CONTROL))
        .expect("Failed to handle 'Ctrl-n' key event");
    assert_eq!("Sheet3", ws.book.get_sheet_name().expect("Failed to get sheet name"));
    assert_eq!(1, ws.state.get_n_prefix());
    ws.handle_input(construct_modified_key_event(KeyCode::Char('n'), KeyModifiers::CONTROL))
        .expect("Failed to handle 'Ctrl-n' key event");
    assert_eq!("Sheet1", ws.book.get_sheet_name().expect("Failed to get sheet name"));
}

#[test]
fn test_range_copy() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    
    ws.book.move_to(&Address { row: 1, col: 1, }).expect("Failed to move to row");
    let original_loc = ws.book.location.clone();
    ws.handle_input(construct_modified_key_event(KeyCode::Char('r'), KeyModifiers::CONTROL))
        .expect("Failed to handle 'Ctrl-r' key event");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(Some(original_loc.clone()), ws.state.range_select.original_location);
    assert!(ws.state.range_select.start.is_none());
    assert!(ws.state.range_select.end.is_none());
    
    ws.handle_input(construct_key_event(KeyCode::Char('l')))
        .expect("Failed to handle 'l' key event");
    ws.handle_input(construct_key_event(KeyCode::Char(' ')))
        .expect("Failed to handle ' ' key event");
    assert_eq!(Some(Address {row:1, col:2, }), ws.state.range_select.start);
    
    ws.handle_input(construct_key_event(KeyCode::Char('j')))
        .expect("Failed to handle 'j' key event");
    ws.handle_input(construct_key_event(KeyCode::Char(' ')))
        .expect("Failed to handle ' ' key event");
    
    assert!(ws.state.range_select.original_location.is_none());
    assert_eq!(Some(Address {row:1, col:2, }), ws.state.range_select.start);
    assert_eq!(Some(Address {row:2, col:2, }), ws.state.range_select.end);
    assert_eq!(original_loc, ws.book.location);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
   
    ws.book.move_to(&Address { row: 5, col: 5, }).expect("Failed to move to row");
    let original_loc_2 = ws.book.location.clone();
    assert_eq!(Address { row: 5, col: 5 }, original_loc_2);
    
    InputScript::default().char('v').run(&mut ws)
        .expect("Failed to handle 'v' key event");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(Some(original_loc_2.clone()), ws.state.range_select.original_location);
    assert!(ws.state.range_select.start.is_some());
    assert!(ws.state.range_select.end.is_none());
    
    ws.handle_input(construct_key_event(KeyCode::Char('h')))
        .expect("Failed to handle 'h' key event");
    ws.handle_input(construct_key_event(KeyCode::Char(' ')))
        .expect("Failed to handle ' ' key event");
    assert_eq!(Some(Address {row:5, col: 5, }), ws.state.range_select.start);
    
    ws.handle_input(construct_key_event(KeyCode::Char('k')))
        .expect("Failed to handle 'k' key event");
    ws.handle_input(construct_key_event(KeyCode::Char(' ')))
        .expect("Failed to handle ' ' key event");
    
    assert!(ws.state.range_select.original_location.is_none());
    assert_eq!(Some(Address {row:5, col:5, }), ws.state.range_select.start);
    assert_eq!(Some(Address {row:5, col:4, }), ws.state.range_select.end);
    assert_eq!(Address {row:4, col:5, }, ws.book.location);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_range_copy_mode_from_edit_mode() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.handle_input(construct_key_event(KeyCode::Char('e')))
        .expect("Failed to handle 'e' key event");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    ws.handle_input(construct_modified_key_event(KeyCode::Char('r'), KeyModifiers::CONTROL))
        .expect("Failed to handle 'Ctrl-r' key event");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
}

#[test]
fn test_gg_movement() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    InputScript::default()
        .char('j')
        .char('j').run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(ws.book.location, Address { row: 3, col: 1 });
    InputScript::default()
        .char('l')
        .char('g')
        .char('g')
        .run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(ws.book.location, Address { row: 1, col: 2 });
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
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    InputScript::default()
        .char('j')
        .char('l')
        .run(&mut ws)
        .expect("Failed to run script");
    ws.book.edit_current_cell($source).expect("Failed to edit cell");
    ws.book.evaluate();
    InputScript::default()
        .event($c)
        .char('l')
        .char('j')
        .event($p)
        .run(&mut ws)
        .expect("Failed to run script");
    let copy = ws.book.get_current_cell_contents()
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
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    ws.book.edit_current_cell("foo")
        .expect("failed to edit cell");
    ws.book.evaluate();
    assert_eq!("foo", ws.book.get_current_cell_contents().expect("failed to get cell contents"));
    InputScript::default()
        .char('d')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!("", ws.book.get_current_cell_contents().expect("failed to get cell contents"));
}

#[test]
fn test_clear_cell_all() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    ws.book.edit_current_cell("foo")
        .expect("failed to edit cell");
    ws.book.evaluate();
    assert_eq!("foo", ws.book.get_current_cell_contents().expect("failed to get cell contents"));
    InputScript::default()
        .char('D')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!("", ws.book.get_current_cell_contents().expect("failed to get cell contents"));
}

#[test]
fn test_sheet_navigation() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    ws.book.new_sheet(Some("sheet 2")).expect("Failed to set sheet name");
    ws.book.new_sheet(Some("sheet 3")).expect("Failed to set sheet name");
    ws.book.new_sheet(Some("sheet 4")).expect("Failed to set sheet name");
    InputScript::default()
        .ctrl('n')
        .ctrl('n')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!("sheet 3", ws.book.get_sheet_name().expect("Failed to get sheet name"));
    InputScript::default()
        .ctrl('p')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!("sheet 2", ws.book.get_sheet_name().expect("Failed to get sheet name"));
}

#[test]
fn test_sheet_column_sizing() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    InputScript::default()
        .char('3')
        .ctrl('l')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(28, ws.book.get_col_size(1).expect("Failed to get column size"));
    InputScript::default()
        .char('1')
        .ctrl('h')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(27, ws.book.get_col_size(1).expect("Failed to get column size"));
}

#[test]
fn test_quit() {
    let mut ws =
        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
    let result = InputScript::default()
        .char('q')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert!(result.is_some());
}
