//! Generate command for autodomd
//!
//! This microcrate implements the generate subcommand that creates the TODO.md file
//! from scanned TODO items and tasks.

use std::path::PathBuf;

use autodomd_library_common::{TaskCollection, TodoResult};
use autodomd_library_generator::{generate_todo_md, GeneratorConfig};

/// Configuration for the generate command
#[derive(Debug, Clone)]
pub struct GenerateCommandConfig {
    /// Output file path (defaults to "TODO.md")
    pub output_path: Option<PathBuf>,
    /// Project root path for relative paths
    pub root_path: Option<PathBuf>,
    /// Whether to include auto-generated header
    pub include_header: bool,
    /// Whether to output verbose information
    pub verbose: bool,
}

impl Default for GenerateCommandConfig {
    fn default() -> Self {
        Self {
            output_path: Some(PathBuf::from("TODO.md")),
            root_path: Some(PathBuf::from(".")),
            include_header: true,
            verbose: false,
        }
    }
}

impl GenerateCommandConfig {
    /// Convert to GeneratorConfig for the generator library
    fn to_generator_config(&self) -> GeneratorConfig {
        GeneratorConfig {
            output_path: self.output_path.clone().unwrap_or_else(|| PathBuf::from("TODO.md")),
            root_path: self.root_path.clone().unwrap_or_else(|| PathBuf::from(".")),
            include_header: self.include_header,
        }
    }
}

/// Result of a generate operation
#[derive(Debug)]
pub struct GenerateResult {
    /// Path to the generated file
    pub output_path: PathBuf,
    /// Number of tasks written
    pub tasks_written: usize,
}

/// Execute the generate command
pub fn run_generate(tasks: &TaskCollection, config: &GenerateCommandConfig) -> TodoResult<GenerateResult> {
    if config.verbose {
        println!("Generating TODO.md...");
        println!("Tasks to process: {}", tasks.len());
        println!("Output path: {}", config.to_generator_config().output_path.display());
    }

    // Generate the TODO.md file
    let generator_config = config.to_generator_config();
    generate_todo_md(tasks, &generator_config)?;

    let result = GenerateResult {
        output_path: generator_config.output_path,
        tasks_written: tasks.len(),
    };

    if config.verbose {
        println!("Generation complete:");
        println!("  Output file: {}", result.output_path.display());
        println!("  Tasks written: {}", result.tasks_written);
    }

    Ok(result)
}
