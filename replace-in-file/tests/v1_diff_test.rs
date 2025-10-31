use replace_in_file::{construct_new_file_content_v1, construct_new_file_content_v2};

#[test]
fn v1_out_of_order_replacements_different_positions() {
    let is_final = true;
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
    let result_v1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let expected = "first\nnew second\nthird\nnew fourth\n";
    assert_eq!(result_v1, expected);

    // v2 is strict and should error for out-of-order
    assert!(construct_new_file_content_v2(diff, original, is_final).is_err());
}

#[test]
fn v1_multiple_out_of_order_replacements() {
    let is_final = true;
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
    let result_v1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let expected = "one\nsecond\nthree\nfourth\nfifth\n";
    assert_eq!(result_v1, expected);

    assert!(construct_new_file_content_v2(diff, original, is_final).is_err());
}

#[test]
fn v1_out_of_order_with_indentation() {
    let is_final = true;
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
    let result_v1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let expected = "function test() {\n\tconst a = 10;\n\tconst b = 2;\n\tconst c = 30;\n\n}";
    assert_eq!(result_v1, expected);

    assert!(construct_new_file_content_v2(diff, original, is_final).is_err());
}

#[test]
fn v1_out_of_order_with_empty_lines() {
    let is_final = true;
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
    let result_v1 = construct_new_file_content_v1(diff, original, is_final).unwrap();
    let expected = "header\nnew body content\nnew footer\n";
    assert_eq!(result_v1, expected);

    assert!(construct_new_file_content_v2(diff, original, is_final).is_err());
}

#[test]
fn v1_missing_final_replace_marker_when_final_true() {
    let original = "line1\nline2\nline3";
    let diff = "------- SEARCH
line2
=======
replaced"; // missing REPLACE end
    let result_v1 = construct_new_file_content_v1(diff, original, true).unwrap();
    let expected = "line1\nreplaced\nline3";
    assert_eq!(result_v1, expected);

    // v2 also finalizes in strict mode
    let result_v2 = construct_new_file_content_v2(diff, original, true).unwrap();
    assert_eq!(result_v2, expected);
}