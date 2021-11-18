use super::*;

#[test]
fn test_traverse_document() {
    let html = include_str!("test-data/file.html");
    let result = traverse_document(html);
    assert_eq!(result, "<title > HUR </title> <h1 > overskrift </h1> <h2 > underoverskrift med <a href=link1> link </a> </h2> <p > tekst 1 </p> <p > tekst med <a href=link2> link </a> og tekst </p> <p id=10> tekst med <span id=20> tekst inde i <span id=30> tekst </span> </span> </p>");
}
