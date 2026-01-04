use clap::{Parser, Subcommand};

/// AutoDomd - Automated TODO management system
#[derive(Parser)]
#[command(name = "autodomd")]
#[command(about = "Scan source code and markdown files for TODO items and generate comprehensive task lists")]
#[command(version)]
struct Cli {
    /// Increase verbosity
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize autodomd for a new project
    Init {
        /// Skip creating sample files
        #[arg(long)]
        no_samples: bool,
    },
    /// Scan project for TODO items and tasks
    Scan {
        /// Root directory to scan (default: current directory)
        #[arg(short, long)]
        root: Option<std::path::PathBuf>,

        /// Follow symbolic links
        #[arg(long)]
        follow_links: bool,

        /// Maximum scan depth
        #[arg(long)]
        max_depth: Option<usize>,
    },
    /// Generate TODO.md from scanned items
    Generate {
        /// Output file path (default: TODO.md)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Skip auto-generated header
        #[arg(long)]
        no_header: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Set up global verbosity (could be improved with a logging crate)
    if cli.verbose {
        println!("autodomd v{}", env!("CARGO_PKG_VERSION"));
    }

    match cli.command {
        Commands::Init { no_samples } => {
            #[cfg(feature = "init")]
            {
                let config = autodomd_command_init::InitCommandConfig {
                    create_samples: !no_samples,
                    verbose: cli.verbose,
                };
                let result = autodomd_command_init::run_init(&config)?;
                if !cli.verbose {
                    println!("✅ Project initialized successfully!");
                    if result.todo_dir_created {
                        println!("  Created todo/ directory");
                    }
                    if !result.sample_files_created.is_empty() {
                        println!("  Created {} sample files", result.sample_files_created.len());
                    }
                }
            }
            #[cfg(not(feature = "init"))]
            {
                return Err("Init command not available - compiled without 'init' feature".into());
            }
        }
        Commands::Scan { root, follow_links, max_depth } => {
            #[cfg(feature = "scan")]
            {
                let config = autodomd_command_scan::ScanCommandConfig {
                    root_path: root,
                    follow_links,
                    max_depth,
                    verbose: cli.verbose,
                };
                let result = autodomd_command_scan::run_scan(&config)?;
                if !cli.verbose {
                    println!("✅ Scan complete!");
                    println!("  Found {} tasks in {} files", result.tasks_found,
                           result.markdown_files_scanned + result.source_files_scanned);
                }
            }
            #[cfg(not(feature = "scan"))]
            {
                return Err("Scan command not available - compiled without 'scan' feature".into());
            }
        }
        Commands::Generate { output, no_header } => {
            #[cfg(feature = "scan")]
            {
                // First scan for tasks
                let scan_config = autodomd_command_scan::ScanCommandConfig {
                    verbose: cli.verbose,
                    ..Default::default()
                };

                let scan_result = autodomd_command_scan::run_scan(&scan_config)?;

                #[cfg(feature = "generate")]
                {
                    let gen_config = autodomd_command_generate::GenerateCommandConfig {
                        output_path: output,
                        include_header: !no_header,
                        verbose: cli.verbose,
                        ..Default::default()
                    };

                    let result = autodomd_command_generate::run_generate(&scan_result.tasks, &gen_config)?;
                    if !cli.verbose {
                        println!("✅ TODO.md generated successfully!");
                        println!("  Created: {}", result.output_path.display());
                        println!("  Tasks documented: {}", result.tasks_written);
                    }
                }
                #[cfg(not(feature = "generate"))]
                {
                    return Err("Generate command not available - compiled without 'generate' feature".into());
                }
            }
            #[cfg(not(feature = "scan"))]
            {
                return Err("Generate command requires scan feature".into());
            }
        }
    };

    Ok(())
}

