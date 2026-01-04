//! TODO parsing functionality for autodomd
//!
//! This microcrate provides parsers for extracting TODO comments from source files
//! and task definitions from markdown files.

use std::fs;
use std::path::Path;
use regex::Regex;

use autodomd_library_common::{Task, TaskCategory, TaskPriority, TodoResult};

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

    // Extract category from directory structure
    let category = extract_category_from_path(file_path);

    // Extract priority from metadata (default to Medium)
    let priority = extract_priority_from_content(&content);

    // Look for the first H1 header
    if let Some(title) = extract_first_h1(&content) {
        let task = Task::from_markdown_with_priority(
            title.to_string(),
            category,
            priority,
            file_path.to_path_buf(),
        );
        Ok(vec![task])
    } else {
        // Fallback: use filename as title
        let filename = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Task");

        let task = Task::from_markdown_with_priority(
            filename.to_string(),
            category,
            priority,
            file_path.to_path_buf(),
        );
        Ok(vec![task])
    }
}

/// Extract priority from markdown content (looks for YAML metadata or **Priority:** line)
fn extract_priority_from_content(content: &str) -> TaskPriority {
    // First try to extract from YAML metadata block
    if let Some(yaml_block) = extract_yaml_metadata(content) {
        if let Some(priority_line) = yaml_block.lines().find(|line| line.starts_with("priority:")) {
            if let Some(priority_str) = priority_line.strip_prefix("priority:").map(|s| s.trim().trim_matches('"')) {
                return match priority_str.to_lowercase().as_str() {
                    "high" => TaskPriority::High,
                    "low" => TaskPriority::Low,
                    _ => TaskPriority::Medium,
                };
            }
        }
    }

    // Fallback to old **Priority:** format
    for line in content.lines() {
        if let Some(priority_str) = line.strip_prefix("**Priority:**") {
            let priority = priority_str.trim().to_lowercase();
            return match priority.as_str() {
                "high" => TaskPriority::High,
                "low" => TaskPriority::Low,
                _ => TaskPriority::Medium,
            };
        }
    }
    TaskPriority::Medium // Default
}

/// Extract YAML metadata block from markdown content
fn extract_yaml_metadata(content: &str) -> Option<&str> {
    let lines: Vec<&str> = content.lines().collect();

    // Look for ```yaml or ``` followed by yaml content
    let mut in_yaml_block = false;
    let mut yaml_start = None;

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "```yaml" || (line.trim() == "```" && i < lines.len() - 1 && lines[i + 1].contains("priority:")) {
            in_yaml_block = true;
            yaml_start = Some(i + 1);
        } else if in_yaml_block && line.trim() == "```" {
            if let Some(start) = yaml_start {
                return Some(&content[lines[start..i].iter().map(|s| s.len() + 1).sum::<usize>() - 1..lines[i].as_ptr() as usize - content.as_ptr() as usize]);
            }
        }
    }

    None
}

/// Extract category from the directory path relative to todo/
fn extract_category_from_path(file_path: &Path) -> TaskCategory {
    // Convert path to string for easier manipulation
    let path_str = file_path.to_string_lossy();

    // Find the todo/ directory in the path
    if let Some(todo_pos) = path_str.find("todo/") {
        let after_todo = &path_str[todo_pos + 5..]; // Skip "todo/"

        // Find the next directory separator
        if let Some(separator_pos) = after_todo.find('/') {
            let category_dir = &after_todo[..separator_pos];

            // Capitalize first letter for better display
            let capitalized = category_dir
                .chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
                .collect::<String>();

            TaskCategory::Custom(capitalized)
        } else {
            // No subdirectory, check if it's directly in todo/
            TaskCategory::General
        }
    } else {
        // Not in todo/ directory, use general category
        TaskCategory::General
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

    // Only process lines that start with a comment pattern (after trimming whitespace)
    // This avoids matching TODO patterns inside string literals
    for &pattern in comment_patterns {
        if trimmed.starts_with(pattern) {
            let after_comment = &trimmed[pattern.len()..];

            // Look for TODO followed by optional category
            if let Some(todo_match) = extract_todo_from_comment(after_comment.trim_start()) {
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
