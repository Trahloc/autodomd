# Implement Core Auto-Todo Functionality

## Overview
Create the core functionality to scan source code and markdown files for TODO items and generate a consolidated TODO.md file.

## Requirements
- Parse TODO comments from source code with category support: `// TODO(Category): Description`
- Parse markdown task files with H1 headers as task titles
- Support multiple programming languages (Rust, JavaScript, Python, etc.)
- Generate deterministic, well-formatted TODO.md output
- Group tasks by category when specified

## Implementation Notes
- Use regex-based parsing for TODO comment extraction
- Support nested directory scanning with exclusion patterns
- Ensure deterministic output ordering
- Handle edge cases like malformed comments gracefully
