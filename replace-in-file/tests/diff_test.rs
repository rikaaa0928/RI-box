use replace_in_file::construct_new_file_content_v2;

// Ported from refer/diff.test.ts

#[test]
fn test_empty_file() {
    let original = "";
    let diff = "------- SEARCH
=======
new content
++++++ REPLACE";
    let expected = "new content\n";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_malformed_search_mixed_symbols() {
    let original = "line1\nline2\nline3";
    let diff = "<<-- SEARCH
line2
=======
replaced
++++++ REPLACE";

    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_malformed_search_insufficient_dashes() {
    let original = "line1\nline2\nline3";
    let diff = "-- SEARCH
line2
=======
replaced
++++++ REPLACE";

    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_malformed_search_missing_space() {
    let original = "line1\nline2\nline3";
    let diff = "-------SEARCH
line2
=======
replaced
++++++ REPLACE";

    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_exact_match_replacement() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
line2
=======
replaced
++++++ REPLACE";
    let expected = "line1\nreplaced\nline3";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_line_trimmed_match_replacement() {
    let original = "line1\n line2 \nline3";
    let diff = "------- SEARCH
line2
=======
replaced
++++++ REPLACE";
    let expected = "line1\nreplaced\nline3";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_block_anchor_match_replacement() {
    let original = "line1\nstart\nmiddle\nend\nline5";
    let diff = "------- SEARCH
start
middle
end
=======
replaced
++++++ REPLACE";
    let expected = "line1\nreplaced\nline5";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_incremental_processing() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
line2
=======
replaced

++++++ REPLACE";
    let expected = "line1\nreplaced\n\nline3";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_final_chunk_with_remaining_content() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
line2
=======
replaced
++++++ REPLACE";
    let expected = "line1\nreplaced\nline3";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_multiple_ordered_replacements() {
    let original = "First\nSecond\nThird\nFourth";
    let diff = "------- SEARCH
First
=======
1st
++++++ REPLACE

------- SEARCH
Third
=======
3rd
++++++ REPLACE";
    let expected = "1st\nSecond\n3rd\nFourth";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_replace_then_delete() {
    let original = "line1\nline2\nline3\nline4";
    let diff = "------- SEARCH
line2
=======
replaced
++++++ REPLACE

------- SEARCH
line4
=======
++++++ REPLACE";
    let expected = "line1\nreplaced\nline3\n";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_delete_then_replace() {
    let original = "line1\nline2\nline3\nline4";
    let diff = "------- SEARCH
line1
=======
++++++ REPLACE

------- SEARCH
line3
=======
replaced
++++++ REPLACE";
    let expected = "line2\nreplaced\nline4";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_malformed_diff_missing_separator() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
line2
++++++ REPLACE
replaced";

    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_malformed_diff_trailing_space_on_separator() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
line2
======= 
replaced
++++++ REPLACE";

    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_malformed_diff_double_replace_markers() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
line2
++++++ REPLACE
first replacement
++++++ REPLACE";

    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_malformed_diff_malformed_separator_with_dashes() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
line2
------- =======
replaced
++++++ REPLACE";

    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_no_match_found() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
non-existent
=======
replaced
++++++ REPLACE";

    assert!(construct_new_file_content_v2(diff, original, true).is_err());
}

#[test]
fn test_missing_final_replace_marker_when_final() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
line2
=======
replaced";
    let expected = "line1\nreplaced\nline3";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_missing_final_replace_marker_with_multiple_lines() {
    let original = "function test() {\n\tconst a = 1;\n\treturn a;\n}";
    let diff = "------- SEARCH
\tconst a = 1;
\treturn a;
=======
\tconst a = 42;
\tconsole.log('updated');
\treturn a;";
    let expected = "function test() {\n\tconst a = 42;\n\tconsole.log('updated');\n\treturn a;\n}";

    let result = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result, expected);
}

// Out-of-order replacements (TS v1 expects success; Rust v2 is strict → errors)
#[test]
fn test_out_of_order_replacements_different_positions() {
    let original = "first\nsecond\nthird\nfourth\n";
    let diff = "------- SEARCH
fourth
=======
new fourth
++++++ REPLACE
------- SEARCH
second
=======
new second
++++++ REPLACE";

    let result = construct_new_file_content_v2(diff, original, true);
    assert!(result.is_err());
}

#[test]
fn test_out_of_order_multiple() {
    let original = "one\ntwo\nthree\nfour\nfive\n";
    let diff = "------- SEARCH
four
=======
fourth
++++++ REPLACE
------- SEARCH
two
=======
second
++++++ REPLACE
------- SEARCH
five
=======
fifth
++++++ REPLACE";

    let result = construct_new_file_content_v2(diff, original, true);
    assert!(result.is_err());
}

#[test]
fn test_out_of_order_with_indentation() {
    let original = "function test() {\n\tconst a = 1;\n\tconst b = 2;\n\tconst c = 3;\n\n}";
    let diff = "------- SEARCH
\tconst c = 3;
=======
\tconst c = 30;
++++++ REPLACE
------- SEARCH
\tconst a = 1;
=======
\tconst a = 10;
++++++ REPLACE";

    let result = construct_new_file_content_v2(diff, original, true);
    assert!(result.is_err());
}

#[test]
fn test_out_of_order_with_empty_lines() {
    let original = "header\n\nbody\n\nfooter\n";
    let diff = "------- SEARCH
footer
=======
new footer
++++++ REPLACE
------- SEARCH

body

=======
new body content
++++++ REPLACE";

    let result = construct_new_file_content_v2(diff, original, true);
    assert!(result.is_err());
}