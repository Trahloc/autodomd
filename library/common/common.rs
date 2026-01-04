//! Common utilities and types for autodomd
//!
//! This microcrate contains shared types, error handling, and utility functions
//! used across the autodomd project.

use std::fmt;
use std::path::PathBuf;

/// Represents the source of a TODO task
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskSource {
    /// Task from a markdown file in todo/ directory
    Markdown,
    /// Task from a TODO comment in source code
    Code,
}

/// Represents the location of a TODO task
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLocation {
    /// Path to the file containing the task
    pub file_path: PathBuf,
    /// Line number where the task was found (1-indexed)
    pub line_number: Option<usize>,
}

impl TaskLocation {
    /// Create a new TaskLocation
    pub fn new(file_path: PathBuf, line_number: Option<usize>) -> Self {
        Self {
            file_path,
            line_number,
        }
    }

    /// Create a TaskLocation for a markdown file (no line number)
    pub fn from_markdown_file(file_path: PathBuf) -> Self {
        Self::new(file_path, None)
    }

    /// Create a TaskLocation for source code (with line number)
    pub fn from_source_file(file_path: PathBuf, line_number: usize) -> Self {
        Self::new(file_path, Some(line_number))
    }
}

/// Represents a category for grouping tasks
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskCategory {
    /// Uncategorized tasks
    General,
    /// Custom category from TODO(Category) syntax
    Custom(String),
}

/// Priority levels for task organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

impl TaskCategory {
    /// Create a TaskCategory from a string
    pub fn from_str(s: &str) -> Self {
        if s.is_empty() {
            TaskCategory::General
        } else {
            TaskCategory::Custom(s.to_string())
        }
    }

    /// Get the display name for this category
    pub fn display_name(&self) -> &str {
        match self {
            TaskCategory::General => "General",
            TaskCategory::Custom(name) => name,
        }
    }
}

impl fmt::Display for TaskCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Represents a single TODO task
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// The title/description of the task
    pub title: String,
    /// The category this task belongs to
    pub category: TaskCategory,
    /// Priority level for organization
    pub priority: TaskPriority,
    /// Where this task was found
    pub location: TaskLocation,
    /// The source type of this task
    pub source: TaskSource,
}

impl Task {
    /// Create a new Task
    pub fn new(
        title: String,
        category: TaskCategory,
        priority: TaskPriority,
        location: TaskLocation,
        source: TaskSource,
    ) -> Self {
        Self {
            title,
            category,
            priority,
            location,
            source,
        }
    }

    /// Create a task from a markdown file
    pub fn from_markdown(title: String, category: TaskCategory, file_path: PathBuf) -> Self {
        Self::new(
            title,
            category,
            TaskPriority::Medium, // Default priority for markdown tasks
            TaskLocation::from_markdown_file(file_path),
            TaskSource::Markdown,
        )
    }

    /// Create a task from source code
    pub fn from_code(
        title: String,
        category: TaskCategory,
        file_path: PathBuf,
        line_number: usize,
    ) -> Self {
        Self::new(
            title,
            category,
            TaskPriority::Medium, // Default priority for code tasks
            TaskLocation::from_source_file(file_path, line_number),
            TaskSource::Code,
        )
    }

    /// Create a task from markdown with explicit priority
    pub fn from_markdown_with_priority(
        title: String,
        category: TaskCategory,
        priority: TaskPriority,
        file_path: PathBuf,
    ) -> Self {
        Self::new(
            title,
            category,
            priority,
            TaskLocation::from_markdown_file(file_path),
            TaskSource::Markdown,
        )
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location.line_number {
            Some(line) => write!(
                f,
                "- [ ] {} ({}:{})",
                self.title,
                self.location.file_path.display(),
                line
            ),
            None => write!(
                f,
                "- [ ] {} ({})",
                self.title,
                self.location.file_path.display()
            ),
        }
    }
}

/// Collection of tasks with utilities for management
#[derive(Debug, Clone, Default)]
pub struct TaskCollection {
    pub tasks: Vec<Task>,
}

impl TaskCollection {
    /// Create a new empty TaskCollection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a task to the collection
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Add multiple tasks to the collection
    pub fn extend(&mut self, tasks: impl IntoIterator<Item = Task>) {
        self.tasks.extend(tasks);
    }

    /// Get all tasks
    pub fn all_tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Get the number of tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Sort tasks by file path and line number for deterministic output
    pub fn sort(&mut self) {
        self.tasks.sort_by(|a, b| {
            a.location
                .file_path
                .cmp(&b.location.file_path)
                .then_with(|| a.location.line_number.cmp(&b.location.line_number))
                .then_with(|| a.title.cmp(&b.title))
        });
    }
}

/// Custom error type for autodomd operations
#[derive(Debug, thiserror::Error)]
pub enum TodoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
}

pub type TodoResult<T> = Result<T, TodoError>;
