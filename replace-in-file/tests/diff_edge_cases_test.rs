use replace_in_file::construct_new_file_content_v2;

// Ported from refer/diff_edge_cases.test.ts

#[test]
fn test_search_prefix_symbols_less_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----- SEARCH
content
=======
new content
++++++ REPLACE";
    let expected = "before\nnew content\nafter";

    let result = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_search_prefix_symbols_more_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----------- SEARCH
content
=======
new content
++++++ REPLACE";
    let expected = "before\nnew content\nafter";

    let result = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_search_less_than_7_and_replace_less_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----- SEARCH
content
=====
new content
++++++ REPLACE";
    let expected = "before\nnew content\nafter";

    let result = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_search_less_than_7_and_replace_more_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----- SEARCH
content
========
new content
++++++ REPLACE";
    let expected = "before\nnew content\nafter";

    let result = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_search_more_than_7_and_replace_more_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----------- SEARCH
content
==========
new content
++++++ REPLACE";
    let expected = "before\nnew content\nafter";

    let result = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_search_more_than_7_and_replace_less_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----------- SEARCH
content
=====
new content
++++++ REPLACE";
    let expected = "before\nnew content\nafter";

    let result = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_consecutive_search_replace_second_search_less_than_7() {
    let is_final = true;
    let original = "before\nfirst content\nafter\nsecond content\nend";
    let diff = "------- SEARCH
first content
=======
first new content
++++++ REPLACE
----- SEARCH
second content
=======
second new content
++++++ REPLACE";
    let expected = "before\nfirst new content\nafter\nsecond new content\nend";

    let result = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_consecutive_second_block_less_than_7_and_equals_less_than_7() {
    let is_final = true;
    let original = "before\nfirst content\nafter\nsecond content\nend";
    let diff = "------- SEARCH
first content
=======
first new content
++++++ REPLACE
----- SEARCH
second content
=====
second new content
++++++ REPLACE";
    let expected = "before\nfirst new content\nafter\nsecond new content\nend";

    let result = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(result, expected);
}