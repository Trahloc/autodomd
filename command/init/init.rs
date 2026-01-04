//! Init command for autodomd
//!
//! This microcrate implements the init subcommand that sets up autodomd
//! for a new project.

use std::fs;
use std::path::Path;

use autodomd_library_common::TodoResult;

/// Configuration for the init command
#[derive(Debug, Clone)]
pub struct InitCommandConfig {
    /// Whether to create sample tasks
    pub create_samples: bool,
    /// Whether to output verbose information
    pub verbose: bool,
}

impl Default for InitCommandConfig {
    fn default() -> Self {
        Self {
            create_samples: true,
            verbose: false,
        }
    }
}

/// Result of an init operation
#[derive(Debug)]
pub struct InitResult {
    /// Whether the todo directory was created
    pub todo_dir_created: bool,
    /// Sample files created
    pub sample_files_created: Vec<String>,
}

/// Execute the init command
pub fn run_init(config: &InitCommandConfig) -> TodoResult<InitResult> {
    if config.verbose {
        println!("Initializing autodomd project...");
    }

    let mut sample_files_created = Vec::new();
    let mut todo_dir_created = false;

    // Create todo directory if it doesn't exist
    if !Path::new("todo").exists() {
        fs::create_dir("todo")?;
        todo_dir_created = true;

        if config.verbose {
            println!("Created todo/ directory");
        }
    }

    // Create sample files if requested
    if config.create_samples {
        // Create sample markdown task
        let sample_task_path = "todo/sample-task.md";
        if !Path::new(sample_task_path).exists() {
            let sample_content = r#"# Implement User Authentication System

This is a sample task to demonstrate autodomd functionality.

## Requirements
- User registration with email validation
- Secure password hashing
- JWT token-based authentication
- Password reset functionality

## Implementation Notes
- Use argon2 for password hashing
- Implement rate limiting for auth endpoints
- Add proper error handling and validation
"#;

            fs::write(sample_task_path, sample_content)?;
            sample_files_created.push(sample_task_path.to_string());

            if config.verbose {
                println!("Created sample task: {}", sample_task_path);
            }
        }

        // Create sample TODO comments in source files
        let sample_source_files = [
            ("bin/src/autodomd.rs", "// TODO(Auth): Implement command-line argument parsing\n// TODO(UI): Add colored output for better user experience"),
            ("library/common/common.rs", "// TODO(Testing): Add comprehensive unit tests for TaskCollection\n// TODO(Docs): Add documentation examples"),
        ];

        for (file_path, content) in sample_source_files {
            if Path::new(file_path).exists() {
                let existing_content = fs::read_to_string(file_path)?;
                if !existing_content.contains("TODO") {
                    let new_content = format!("{}\n\n{}", existing_content, content);
                    fs::write(file_path, new_content)?;
                    sample_files_created.push(file_path.to_string());

                    if config.verbose {
                        println!("Added sample TODOs to: {}", file_path);
                    }
                }
            }
        }
    }

    if config.verbose {
        println!("Init complete!");
        if todo_dir_created {
            println!("  - Created todo/ directory");
        }
        if !sample_files_created.is_empty() {
            println!("  - Created {} sample files", sample_files_created.len());
        }
    }

    Ok(InitResult {
        todo_dir_created,
        sample_files_created,
    })
}
