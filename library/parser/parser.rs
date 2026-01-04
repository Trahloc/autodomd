//! TODO parsing functionality for autodomd
//!
//! This microcrate provides parsers for extracting TODO comments from source files
//! and task definitions from markdown files.

use std::fs;
use std::path::Path;
use regex::Regex;

use autodomd_library_common::{Task, TaskCategory, TodoResult};

/// Parse markdown files for TODO tasks
pub fn parse_markdown_files(files: &[std::path::PathBuf]) -> TodoResult<Vec<Task>> {
    let mut tasks = Vec::new();

    for file_path in files {
        match parse_markdown_file(file_path) {
            Ok(file_tasks) => tasks.extend(file_tasks),
            Err(e) => {
                eprintln!("Warning: Failed to parse markdown file {}: {}", file_path.display(), e);
                // Continue with other files
            }
        }
    }

    Ok(tasks)
}

/// Parse a single markdown file for TODO tasks
fn parse_markdown_file(file_path: &Path) -> TodoResult<Vec<Task>> {
    let content = fs::read_to_string(file_path)?;

    // Look for the first H1 header
    if let Some(title) = extract_first_h1(&content) {
        let task = Task::from_markdown(
            title.to_string(),
            TaskCategory::General, // Markdown tasks are general by default
            file_path.to_path_buf(),
        );
        Ok(vec![task])
    } else {
        // Fallback: use filename as title
        let filename = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Task");

        let task = Task::from_markdown(
            filename.to_string(),
            TaskCategory::General,
            file_path.to_path_buf(),
        );
        Ok(vec![task])
    }
}

/// Extract the first H1 header from markdown content
fn extract_first_h1(content: &str) -> Option<&str> {
    // Look for lines starting with # followed by a space
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(&trimmed[2..]); // Skip "# "
        }
    }
    None
}

/// Parse source files for TODO comments
pub fn parse_source_files(files: &[std::path::PathBuf]) -> TodoResult<Vec<Task>> {
    let mut tasks = Vec::new();

    for file_path in files {
        match parse_source_file(file_path) {
            Ok(file_tasks) => tasks.extend(file_tasks),
            Err(e) => {
                eprintln!("Warning: Failed to parse source file {}: {}", file_path.display(), e);
                // Continue with other files
            }
        }
    }

    Ok(tasks)
}

/// Parse a single source file for TODO comments
fn parse_source_file(file_path: &Path) -> TodoResult<Vec<Task>> {
    let content = fs::read_to_string(file_path)?;
    let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let comment_patterns = get_comment_patterns(extension);
    let mut tasks = Vec::new();

    for (line_number, line) in content.lines().enumerate() {
        if let Some(task) = parse_todo_line(line, &comment_patterns, line_number + 1, file_path) {
            tasks.push(task);
        }
    }

    Ok(tasks)
}

/// Get comment patterns for a file extension
fn get_comment_patterns(extension: &str) -> Vec<&'static str> {
    match extension {
        "rs" => vec!["//", "/*"], // Rust
        "js" | "ts" | "jsx" | "tsx" => vec!["//", "/*"], // JavaScript/TypeScript
        "py" => vec!["#"], // Python
        "java" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" => vec!["//", "/*"], // C/C++/Java
        "go" => vec!["//"], // Go
        "rb" => vec!["#"], // Ruby
        "php" => vec!["//", "#", "/*"], // PHP
        "swift" => vec!["//", "/*"], // Swift
        "kt" => vec!["//", "/*"], // Kotlin
        "scala" => vec!["//", "/*"], // Scala
        _ => vec!["//", "#", "/*"], // Default patterns
    }
}

/// Parse a single line for TODO comments
fn parse_todo_line(line: &str, comment_patterns: &[&str], line_number: usize, file_path: &Path) -> Option<Task> {
    let trimmed = line.trim();

    for &pattern in comment_patterns {
        if let Some(comment_start) = trimmed.find(pattern) {
            let after_comment = &trimmed[comment_start + pattern.len()..];

            // Look for TODO followed by optional category
            if let Some(todo_match) = extract_todo_from_comment(after_comment) {
                return Some(Task::from_code(
                    todo_match.title,
                    todo_match.category,
                    file_path.to_path_buf(),
                    line_number,
                ));
            }
        }
    }

    None
}

/// Extract TODO information from a comment
struct TodoMatch {
    title: String,
    category: TaskCategory,
}

fn extract_todo_from_comment(comment: &str) -> Option<TodoMatch> {
    let trimmed = comment.trim();

    // Match: TODO(Category): Description
    let todo_regex = Regex::new(r"TODO(?:\((\w+)\))?\s*:\s*(.+)").ok()?;

    if let Some(captures) = todo_regex.captures(trimmed) {
        let category_str = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        let title = captures.get(2).map(|m| m.as_str().trim())?;

        let category = TaskCategory::from_str(category_str);

        Some(TodoMatch {
            title: title.to_string(),
            category,
        })
    } else {
        None
    }
}
