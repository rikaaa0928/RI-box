use regex::Regex;
use std::sync::OnceLock;
use thiserror::Error;

pub const REPLACE_IN_FILE_TOOL_INSTRUCTIONS: &str =
    include_str!("../replace_in_file_tool_instructions.md");

#[derive(Error, Debug)]
pub enum DiffError {
    #[error("The SEARCH block:\n{0}\n...does not match anything in the file.")]
    SearchBlockNotFound(String),

    #[error("The SEARCH block:\n{0}\n...matched an incorrect content in the file.")]
    SearchBlockIncorrectMatch(String),

    #[error(
        "Invalid state transition.\nValid transitions are:\n- Idle → StateSearch\n- StateSearch → StateReplace"
    )]
    InvalidStateTransition,

    #[error("Invalid SEARCH/REPLACE block structure - no lines available to process")]
    NoLinesAvailable,

    #[error(
        "Invalid REPLACE marker detected - could not find matching SEARCH block starting from line {0}"
    )]
    InvalidReplaceMarker(usize),

    #[error("Malformed REPLACE block - missing valid separator after line {0}")]
    MalformedReplaceBlock(usize),

    #[error("Malformed SEARCH/REPLACE block structure: Missing valid closing REPLACE marker")]
    MissingReplaceMarker,

    #[error(
        "File processing incomplete - SEARCH/REPLACE operations still active during finalization"
    )]
    ProcessingIncomplete,
}

const SEARCH_BLOCK_START: &str = "------- SEARCH";
const SEARCH_BLOCK_END: &str = "=======";
const REPLACE_BLOCK_END: &str = "+++++++ REPLACE";

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

/// Attempts a line-trimmed fallback match
fn line_trimmed_fallback_match(
    original_content: &str,
    search_content: &str,
    start_index: usize,
) -> Option<(usize, usize)> {
    let original_lines: Vec<&str> = original_content.split('\n').collect();
    let mut search_lines: Vec<&str> = search_content.split('\n').collect();

    // Trim trailing empty line if exists
    if search_lines.last().is_some_and(|l| l.is_empty()) {
        search_lines.pop();
    }

    if search_lines.is_empty() {
        return None;
    }

    // Find the line number where start_index falls
    let mut start_line_num = 0;
    let mut current_index = 0;
    while current_index < start_index && start_line_num < original_lines.len() {
        current_index += original_lines[start_line_num].len() + 1; // +1 for \n
        start_line_num += 1;
    }

    // For each possible starting position in original content
    for i in start_line_num..=original_lines.len().saturating_sub(search_lines.len()) {
        let mut matches = true;

        // Try to match all search lines from this position
        for (j, search_line) in search_lines.iter().enumerate() {
            let original_trimmed = original_lines[i + j].trim();
            let search_trimmed = search_line.trim();

            if original_trimmed != search_trimmed {
                matches = false;
                break;
            }
        }

        // If we found a match, calculate the exact character positions
        if matches {
            // Find start character index
            let mut match_start_index = 0;
            for k in 0..i {
                match_start_index += original_lines[k].len() + 1; // +1 for \n
            }

            // Find end character index
            let mut match_end_index = match_start_index;
            for k in 0..search_lines.len() {
                match_end_index += original_lines[i + k].len() + 1; // +1 for \n
            }

            return Some((match_start_index, match_end_index));
        }
    }

    None
}

/// Attempts to match blocks using first and last lines as anchors
fn block_anchor_fallback_match(
    original_content: &str,
    search_content: &str,
    start_index: usize,
) -> Option<(usize, usize)> {
    let original_lines: Vec<&str> = original_content.split('\n').collect();
    let mut search_lines: Vec<&str> = search_content.split('\n').collect();

    // Only use this approach for blocks of 3+ lines
    if search_lines.len() < 3 {
        return None;
    }

    // Trim trailing empty line if exists
    if search_lines.last().is_some_and(|l| l.is_empty()) {
        search_lines.pop();
    }

    let first_line_search = search_lines[0].trim();
    let last_line_search = search_lines[search_lines.len() - 1].trim();
    let search_block_size = search_lines.len();

    // Find the line number where start_index falls
    let mut start_line_num = 0;
    let mut current_index = 0;
    while current_index < start_index && start_line_num < original_lines.len() {
        current_index += original_lines[start_line_num].len() + 1;
        start_line_num += 1;
    }

    // Look for matching start and end anchors
    for i in start_line_num..=original_lines.len().saturating_sub(search_block_size) {
        // Check if first line matches
        if original_lines[i].trim() != first_line_search {
            continue;
        }

        // Check if last line matches at the expected position
        if original_lines[i + search_block_size - 1].trim() != last_line_search {
            continue;
        }

        // Calculate exact character positions
        let mut match_start_index = 0;
        for k in 0..i {
            match_start_index += original_lines[k].len() + 1;
        }

        let mut match_end_index = match_start_index;
        for k in 0..search_block_size {
            match_end_index += original_lines[i + k].len() + 1;
        }

        return Some((match_start_index, match_end_index));
    }

    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessingState {
    Idle = 0,
    StateSearch = 1 << 0,
    StateReplace = 1 << 1,
}

struct NewFileContentConstructor {
    original_content: String,
    is_final: bool,
    state: u8,
    pending_non_standard_lines: Vec<String>,
    result: String,
    last_processed_index: usize,
    current_search_content: String,
    search_match_index: isize,
    search_end_index: isize,
}

impl NewFileContentConstructor {
    fn new(original_content: String, is_final: bool) -> Self {
        Self {
            original_content,
            is_final,
            state: ProcessingState::Idle as u8,
            pending_non_standard_lines: Vec::new(),
            result: String::new(),
            last_processed_index: 0,
            current_search_content: String::new(),
            search_match_index: -1,
            search_end_index: -1,
        }
    }

    fn reset_for_next_block(&mut self) {
        self.state = ProcessingState::Idle as u8;
        self.current_search_content.clear();
        self.search_match_index = -1;
        self.search_end_index = -1;
    }

    fn find_last_matching_line_index(&self, regex: &Regex, line_limit: usize) -> Option<usize> {
        (0..line_limit).rev().find(|&i| regex.is_match(&self.pending_non_standard_lines[i]))
    }

    fn update_processing_state(&mut self, new_state: ProcessingState) -> Result<(), DiffError> {
        let is_valid_transition = (self.state == ProcessingState::Idle as u8
            && new_state == ProcessingState::StateSearch)
            || (self.state == (ProcessingState::StateSearch as u8)
                && new_state == ProcessingState::StateReplace);

        if !is_valid_transition {
            return Err(DiffError::InvalidStateTransition);
        }

        self.state |= new_state as u8;
        Ok(())
    }

    fn is_state_active(&self, state: ProcessingState) -> bool {
        (self.state & (state as u8)) == (state as u8)
    }

    fn activate_replace_state(&mut self) -> Result<(), DiffError> {
        self.update_processing_state(ProcessingState::StateReplace)
    }

    fn activate_search_state(&mut self) -> Result<(), DiffError> {
        self.update_processing_state(ProcessingState::StateSearch)?;
        self.current_search_content.clear();
        Ok(())
    }

    fn is_searching_active(&self) -> bool {
        self.is_state_active(ProcessingState::StateSearch)
    }

    fn is_replacing_active(&self) -> bool {
        self.is_state_active(ProcessingState::StateReplace)
    }

    fn has_pending_non_standard_lines(&self, pending_non_standard_line_limit: usize) -> bool {
        self.pending_non_standard_lines.len() - pending_non_standard_line_limit
            < self.pending_non_standard_lines.len()
    }

    pub fn process_line(&mut self, line: String) -> Result<(), DiffError> {
        let pending_non_standard_line_limit = self.pending_non_standard_lines.len();
        self.internal_process_line(line, true, pending_non_standard_line_limit)?;
        Ok(())
    }

    pub fn get_result(mut self) -> Result<String, DiffError> {
        // Handle the case where we're still in replace mode when processing ends
        // and this is the final chunk - treat it as if we encountered the REPLACE marker
        if self.is_final && self.is_replacing_active() && self.search_match_index != -1 {
            // Finalize the current replacement
            self.last_processed_index = self.search_end_index as usize;
            self.reset_for_next_block();
        }

        // If this is the final chunk, append any remaining original content
        if self.is_final && self.last_processed_index < self.original_content.len() {
            self.result
                .push_str(&self.original_content[self.last_processed_index..]);
        }

        if self.is_final && self.state != ProcessingState::Idle as u8 {
            return Err(DiffError::ProcessingIncomplete);
        }
        Ok(self.result)
    }

    fn internal_process_line(
        &mut self,
        line: String,
        can_write_pending_non_standard_lines: bool,
        mut pending_non_standard_line_limit: usize,
    ) -> Result<usize, DiffError> {
        let mut remove_line_count = 0;

        if is_search_block_start(&line) {
            remove_line_count = self
                .trim_pending_non_standard_trailing_empty_lines(pending_non_standard_line_limit);
            if remove_line_count > 0 {
                pending_non_standard_line_limit =
                    pending_non_standard_line_limit.saturating_sub(remove_line_count);
            }
            if self.has_pending_non_standard_lines(pending_non_standard_line_limit) {
                self.try_fix_search_replace_block(pending_non_standard_line_limit)?;
                if can_write_pending_non_standard_lines {
                    self.pending_non_standard_lines.clear();
                }
            }
            self.activate_search_state()?;
        } else if is_search_block_end(&line) {
            // 校验非标内容
            if !self.is_searching_active() {
                self.try_fix_search_block(pending_non_standard_line_limit)?;
                if can_write_pending_non_standard_lines {
                    self.pending_non_standard_lines.clear();
                }
            }
            self.activate_replace_state()?;
            self.before_replace()?;
        } else if is_replace_block_end(&line) {
            if !self.is_replacing_active() {
                self.try_fix_replace_block(pending_non_standard_line_limit)?;
                if can_write_pending_non_standard_lines {
                    self.pending_non_standard_lines.clear();
                }
            }
            self.last_processed_index = self.search_end_index as usize;
            self.reset_for_next_block();
        } else if self.is_replacing_active() {
            // Output replacement lines immediately if we know the insertion point
            if self.search_match_index != -1 {
                self.result.push_str(&line);
                self.result.push('\n');
            }
        } else if self.is_searching_active() {
            self.current_search_content.push_str(&line);
            self.current_search_content.push('\n');
        } else if can_write_pending_non_standard_lines {
            // 处理非标内容
            self.pending_non_standard_lines.push(line);
        }

        Ok(remove_line_count)
    }

    fn before_replace(&mut self) -> Result<(), DiffError> {
        if self.current_search_content.is_empty() {
            // Empty search block
            if self.original_content.is_empty() {
                // New file scenario: nothing to match, just start inserting
                self.search_match_index = 0;
                self.search_end_index = 0;
            } else {
                // Complete file replacement scenario: treat the entire file as matched
                self.search_match_index = 0;
                self.search_end_index = self.original_content.len() as isize;
            }
        } else {
            // Exact search match scenario
            if let Some(exact_index) = self.original_content[self.last_processed_index..]
                .find(&self.current_search_content)
            {
                let exact_index = self.last_processed_index + exact_index;
                self.search_match_index = exact_index as isize;
                self.search_end_index = (exact_index + self.current_search_content.len()) as isize;
            } else {
                // Attempt fallback line-trimmed matching
                if let Some((match_start, match_end)) = line_trimmed_fallback_match(
                    &self.original_content,
                    &self.current_search_content,
                    self.last_processed_index,
                ) {
                    self.search_match_index = match_start as isize;
                    self.search_end_index = match_end as isize;
                } else {
                    // Try block anchor fallback for larger blocks
                    if let Some((match_start, match_end)) = block_anchor_fallback_match(
                        &self.original_content,
                        &self.current_search_content,
                        self.last_processed_index,
                    ) {
                        self.search_match_index = match_start as isize;
                        self.search_end_index = match_end as isize;
                    } else {
                        return Err(DiffError::SearchBlockNotFound(
                            self.current_search_content.trim_end().to_string(),
                        ));
                    }
                }
            }
        }

        if (self.search_match_index as usize) < self.last_processed_index {
            return Err(DiffError::SearchBlockIncorrectMatch(
                self.current_search_content.trim_end().to_string(),
            ));
        }

        // Output everything up to the match location
        self.result.push_str(
            &self.original_content[self.last_processed_index..self.search_match_index as usize],
        );

        Ok(())
    }

    fn try_fix_search_block(&mut self, line_limit: usize) -> Result<usize, DiffError> {
        let mut remove_line_count = 0;
        let line_limit = if line_limit == 0 {
            return Err(DiffError::NoLinesAvailable);
        } else {
            line_limit
        };

        let search_tag_regexp = Regex::new(r"^([-]{3,}|[<]{3,}) SEARCH$").unwrap();
        let search_tag_index = self
            .find_last_matching_line_index(&search_tag_regexp, line_limit)
            .ok_or(DiffError::InvalidReplaceMarker(0))?;

        let fix_lines: Vec<String> =
            self.pending_non_standard_lines[search_tag_index..line_limit].to_vec();
        let mut fix_lines = fix_lines;
        fix_lines[0] = SEARCH_BLOCK_START.to_string();

        for line in fix_lines {
            remove_line_count += self.internal_process_line(line, false, search_tag_index)?;
        }

        Ok(remove_line_count)
    }

    fn try_fix_replace_block(&mut self, line_limit: usize) -> Result<usize, DiffError> {
        let mut remove_line_count = 0;
        let line_limit = if line_limit == 0 {
            return Err(DiffError::NoLinesAvailable);
        } else {
            line_limit
        };

        let replace_begin_tag_regexp = Regex::new(r"^[=]{3,}$").unwrap();
        let replace_begin_tag_index = self
            .find_last_matching_line_index(&replace_begin_tag_regexp, line_limit)
            .ok_or(DiffError::MalformedReplaceBlock(0))?;

        let fix_lines: Vec<String> = self.pending_non_standard_lines[replace_begin_tag_index
            .saturating_sub(remove_line_count)
            ..line_limit.saturating_sub(remove_line_count)]
            .to_vec();
        let mut fix_lines = fix_lines;
        fix_lines[0] = SEARCH_BLOCK_END.to_string();

        for line in fix_lines {
            remove_line_count += self.internal_process_line(
                line,
                false,
                replace_begin_tag_index.saturating_sub(remove_line_count),
            )?;
        }

        Ok(remove_line_count)
    }

    fn try_fix_search_replace_block(&mut self, line_limit: usize) -> Result<usize, DiffError> {
        let mut remove_line_count = 0;
        let line_limit = if line_limit == 0 {
            return Err(DiffError::NoLinesAvailable);
        } else {
            line_limit
        };

        let replace_end_tag_regexp = Regex::new(r"^([+]{3,}|[>]{3,}) REPLACE$").unwrap();
        let replace_end_tag_index =
            self.find_last_matching_line_index(&replace_end_tag_regexp, line_limit);
        let like_replace_end_tag = replace_end_tag_index == Some(line_limit - 1);

        if like_replace_end_tag {
            let replace_end_tag_index = replace_end_tag_index.unwrap();
            let mut fix_lines: Vec<String> = self.pending_non_standard_lines[replace_end_tag_index
                .saturating_sub(remove_line_count)
                ..line_limit.saturating_sub(remove_line_count)]
                .to_vec();
            let last_idx = fix_lines.len() - 1;
            fix_lines[last_idx] = REPLACE_BLOCK_END.to_string();

            for line in fix_lines {
                remove_line_count += self.internal_process_line(
                    line,
                    false,
                    replace_end_tag_index.saturating_sub(remove_line_count),
                )?;
            }
        } else {
            return Err(DiffError::MissingReplaceMarker);
        }

        Ok(remove_line_count)
    }

    fn trim_pending_non_standard_trailing_empty_lines(&mut self, line_limit: usize) -> usize {
        let mut removed_count = 0;
        let mut i = line_limit.min(self.pending_non_standard_lines.len());

        while i > 0 {
            i -= 1;
            if self.pending_non_standard_lines[i].trim().is_empty() {
                self.pending_non_standard_lines.pop();
                removed_count += 1;
            } else {
                break;
            }
        }

        removed_count
    }
}

pub fn construct_new_file_content_v2(
    diff_content: &str,
    original_content: &str,
    is_final: bool,
) -> Result<String, DiffError> {
    let mut constructor = NewFileContentConstructor::new(original_content.to_string(), is_final);

    let mut lines: Vec<&str> = diff_content.split('\n').collect();

    // If the last line looks like a partial marker but isn't recognized, remove it
    if let Some(last_line) = lines.last()
        && !last_line.is_empty()
            && (last_line.starts_with(SEARCH_BLOCK_CHAR)
                || last_line.starts_with(LEGACY_SEARCH_BLOCK_CHAR)
                || last_line.starts_with("=")
                || last_line.starts_with(REPLACE_BLOCK_CHAR)
                || last_line.starts_with(LEGACY_REPLACE_BLOCK_CHAR))
            && *last_line != SEARCH_BLOCK_START
            && *last_line != SEARCH_BLOCK_END
            && *last_line != REPLACE_BLOCK_END
        {
            lines.pop();
        }

    for line in lines {
        constructor.process_line(line.to_string())?;
    }

    constructor.get_result()
}
