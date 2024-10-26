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

#[test]

fn test_address_parse() {
    if let Some((a1, iter)) = formula::try_parse_addr("A1:A2 foo bar".into()) {
        assert_eq!("A1:A2", a1.to_string());
        assert_eq!(&iter[0..], " foo bar");
    }
}
