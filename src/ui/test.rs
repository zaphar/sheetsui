use super::cmd::{parse, Cmd};

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
fn test_insert_row_cmd() {
    let input = "insert-row 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertRow(1));
}

#[test]
fn test_insert_row_cmd_short() {
    let input = "ir 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertRow(1));
}
