use super::*;

#[test]
fn test_dimensions_calculation() {
    let mut tbl = Tbl::new();
    tbl.update_entry(Address::new(0, 0), CellValue::Text(String::new())).unwrap();
    assert_eq!((1, 1), tbl.dimensions());
    tbl.update_entry(Address::new(0, 10), CellValue::Text(String::new())).unwrap();
    assert_eq!((1, 11), tbl.dimensions());
    tbl.update_entry(Address::new(20, 5), CellValue::Text(String::new())).unwrap();
    assert_eq!((21, 11), tbl.dimensions());
}

