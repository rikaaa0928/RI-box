use replace_in_file::{construct_new_file_content_v1, construct_new_file_content_v2};

#[test]
fn v1_final_chunk_with_remaining_content() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH\nline2\n=======\nreplaced\n+++++++ REPLACE";
    let expected = "line1\nreplaced\nline3";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r1, r2);
}

#[test]
fn v1_missing_final_replace_marker_multiple_lines_is_final_true() {
    let original = "function test() {\n\tconst a = 1;\n\treturn a;\n}";
    let diff = "------- SEARCH\n\tconst a = 1;\n\treturn a;\n=======\n\tconst a = 42;\n\tconsole.log('updated');\n\treturn a;";
    // Note: missing +++++++ REPLACE marker
    let expected = "function test() {\n\tconst a = 42;\n\tconsole.log('updated');\n\treturn a;\n}";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    assert_eq!(r1, expected);
}

// Edge prefix/separator flexibility parity with TS

#[test]
fn v1_search_prefix_less_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----- SEARCH\ncontent\n=======\nnew content\n+++++++ REPLACE";
    let expected = "before\nnew content\nafter";
    let r1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r2, expected);
}

#[test]
fn v1_search_prefix_more_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----------- SEARCH\ncontent\n=======\nnew content\n+++++++ REPLACE";
    let expected = "before\nnew content\nafter";
    let r1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r2, expected);
}

#[test]
fn v1_search_less_than_7_and_separator_less_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----- SEARCH\ncontent\n=====\nnew content\n+++++++ REPLACE";
    let expected = "before\nnew content\nafter";
    let r1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r2, expected);
}

#[test]
fn v1_search_less_than_7_and_separator_more_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----- SEARCH\ncontent\n========\nnew content\n+++++++ REPLACE";
    let expected = "before\nnew content\nafter";
    let r1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r2, expected);
}

#[test]
fn v1_search_more_than_7_and_separator_more_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----------- SEARCH\ncontent\n==========\nnew content\n+++++++ REPLACE";
    let expected = "before\nnew content\nafter";
    let r1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r2, expected);
}

#[test]
fn v1_search_more_than_7_and_separator_less_than_7() {
    let is_final = true;
    let original = "before\ncontent\nafter";
    let diff = "----------- SEARCH\ncontent\n=====\nnew content\n+++++++ REPLACE";
    let expected = "before\nnew content\nafter";
    let r1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, is_final).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r2, expected);
}

// Selected edge_cases2 scenarios that v1/v2 both support or differ clearly

#[test]
fn v1_missing_search_block_v1_success_v2_error() {
    let original = "line1\nline2";
    let diff = "=======\nnew content\n+++++++ REPLACE";
    // v1: treats empty SEARCH as whole-file insert at start when is_final
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    assert_eq!(r1, "new content\n");
    // v2: stricter, should error
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn v1_empty_search_block_inserts_content_parity() {
    let original = "any content";
    let diff = "------- SEARCH\n=======\ninserted\n+++++++ REPLACE";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, "inserted\n");
    assert_eq!(r1, r2);
}

#[test]
fn v1_mixed_line_endings_normalization_parity() {
    let original = "line1\r\nline2";
    let diff = "------- SEARCH\nline1\r\n=======\nline1\n+++++++ REPLACE";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, "line1\nline2");
    assert_eq!(r1, r2);
}

#[test]
fn v1_special_characters_in_search_parity() {
    let original = "text with $^.*\nend";
    let diff = "------- SEARCH\n$^.*\n=======\nreplaced\n+++++++ REPLACE";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, "text with replaced\nend");
    assert_eq!(r1, r2);
}