use replace_in_file::construct_new_file_content_v2;

// Ported (v2-adapted) from refer/diff_edge_cases2.test.ts

#[test]
fn test_missing_search_block_should_error() {
    let original = "line1\nline2";
    let diff = "=======
new content
++++++ REPLACE";
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_consecutive_empty_search_blocks_should_error_in_v2() {
    let original = "text";
    let diff = "------- SEARCH
=======
replaced
++++++ REPLACE
------- SEARCH
=======
another
++++++ REPLACE";
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_reverse_markers_order_should_error() {
    let original = "content";
    let diff = "+++++++ SEARCH
=======
invalid
------- REPLACE";
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_incomplete_block_structure_missing_separator_should_error() {
    let original = "valid text";
    let diff = "------- SEARCH
text
++++++ REPLACE";
    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_empty_search_block_replaces_entire_file() {
    let original = "any content";
    let diff = "------- SEARCH
=======
inserted
++++++ REPLACE";
    let expected = "inserted\n";
    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_mixed_line_endings() {
    let original = "line1\r\nline2";
    let diff = "------- SEARCH
line1\r
=======
line1
++++++ REPLACE";
    let expected = "line1\nline2";
    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_special_characters_in_search() {
    let original = "text with $^.*\nend";
    let diff = "------- SEARCH
$^.*
=======
replaced
++++++ REPLACE";
    let expected = "text with replaced\nend";
    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_special_regex_chars_and_nested_search_markers() {
    let original = "text with $^.*\n--- SEARCH\nend";
    let diff = "------- SEARCH
$^.*
=======
replaced
++++++ REPLACE

------- SEARCH
--- SEARCH
=======
before
++++++ REPLACE";
    // v2 does not support this nested marker scenario; expect an error
    let result = construct_new_file_content_v2(diff, original, true);
    assert!(result.is_err());
}

#[test]
fn test_invalid_search_marker_format_is_valid_in_v2() {
    // In v2, '--- SEARCH' is a valid start marker (3+ dashes)
    let original = "text with $^.*\n--- SEARCH\nend";
    let diff = "--- SEARCH
$^.*
=======
replaced
++++++ REPLACE

------- SEARCH
--- SEARCH
=======
before
++++++ REPLACE";
    // However, this specific sequence is not accepted by v2 state machine
    let result = construct_new_file_content_v2(diff, original, true);
    assert!(result.is_err());
}

#[test]
fn test_incomplete_search_marker_should_error_in_v2() {
    let original = "text with $^.*\n--- SEARCH\nend";
    let diff = "--- SEARCH
$^.*
=======
replaced
++++++ REPLACE

------ SEARCH
--- SEARCH
=======
before
++++++ REPLACE";
    // This configuration leads to incorrect match sequencing
    let result = construct_new_file_content_v2(diff, original, true);
    assert!(result.is_err());
}

#[test]
fn test_custom_nested_search_markers() {
    let original = "text with $^.*\n--- SEARCH2\nend";
    let diff = "--- SEARCH
$^.*
=======
replaced
++++++ REPLACE

------ SEARCH
--- SEARCH2
=======
before
++++++ REPLACE";
    let expected = "text with replaced\nbefore\nend";
    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_text_containing_nested_search_markers() {
    let original = "text with $^.*\ntext with --- SEARCH2\nend";
    let diff = "--- SEARCH
$^.*
=======
replaced
++++++ REPLACE

------ SEARCH
text with --- SEARCH2
=======
before
++++++ REPLACE";
    let expected = "text with replaced\nbefore\nend";
    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_missing_replacement_marker_lenient_mode() {
    let original = "text with $^.*\ntext with --- SEARCH2\nend";
    let diff = "--- SEARCH
$^.*
=======
replaced
++++++ REPLACE

------ SEARCH
text with --- SEARCH2
=======
before";
    // isFinal=false → allow incomplete block, return partial result
    let expected = "text with replaced\nbefore\n";
    let result = construct_new_file_content_v2(diff, original, false).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_missing_replacement_marker_strict_mode_finalizes_in_v2() {
    let original = "text with $^.*\ntext with --- SEARCH2\nend";
    let diff = "--- SEARCH
$^.*
=======
replaced
++++++ REPLACE

------ SEARCH
text with --- SEARCH2
=======
before";
    // isFinal=true → v2 finalizes the replacement and appends remaining content
    let expected = "text with replaced\nbefore\nend";
    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}