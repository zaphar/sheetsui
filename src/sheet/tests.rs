use super::*;

#[test]
fn test_dimensions_calculation() {
    let mut tbl = Tbl::new();
    tbl.update_entry(Address::new(0, 0), Computable::Text(String::new()));
    assert_eq!((0, 0), tbl.dimensions());
    tbl.update_entry(Address::new(0, 10), Computable::Text(String::new()));
    assert_eq!((0, 10), tbl.dimensions());
    tbl.update_entry(Address::new(20, 5), Computable::Text(String::new()));
    assert_eq!((20, 10), tbl.dimensions());
}
