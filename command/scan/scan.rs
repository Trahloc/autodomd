//! Scan command for autodomd
//!
//! This microcrate implements the scan subcommand that searches for TODO comments
//! and markdown tasks in the project.

use std::path::PathBuf;

use autodomd_library_common::{TaskCollection, TodoResult};
use autodomd_library_scanner::{scan_all_files, ScanConfig};
use autodomd_library_parser::{parse_markdown_files, parse_source_files};

/// Configuration for the scan command
#[derive(Debug, Clone)]
pub struct ScanCommandConfig {
    /// Root directory to scan (defaults to current directory)
    pub root_path: Option<PathBuf>,
    /// Whether to follow symbolic links
    pub follow_links: bool,
    /// Maximum scan depth
    pub max_depth: Option<usize>,
    /// Whether to output verbose information
    pub verbose: bool,
}

impl Default for ScanCommandConfig {
    fn default() -> Self {
        Self {
            root_path: None,
            follow_links: false,
            max_depth: Some(10),
            verbose: false,
        }
    }
}

impl ScanCommandConfig {
    /// Convert to ScanConfig for the scanner library
    fn to_scan_config(&self) -> ScanConfig {
        ScanConfig {
            root_path: self.root_path.clone().unwrap_or_else(|| PathBuf::from(".")),
            follow_links: self.follow_links,
            max_depth: self.max_depth,
        }
    }
}

/// Result of a scan operation
#[derive(Debug)]
pub struct ScanResult {
    /// All discovered tasks
    pub tasks: TaskCollection,
    /// Number of markdown files scanned
    pub markdown_files_scanned: usize,
    /// Number of source files scanned
    pub source_files_scanned: usize,
    /// Number of tasks found
    pub tasks_found: usize,
}

/// Execute the scan command
pub fn run_scan(config: &ScanCommandConfig) -> TodoResult<ScanResult> {
    if config.verbose {
        println!("Starting scan...");
        println!("Root path: {}", config.to_scan_config().root_path.display());
    }

    // Scan for files
    let scan_config = config.to_scan_config();
    let (markdown_files, source_files) = scan_all_files(&scan_config)?;

    if config.verbose {
        println!("Found {} markdown files", markdown_files.len());
        println!("Found {} source files", source_files.len());
    }

    // Parse files for tasks
    let mut all_tasks = TaskCollection::new();

    // Parse markdown files
    let markdown_tasks = parse_markdown_files(&markdown_files)?;
    all_tasks.extend(markdown_tasks);

    // Parse source files
    let source_tasks = parse_source_files(&source_files)?;
    all_tasks.extend(source_tasks);

    // Sort tasks for consistent output
    all_tasks.sort();

    let tasks_found = all_tasks.len();

    let result = ScanResult {
        tasks: all_tasks,
        markdown_files_scanned: markdown_files.len(),
        source_files_scanned: source_files.len(),
        tasks_found,
    };

    if config.verbose {
        println!("Scan complete:");
        println!("  Markdown files: {}", result.markdown_files_scanned);
        println!("  Source files: {}", result.source_files_scanned);
        println!("  Tasks found: {}", result.tasks_found);
    }

    Ok(result)
}
