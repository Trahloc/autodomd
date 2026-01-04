# Implement Configuration File Support

**Priority:** Medium
**Dependencies:** Core Auto-Todo Functionality
**Estimated Effort:** High (new library dependency needed)

## Overview
Add support for configuration files to customize autodomd behavior without command-line options.

## Requirements
- Support `.autodomd.toml` or `.autodomd.json` configuration files
- Configure scan paths, file types, and categories
- Allow custom output formatting options
- Provide sensible defaults when no config file exists
- Support project-specific and user-global configurations

## Dependencies
- ✅ Core scanning and parsing functionality must be complete
- ❌ No other tasks depend on this one

## Implementation Notes
- Use a standard format like TOML for configuration (add `toml` crate)
- Allow configuration inheritance (project overrides user defaults)
- Validate configuration on load with helpful error messages
- Document all available configuration options

## Success Criteria
- `autodomd --config custom.toml scan` works
- Configuration file validation with helpful errors
- Project config overrides user defaults
- All CLI options can be set via config file
