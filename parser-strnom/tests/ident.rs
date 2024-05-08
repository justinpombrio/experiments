use strnom::{alt_longest, parse, regex, string, Parser};

#[test]
fn test_ident() {
    let keyword = string("null").constant("NULL");
    let ident = regex("identifier", "[a-zA-Z_]+").unwrap().constant("ID");
    let word_owned = alt_longest("word", (keyword, ident));
    let word = word_owned.refn();

    assert_eq!(parse("<test>", "nullary", word), Ok("ID"));
    assert_eq!(parse("<test>", "null", word), Ok("NULL"));
    assert_eq!(parse("<test>", "foonull", word), Ok("ID"));
}
