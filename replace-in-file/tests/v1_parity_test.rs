use replace_in_file::{construct_new_file_content_v1, construct_new_file_content_v2};

// Parity tests: v1 should match TS v1 behavior and, for supported cases, equal v2

#[test]
fn v1_empty_file() {
    let original = "";
    let diff = "------- SEARCH\n=======\nnew content\n+++++++ REPLACE";
    let expected = "new content\n";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r1, r2);
}

#[test]
fn v1_malformed_search_mixed_symbols() {
    let original = "line1\nline2\nline3";
    let diff = "<<-- SEARCH\nline2\n=======\nreplaced\n+++++++ REPLACE";
    assert!(construct_new_file_content_v1(diff, original, true).is_err());
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn v1_malformed_search_insufficient_dashes() {
    let original = "line1\nline2\nline3";
    let diff = "-- SEARCH\nline2\n=======\nreplaced\n+++++++ REPLACE";
    assert!(construct_new_file_content_v1(diff, original, true).is_err());
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn v1_malformed_search_missing_space() {
    let original = "line1\nline2\nline3";
    let diff = "-------SEARCH\nline2\n=======\nreplaced\n+++++++ REPLACE";
    assert!(construct_new_file_content_v1(diff, original, true).is_err());
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn v1_exact_match_replacement() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH\nline2\n=======\nreplaced\n+++++++ REPLACE";
    let expected = "line1\nreplaced\nline3";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r1, r2);
}

#[test]
fn v1_line_trimmed_match_replacement() {
    let original = "line1\n line2 \nline3";
    let diff = "------- SEARCH\nline2\n=======\nreplaced\n+++++++ REPLACE";
    let expected = "line1\nreplaced\nline3";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r1, r2);
}

#[test]
fn v1_block_anchor_match_replacement() {
    let original = "line1\nstart\nmiddle\nend\nline5";
    let diff = "------- SEARCH\nstart\nmiddle\nend\n=======\nreplaced\n+++++++ REPLACE";
    let expected = "line1\nreplaced\nline5";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r1, r2);
}

#[test]
fn v1_incremental_processing_like() {
    let original = "line1\nline2\nline3";
    let diff = [
        "------- SEARCH\nline2\n=======",
        "replaced\n",
        "+++++++ REPLACE",
    ]
    .join("\n");
    let expected = "line1\nreplaced\n\nline3";
    let r1 = construct_new_file_content_v1(&diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(&diff, original, true).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r1, r2);
}

#[test]
fn v1_multiple_ordered_replacements() {
    let original = "First\nSecond\nThird\nFourth";
    let diff = "------- SEARCH\nFirst\n=======\n1st\n+++++++ REPLACE\n\n------- SEARCH\nThird\n=======\n3rd\n+++++++ REPLACE";
    let expected = "1st\nSecond\n3rd\nFourth";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r1, r2);
}

#[test]
fn v1_replace_then_delete() {
    let original = "line1\nline2\nline3\nline4";
    let diff = "------- SEARCH\nline2\n=======\nreplaced\n+++++++ REPLACE\n\n------- SEARCH\nline4\n=======\n+++++++ REPLACE";
    let expected = "line1\nreplaced\nline3\n";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r1, r2);
}

#[test]
fn v1_delete_then_replace() {
    let original = "line1\nline2\nline3\nline4";
    let diff = "------- SEARCH\nline1\n=======\n+++++++ REPLACE\n\n------- SEARCH\nline3\n=======\nreplaced\n+++++++ REPLACE";
    let expected = "line2\nreplaced\nline4";
    let r1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let r2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(r1, expected);
    assert_eq!(r1, r2);
}

#[test]
fn v1_malformed_diff_missing_separator() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH\nline2\n+++++++ REPLACE\nreplaced";
    assert!(construct_new_file_content_v1(diff, original, true).is_err());
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn v1_malformed_diff_trailing_space_on_separator() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH\nline2\n======= \nreplaced\n+++++++ REPLACE";
    assert!(construct_new_file_content_v1(diff, original, true).is_err());
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn v1_malformed_diff_double_replace_markers() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH\nline2\n+++++++ REPLACE\nfirst replacement\n+++++++ REPLACE";
    assert!(construct_new_file_content_v1(diff, original, true).is_err());
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn v1_malformed_diff_malformed_separator_with_dashes() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH\nline2\n------- =======\nreplaced\n+++++++ REPLACE";
    assert!(construct_new_file_content_v1(diff, original, true).is_err());
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn v1_no_match_found() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH\nnon-existent\n=======\nreplaced\n+++++++ REPLACE";
    assert!(construct_new_file_content_v1(diff, original, true).is_err());
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}