//! TODO.md generation functionality for autodomd
//!
//! This microcrate provides functionality for generating the final TODO.md file
//! from parsed TODO items and tasks.

use std::collections::HashMap;
use std::fs;

use autodomd_library_common::{Task, TaskCollection, TaskSource, TodoResult};

/// Configuration for TODO.md generation
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Output file path (usually "TODO.md")
    pub output_path: std::path::PathBuf,
    /// Project root path for relative path calculation
    pub root_path: std::path::PathBuf,
    /// Whether to include auto-generated warning
    pub include_header: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            output_path: std::path::PathBuf::from("TODO.md"),
            root_path: std::path::PathBuf::from("."),
            include_header: true,
        }
    }
}

/// Generate TODO.md file from a task collection
pub fn generate_todo_md(tasks: &TaskCollection, config: &GeneratorConfig) -> TodoResult<()> {
    let mut sorted_tasks = tasks.tasks.clone();
    sorted_tasks.sort_by(|a, b| {
        // Sort by category first, then by file path, then by line number
        match a.category.display_name().cmp(b.category.display_name()) {
            std::cmp::Ordering::Equal => {
                a.location.file_path.cmp(&b.location.file_path)
                    .then_with(|| a.location.line_number.cmp(&b.location.line_number))
            }
            other => other,
        }
    });

    let content = generate_markdown_content(&sorted_tasks, config);
    fs::write(&config.output_path, content)?;
    Ok(())
}

/// Generate the markdown content for TODO.md
fn generate_markdown_content(tasks: &[Task], config: &GeneratorConfig) -> String {
    let mut content = String::new();

    // Add header
    if config.include_header {
        content.push_str("# Project Tasks\n");
        content.push_str("*Auto-generated. Do not edit.*\n\n");
        content.push_str("---\n\n");
    }

    // Group tasks by category
    let mut tasks_by_category: HashMap<String, Vec<&Task>> = HashMap::new();

    for task in tasks {
        let category_name = task.category.display_name().to_string();
        tasks_by_category.entry(category_name).or_insert_with(Vec::new).push(task);
    }

    // Generate sections for each category, sorted by priority within each category
    let mut category_names: Vec<String> = tasks_by_category.keys().cloned().collect();
    category_names.sort();

    for category_name in category_names {
        if let Some(category_tasks) = tasks_by_category.get_mut(&category_name) {
            content.push_str(&format!("## {}\n\n", category_name));

            // Sort tasks within category by priority (High first, then Medium, then Low)
            category_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

            for task in category_tasks.iter() {
                content.push_str(&format!("{}\n", task));

                // Add brief description if available (from markdown tasks)
                if let TaskSource::Markdown = task.source {
                    if let Ok(description) = extract_brief_description(&task.location.file_path) {
                        if !description.is_empty() {
                            content.push_str(&format!("  *{}*\n", description));
                        }
                    }
                }
            }

            content.push_str("\n");
        }
    }

    // If no tasks found, add a note
    if tasks.is_empty() {
        content.push_str("*No tasks found.*\n");
    }

    content
}

/// Generate a simple summary of tasks by category
pub fn generate_summary(tasks: &[Task]) -> String {
    let mut summary = String::new();
    let mut category_counts: HashMap<String, usize> = HashMap::new();

    for task in tasks {
        let category_name = task.category.display_name().to_string();
        *category_counts.entry(category_name).or_insert(0) += 1;
    }

    summary.push_str("## Task Summary\n\n");

    if category_counts.is_empty() {
        summary.push_str("*No tasks found.*\n");
    } else {
        let mut sorted_categories: Vec<(String, usize)> = category_counts.into_iter().collect();
        sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));

        for (category, count) in sorted_categories {
            summary.push_str(&format!("- **{}**: {} tasks\n", category, count));
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use autodomd_library_common::{TaskLocation, TaskSource};

    #[test]
    fn test_generate_markdown_content_empty() {
        let tasks = Vec::new();
        let config = GeneratorConfig::default();
        let content = generate_markdown_content(&tasks, &config);

        assert!(content.contains("Auto-generated"));
        assert!(content.contains("No tasks found"));
    }

    #[test]
    fn test_generate_markdown_content_with_tasks() {
        let task = Task::from_code(
            "Test task".to_string(),
            TaskCategory::General,
            std::path::PathBuf::from("test.rs"),
            42,
        );

        let tasks = vec![task];
        let config = GeneratorConfig::default();
        let content = generate_markdown_content(&tasks, &config);

        assert!(content.contains("Auto-generated"));
        assert!(content.contains("General"));
        assert!(content.contains("Test task"));
        assert!(content.contains("test.rs:42"));
    }

    #[test]
    fn test_generate_summary() {
        let task1 = Task::from_code(
            "Task 1".to_string(),
            TaskCategory::General,
            std::path::PathBuf::from("test.rs"),
            1,
        );

        let task2 = Task::from_code(
            "Task 2".to_string(),
            TaskCategory::Custom("Auth".to_string()),
            std::path::PathBuf::from("auth.rs"),
            2,
        );

        let tasks = vec![task1, task2];
        let summary = generate_summary(&tasks);

        assert!(summary.contains("Task Summary"));
        assert!(summary.contains("General"));
        assert!(summary.contains("Auth"));
    }
}

/// Extract a brief description from a markdown task file
fn extract_brief_description(file_path: &std::path::Path) -> TodoResult<String> {
    let content = std::fs::read_to_string(file_path)?;

    // Skip YAML metadata block if present
    let content_after_yaml = if content.trim().starts_with("```yaml") {
        if let Some(end_yaml) = content.find("```\n") {
            &content[end_yaml + 4..]
        } else {
            &content
        }
    } else {
        &content
    };

    // Look for Overview section
    if let Some(overview_start) = content_after_yaml.find("## Overview") {
        let after_overview = &content_after_yaml[overview_start + 11..];
        if let Some(end_section) = after_overview.find("\n## ") {
            let overview_text = &after_overview[..end_section].trim();
            // Take first sentence or first 100 chars
            if let Some(first_sentence_end) = overview_text.find('.') {
                if first_sentence_end < 100 {
                    return Ok(overview_text[..first_sentence_end + 1].to_string());
                }
            }
            Ok(overview_text.chars().take(100).collect::<String>() + if overview_text.len() > 100 { "..." } else { "" })
        } else {
            Ok("".to_string())
        }
    } else {
        Ok("".to_string())
    }
}
