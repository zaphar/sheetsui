use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::ui::Modality;

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
    assert_eq!(cmd, Cmd::InsertRow(1));
}

#[test]
fn test_insert_rows_cmd_short() {
    let input = "ir 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertRow(1));
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

fn construct_key_event(code: KeyCode) -> Event {
    construct_modified_key_event(code, KeyModifiers::empty())
}

fn construct_modified_key_event(code: KeyCode, mods: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, mods))
}

// TODO(zaphar): Interaction testing for input.
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
    ws.handle_input(construct_key_event(KeyCode::Char('2')))
        .expect("Failed to handle '3' key event");
    ws.handle_input(construct_key_event(KeyCode::Char('3')))
        .expect("Failed to handle '3' key event");
    ws.handle_input(construct_key_event(KeyCode::Char('9')))
        .expect("Failed to handle '3' key event");
    assert_eq!(239, ws.state.get_n_prefix());
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
