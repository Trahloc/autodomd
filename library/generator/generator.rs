//! TODO.md generation functionality for autodomd
//!
//! This microcrate provides functionality for generating the final TODO.md file
//! from parsed TODO items and tasks.

use std::collections::HashMap;
use std::fs;

use autodomd_library_common::{Task, TaskCollection, TaskSource, TodoResult};
use chrono::{DateTime, Utc};

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
        content.push_str("---\n");
        content.push_str("format: extramark-todo-v1\n");
        content.push_str("generator: autodomd\n");
        content.push_str("generated_at: ");
        let now: DateTime<Utc> = Utc::now();
        content.push_str(&now.format("%Y-%m-%dT%H:%M:%SZ").to_string());
        content.push_str("\n");
        content.push_str("total_tasks: ");
        content.push_str(&tasks.len().to_string());
        content.push_str("\n");
        content.push_str("regenerate_command: autodomd generate\n");
        content.push_str("---\n\n");
        content.push_str("# Project Tasks\n\n");
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
            content.push_str(&format!("## {} Tasks\n\n", category_name));

            // Sort tasks within category by priority (High first, then Medium, then Low)
            category_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

            for task in category_tasks.iter() {
                // Clean, minimal task format with proper hierarchy
                content.push_str(&format!("### {}\n\n", task.title));

                // Add brief description if available
                if let TaskSource::Markdown = task.source {
                    if let Ok(description) = extract_brief_description(&task.location.file_path) {
                        if !description.is_empty() {
                            content.push_str(&format!("{}\n\n", description));
                        }
                    }
                }

                // Add key metadata in clean format
                let mut metadata_lines = Vec::new();

                // Add timestamps
                if let Ok(metadata) = std::fs::metadata(&task.location.file_path) {
                    if let Ok(created) = metadata.created() {
                        let created_dt: DateTime<Utc> = created.into();
                        metadata_lines.push(format!("📅 Created: {}", created_dt.format("%Y-%m-%d")));
                    }
                    if let Ok(modified) = metadata.modified() {
                        let modified_dt: DateTime<Utc> = modified.into();
                        metadata_lines.push(format!("🔄 Modified: {}", modified_dt.format("%Y-%m-%d")));
                    }
                }

                // Add extracted metadata from YAML frontmatter (dependencies, effort, etc.)
                if let TaskSource::Markdown = task.source {
                    if let Ok(metadata) = extract_task_metadata(&task.location.file_path) {
                        for (key, value) in metadata {
                            match key.as_str() {
                                "dependencies" if !value.is_empty() && value != "[]" => {
                                    let cleaned = clean_yaml_array(&value);
                                    if !cleaned.is_empty() {
                                        metadata_lines.push(format!("🔗 Depends: {}", cleaned));
                                    }
                                }
                                "blocks" if !value.is_empty() && value != "[]" => {
                                    let cleaned = clean_yaml_array(&value);
                                    if !cleaned.is_empty() {
                                        metadata_lines.push(format!("🚫 Blocks: {}", cleaned));
                                    }
                                }
                                "estimated_effort" => {
                                    metadata_lines.push(format!("⚡ Effort: {}", value));
                                }
                                _ => {}
                            }
                        }
                    }
                }

                if !metadata_lines.is_empty() {
                    content.push_str(&metadata_lines.join(" • "));
                    content.push_str("\n\n");
                }

                // Show blocking relationships as indented sub-items
                if let TaskSource::Markdown = task.source {
                    if let Ok(metadata) = extract_task_metadata(&task.location.file_path) {
                        for (key, value) in metadata {
                            if key == "blocks" && !value.is_empty() && value != "[]" {
                                let blockers = clean_yaml_array(&value);
                                if !blockers.is_empty() {
                                    content.push_str("**Enables work on:**\n");
                                    for blocker in blockers.split(", ") {
                                        content.push_str(&format!("  - `{}`\n", blocker.trim()));
                                    }
                                    content.push_str("\n");
                                }
                            }
                        }
                    }
                }

                // Link to full specification
                content.push_str(&format!("📄 {}\n\n", task.location.file_path.display()));
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

/// Extract structured metadata from a markdown task file
fn extract_task_metadata(file_path: &std::path::Path) -> TodoResult<Vec<(String, String)>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut metadata = Vec::new();

    // Extract YAML metadata block
    if let Some(yaml_block) = extract_yaml_metadata_block(&content) {
        for line in yaml_block.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().trim_matches('"').to_string();
                if !key.is_empty() && !value.is_empty() {
                    metadata.push((key, value));
                }
            }
        }
    }

    Ok(metadata)
}

/// Extract YAML metadata block from markdown content
fn extract_yaml_metadata_block(content: &str) -> Option<&str> {
    let lines: Vec<&str> = content.lines().collect();

    // Look for ```yaml or ``` followed by yaml content
    let mut in_yaml_block = false;
    let mut yaml_start = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "```yaml" || (trimmed == "```" && i < lines.len() - 1 &&
            (lines[i + 1].contains("priority:") || lines[i + 1].contains("dependencies:"))) {
            in_yaml_block = true;
            yaml_start = Some(i + 1);
        } else if in_yaml_block && trimmed == "```" {
            if let Some(start) = yaml_start {
                return Some(&content[lines[start].as_ptr() as usize - content.as_ptr() as usize..
                               lines[i].as_ptr() as usize - content.as_ptr() as usize]);
            }
        }
    }

    None
}

/// Clean YAML array format for display
fn clean_yaml_array(value: &str) -> String {
    // Remove brackets and quotes, clean up commas
    value.trim_matches(&['[', ']'] as &[_])
         .split(',')
         .map(|s| s.trim().trim_matches('"'))
         .filter(|s| !s.is_empty())
         .collect::<Vec<&str>>()
         .join(", ")
}

/// Extract a brief description from a markdown task file
fn extract_brief_description(file_path: &std::path::Path) -> TodoResult<String> {
    let content = std::fs::read_to_string(file_path)?;

    // Skip YAML metadata block if present
    let content_after_yaml = if let Some(yaml_block) = extract_yaml_metadata_block(&content) {
        // Find the end of YAML block and skip to content after ```
        if let Some(end_pos) = content[yaml_block.as_ptr() as usize - content.as_ptr() as usize..].find("```\n") {
            let yaml_end_pos = yaml_block.as_ptr() as usize - content.as_ptr() as usize + end_pos + 4;
            &content[yaml_end_pos..]
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
            // Take first sentence or first 200 chars for more context
            if let Some(first_sentence_end) = overview_text.find('.') {
                if first_sentence_end < 200 {
                    return Ok(overview_text[..first_sentence_end + 1].to_string());
                }
            }
            Ok(overview_text.chars().take(200).collect::<String>() + if overview_text.len() > 200 { "..." } else { "" })
        } else {
            // Take the whole overview section if no other sections follow
            Ok(after_overview.trim().chars().take(200).collect::<String>() + if after_overview.len() > 200 { "..." } else { "" })
        }
    } else {
        Ok("".to_string())
    }
}
