# Implement CLI Interface and Script

## Overview
Create a command-line interface that provides easy access to the auto-todo functionality.

## Requirements
- `autodomd scan` - Scan project for TODO items
- `autodomd generate` - Generate TODO.md from scanned items
- `autodomd init` - Initialize project with sample files
- Support for configuration options (output paths, verbosity, etc.)
- Proper error handling and user feedback

## Implementation Notes
- Use clap for CLI argument parsing
- Provide clear help text and usage examples
- Support global options like `--verbose`
- Ensure commands can be chained together effectively
