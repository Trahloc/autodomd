//! File and directory scanning functionality for autodomd
//!
//! This microcrate provides utilities for scanning source files and markdown files
//! to extract TODO comments and tasks.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use autodomd_library_common::TodoResult;

/// Configuration for file scanning operations
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Root directory to scan from
    pub root_path: PathBuf,
    /// Whether to follow symbolic links
    pub follow_links: bool,
    /// Maximum depth to scan (None for unlimited)
    pub max_depth: Option<usize>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            follow_links: false,
            max_depth: Some(10), // Reasonable default to prevent infinite recursion
        }
    }
}

/// Scan for markdown files in todo/ directories
pub fn scan_markdown_files(config: &ScanConfig) -> TodoResult<Vec<PathBuf>> {
    let todo_dir = config.root_path.join("todo");
    if !todo_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    let walker = WalkDir::new(&todo_dir)
        .follow_links(config.follow_links)
        .max_depth(config.max_depth.unwrap_or(usize::MAX))
        .into_iter()
        .filter_map(|entry| entry.ok()) // Skip entries with errors
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .map(|entry| entry.path().to_path_buf());

    files.extend(walker);

    // Sort for deterministic output
    files.sort();

    Ok(files)
}

/// Supported source file extensions for TODO comment scanning
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs",    // Rust
    "js",    // JavaScript
    "ts",    // TypeScript
    "jsx",   // React JavaScript
    "tsx",   // React TypeScript
    "py",    // Python
    "java",  // Java
    "c",     // C
    "cpp",   // C++
    "cc",    // C++
    "cxx",   // C++
    "h",     // C/C++ headers
    "hpp",   // C++ headers
    "go",    // Go
    "rb",    // Ruby
    "php",   // PHP
    "swift", // Swift
    "kt",    // Kotlin
    "scala", // Scala
];

/// Directories to exclude from scanning
const EXCLUDED_DIRS: &[&str] = &[
    "target",     // Rust build artifacts
    "node_modules", // Node.js dependencies
    "__pycache__",  // Python cache
    ".git",        // Git repository
    ".svn",        // SVN repository
    ".hg",         // Mercurial repository
    ".DS_Store",   // macOS
    "Thumbs.db",   // Windows
];

/// Check if a directory should be excluded from scanning
fn should_exclude_dir(dir_name: &str) -> bool {
    EXCLUDED_DIRS.contains(&dir_name) || dir_name.starts_with('.')
}

/// Check if a file has a supported extension for TODO scanning
fn is_supported_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SOURCE_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

/// Scan for source files that may contain TODO comments
pub fn scan_source_files(config: &ScanConfig) -> TodoResult<Vec<PathBuf>> {
    let mut files = Vec::new();

    let walker = WalkDir::new(&config.root_path)
        .follow_links(config.follow_links)
        .max_depth(config.max_depth.unwrap_or(usize::MAX))
        .into_iter()
        .filter_map(|entry| entry.ok()) // Skip entries with errors
        .filter(|entry| {
            // Only process files, not directories
            if !entry.file_type().is_file() {
                return false;
            }

            // Check if this is a supported source file
            if !is_supported_source_file(entry.path()) {
                return false;
            }

            // Check if any parent directory should be excluded
            for ancestor in entry.path().ancestors().skip(1) {
                if let Some(dir_name) = ancestor.file_name().and_then(|n| n.to_str()) {
                    if should_exclude_dir(dir_name) {
                        return false;
                    }
                }
            }

            true
        })
        .map(|entry| entry.path().to_path_buf());

    files.extend(walker);

    // Sort for deterministic output
    files.sort();

    Ok(files)
}

/// Combined scan that returns both markdown and source files
pub fn scan_all_files(config: &ScanConfig) -> TodoResult<(Vec<PathBuf>, Vec<PathBuf>)> {
    let markdown_files = scan_markdown_files(config)?;
    let source_files = scan_source_files(config)?;
    Ok((markdown_files, source_files))
}
