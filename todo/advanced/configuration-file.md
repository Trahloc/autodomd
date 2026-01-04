# Implement Configuration File Support

## Overview
Add support for configuration files to customize autodomd behavior without command-line options.

## Requirements
- Support `.autodomd.toml` or `.autodomd.json` configuration files
- Configure scan paths, file types, and categories
- Allow custom output formatting options
- Provide sensible defaults when no config file exists
- Support project-specific and user-global configurations

## Implementation Notes
- Use a standard format like TOML for configuration
- Allow configuration inheritance (project overrides user defaults)
- Validate configuration on load with helpful error messages
- Document all available configuration options
