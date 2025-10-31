use regex::Regex;
use std::sync::OnceLock;

use crate::DiffError;

const SEARCH_BLOCK_CHAR: &str = "-";
const REPLACE_BLOCK_CHAR: &str = "+";
const LEGACY_SEARCH_BLOCK_CHAR: &str = "<";
const LEGACY_REPLACE_BLOCK_CHAR: &str = ">";

fn search_block_start_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^[-]{3,} SEARCH>?$").unwrap())
}

fn legacy_search_block_start_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^[<]{3,} SEARCH>?$").unwrap())
}

fn search_block_end_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^[=]{3,}$").unwrap())
}

fn replace_block_end_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^[+]{3,} REPLACE>?$").unwrap())
}

fn legacy_replace_block_end_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^[>]{3,} REPLACE>?$").unwrap())
}

fn is_search_block_start(line: &str) -> bool {
    search_block_start_regex().is_match(line) || legacy_search_block_start_regex().is_match(line)
}

fn is_search_block_end(line: &str) -> bool {
    search_block_end_regex().is_match(line)
}

fn is_replace_block_end(line: &str) -> bool {
    replace_block_end_regex().is_match(line) || legacy_replace_block_end_regex().is_match(line)
}

fn line_trimmed_fallback_match(
    original_content: &str,
    search_content: &str,
    start_index: usize,
) -> Option<(usize, usize)> {
    let original_lines: Vec<&str> = original_content.split('\n').collect();
    let mut search_lines: Vec<&str> = search_content.split('\n').collect();

    if search_lines.last().is_some_and(|l| l.is_empty()) {
        search_lines.pop();
    }

    if search_lines.is_empty() {
        return None;
    }

    let mut start_line_num = 0;
    let mut current_index = 0;
    while current_index < start_index && start_line_num < original_lines.len() {
        current_index += original_lines[start_line_num].len() + 1;
        start_line_num += 1;
    }

    for i in start_line_num..=original_lines.len().saturating_sub(search_lines.len()) {
        let mut matches = true;
        for (j, search_line) in search_lines.iter().enumerate() {
            let original_trimmed = original_lines[i + j].trim();
            let search_trimmed = search_line.trim();
            if original_trimmed != search_trimmed {
                matches = false;
                break;
            }
        }
        if matches {
            let mut match_start_index = 0;
            for k in 0..i {
                match_start_index += original_lines[k].len() + 1;
            }
            let mut match_end_index = match_start_index;
            for k in 0..search_lines.len() {
                match_end_index += original_lines[i + k].len();
                // add newline offset only if not the last line of the file
                if (i + k) < original_lines.len() - 1 {
                    match_end_index += 1;
                }
            }
            return Some((match_start_index, match_end_index));
        }
    }

    None
}

fn block_anchor_fallback_match(
    original_content: &str,
    search_content: &str,
    start_index: usize,
) -> Option<(usize, usize)> {
    let original_lines: Vec<&str> = original_content.split('\n').collect();
    let mut search_lines: Vec<&str> = search_content.split('\n').collect();

    if search_lines.len() < 3 {
        return None;
    }

    if search_lines.last().is_some_and(|l| l.is_empty()) {
        search_lines.pop();
    }

    let first_line_search = search_lines[0].trim();
    let last_line_search = search_lines[search_lines.len() - 1].trim();
    let search_block_size = search_lines.len();

    let mut start_line_num = 0;
    let mut current_index = 0;
    while current_index < start_index && start_line_num < original_lines.len() {
        current_index += original_lines[start_line_num].len() + 1;
        start_line_num += 1;
    }

    for i in start_line_num..=original_lines.len().saturating_sub(search_block_size) {
        if original_lines[i].trim() != first_line_search {
            continue;
        }
        if original_lines[i + search_block_size - 1].trim() != last_line_search {
            continue;
        }

        let mut match_start_index = 0;
        for k in 0..i {
            match_start_index += original_lines[k].len() + 1;
        }
        let mut match_end_index = match_start_index;
        for k in 0..search_block_size {
            match_end_index += original_lines[i + k].len();
            // add newline offset only if not the last line of the file
            if (i + k) < original_lines.len() - 1 {
                match_end_index += 1;
            }
        }
        return Some((match_start_index, match_end_index));
    }

    None
}

pub fn construct_new_file_content_v1(
    diff_content: &str,
    original_content: &str,
    is_final: bool,
) -> Result<String, DiffError> {
    let mut result = String::new();
    let mut last_processed_index: usize = 0;

    let mut current_search_content = String::new();
    let mut current_replace_content = String::new();
    let mut in_search = false;
    let mut in_replace = false;

    let mut search_match_index: isize = -1;
    let mut search_end_index: isize = -1;

    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut pending_out_of_order_replacement = false;

    let mut lines: Vec<&str> = diff_content.split('\n').collect();
    if let Some(last_line) = lines.last().copied() {
        if !lines.is_empty()
            && (last_line.starts_with(SEARCH_BLOCK_CHAR)
                || last_line.starts_with(LEGACY_SEARCH_BLOCK_CHAR)
                || last_line.starts_with('=')
                || last_line.starts_with(REPLACE_BLOCK_CHAR)
                || last_line.starts_with(LEGACY_REPLACE_BLOCK_CHAR))
            && !is_search_block_start(last_line)
            && !is_search_block_end(last_line)
            && !is_replace_block_end(last_line)
        {
            lines.pop();
        }
    }

    for line in lines {
        // Detect malformed marker-like lines only when not inside a block
        if !in_search && !in_replace {
            if (line.starts_with(SEARCH_BLOCK_CHAR) || line.starts_with(LEGACY_SEARCH_BLOCK_CHAR))
                && !is_search_block_start(line)
            {
                return Err(DiffError::NoLinesAvailable);
            }
            if line.starts_with('=') && !is_search_block_end(line) {
                return Err(DiffError::NoLinesAvailable);
            }
            if (line.starts_with(REPLACE_BLOCK_CHAR) || line.starts_with(LEGACY_REPLACE_BLOCK_CHAR))
                && !is_replace_block_end(line)
            {
                return Err(DiffError::NoLinesAvailable);
            }
        }
        if is_search_block_start(line) {
            in_search = true;
            current_search_content.clear();
            current_replace_content.clear();
            continue;
        }

        if is_search_block_end(line) {
            in_search = false;
            in_replace = true;

            if current_search_content.is_empty() {
                // TS v1 lenient behavior: empty SEARCH means replace entire file
                search_match_index = 0;
                search_end_index = original_content.len() as isize;
            } else {
                if let Some(exact_index) = original_content
                    .get(last_processed_index..)
                    .and_then(|slice| slice.find(&current_search_content))
                {
                    let exact = last_processed_index + exact_index;
                    search_match_index = exact as isize;
                    search_end_index = (exact + current_search_content.len()) as isize;
                } else if let Some((start, end)) =
                    line_trimmed_fallback_match(original_content, &current_search_content, last_processed_index)
                {
                    search_match_index = start as isize;
                    search_end_index = end as isize;
                } else if let Some((start, end)) =
                    block_anchor_fallback_match(original_content, &current_search_content, last_processed_index)
                {
                    search_match_index = start as isize;
                    search_end_index = end as isize;
                } else if let Some(full_file_index) = original_content.find(&current_search_content) {
                    search_match_index = full_file_index as isize;
                    search_end_index = (full_file_index + current_search_content.len()) as isize;
                    if (search_match_index as usize) < last_processed_index {
                        pending_out_of_order_replacement = true;
                    }
                } else {
                    return Err(DiffError::SearchBlockNotFound(current_search_content.trim_end().to_string()));
                }
            }

            if (search_match_index as usize) < last_processed_index {
                pending_out_of_order_replacement = true;
            }

            if !pending_out_of_order_replacement {
                result.push_str(&original_content[last_processed_index..(search_match_index as usize)]);
            }
            continue;
        }

        if is_replace_block_end(line) {
            if search_match_index == -1 {
                return Err(DiffError::NoLinesAvailable);
            }

            replacements.push((
                search_match_index as usize,
                search_end_index as usize,
                current_replace_content.clone(),
            ));

            if !pending_out_of_order_replacement {
                last_processed_index = search_end_index as usize;
            }

            in_search = false;
            in_replace = false;
            current_search_content.clear();
            current_replace_content.clear();
            search_match_index = -1;
            search_end_index = -1;
            pending_out_of_order_replacement = false;
            continue;
        }

        if in_search {
            current_search_content.push_str(line);
            current_search_content.push('\n');
        } else if in_replace {
            current_replace_content.push_str(line);
            current_replace_content.push('\n');
            if search_match_index != -1 && !pending_out_of_order_replacement {
                result.push_str(line);
                result.push('\n');
            }
        }
    }

    if is_final {
        if in_replace && search_match_index != -1 {
            replacements.push((
                search_match_index as usize,
                search_end_index as usize,
                current_replace_content.clone(),
            ));
            if !pending_out_of_order_replacement {
                last_processed_index = search_end_index as usize;
            }
            in_search = false;
            in_replace = false;
            current_search_content.clear();
            current_replace_content.clear();
            search_match_index = -1;
            search_end_index = -1;
            pending_out_of_order_replacement = false;
        }

        replacements.sort_by_key(|(start, _, _)| *start);

        result.clear();
        let mut current_pos = 0usize;
        for (start, end, content) in replacements.iter() {
            result.push_str(&original_content[current_pos..*start]);
            result.push_str(content);
            current_pos = *end;
        }
        result.push_str(&original_content[current_pos..]);
    }

    Ok(result)
}