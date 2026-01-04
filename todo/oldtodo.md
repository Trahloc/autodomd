# AutoDomd Implementation Plan (Rust Edition)

**Objective:** Create a Rust-based auto-todo system that aggregates TODOs from source code and markdown files into a single, read-only `TODO.md`.

**Architecture:** Following r2026t microcrate convention with flattened structure and explicit naming.

## Phase 1: Core Data Structures & Types

### 1.1 Define Core Types (`library/common/`)
- [x] Create `Task` struct for individual TODO items
- [x] Create `TaskSource` enum (Code, Markdown)
- [x] Create `TaskLocation` struct for file/line references
- [x] Create `TaskCategory` enum for grouping
- [x] Add error types (`TodoError`) with proper error handling

### 1.2 Define Collection Types
- [x] Create `TaskCollection` for managing multiple tasks
- [x] Add methods for filtering, sorting, grouping tasks
- [x] Implement `Display` traits for markdown output

## Phase 2: File Scanning (`library/scanner/`)

### 2.1 Markdown File Scanner
- [x] Implement recursive directory traversal for `todo/` folders
- [x] Add file filtering (`.md` files only)
- [x] Implement deterministic sorting (alphabetical by filename)
- [x] Return `Vec<PathBuf>` of markdown files

### 2.2 Source Code File Scanner
- [x] Scan relevant directories (`src/`, `bin/`, `command/`, `library/`, etc.)
- [x] Filter by supported file extensions (`.rs`, `.js`, `.ts`, `.py`, etc.)
- [x] Implement exclusion patterns (target/, .git/, etc.)
- [x] Return `Vec<PathBuf>` of source files

## Phase 3: Content Parsing (`library/parser/`)

### 3.1 Markdown Task Parser
- [x] Parse H1 headers from markdown files
- [x] Extract task titles and descriptions
- [x] Handle fallback to filename when no H1 found
- [x] Generate relative paths for links
- [x] Return `Vec<Task>` from markdown sources

### 3.2 Source Code Comment Parser
- [x] Parse TODO comments with categories: `// TODO(Category): Description`
- [x] Support multiple comment styles (`//`, `/* */`, `#`, etc.)
- [x] Extract line numbers and file locations
- [x] Parse category tags from parentheses
- [x] Return `Vec<Task>` from code sources

### 3.3 Language-Specific Parsers
- [x] Rust: `// TODO(Category):` and `/* TODO(Category): */`
- [x] JavaScript/TypeScript: `// TODO(Category):` and `/* TODO(Category): */`
- [x] Python: `# TODO(Category):`
- [x] Add extensible parser registry

## Phase 4: TODO.md Generation (`library/generator/`)

### 4.1 Output Formatting
- [x] Create standard markdown header with auto-generated warning
- [x] Format task items as `- [ ] [Title](link)` for markdown tasks
- [x] Format code tasks with file links and line numbers
- [x] Group tasks by category when available

### 4.2 File Assembly
- [x] Combine markdown and code tasks
- [x] Sort tasks deterministically
- [x] Handle empty sections gracefully
- [x] Write to `TODO.md` with proper encoding

## Phase 5: CLI Commands (`command/*/`)

### 5.1 Scan Command (`command/scan/`)
- [x] Parse CLI arguments (directory paths, file types, etc.)
- [x] Execute scanning phase
- [x] Return structured data for further processing

### 5.2 Generate Command (`command/generate/`)
- [x] Parse CLI arguments (output path, format options)
- [x] Execute parsing and generation phases
- [x] Write TODO.md file
- [x] Provide success/failure feedback

### 5.3 Init Command (`command/init/`)
- [x] Create `todo/` directory structure
- [x] Generate sample markdown task file
- [x] Add sample TODO comments to source files
- [x] Provide getting started instructions

## Phase 6: Binary Orchestrator (`bin/src/autodomd.rs`)

### 6.1 CLI Framework
- [ ] Choose and integrate CLI parsing library (clap recommended)
- [ ] Define subcommands (scan, generate, init)
- [ ] Parse global options (verbose, dry-run, etc.)

### 6.2 Command Dispatch
- [ ] Route to appropriate command microcrates
- [ ] Handle command results and error propagation
- [ ] Provide unified error handling and exit codes

## Phase 7: Testing & Validation (`tests/`)

### 7.1 Unit Tests
- [ ] Test each library microcrate individually
- [ ] Test parsing logic for various comment styles
- [ ] Test file scanning with different directory structures
- [ ] Test markdown generation formats

### 7.2 Integration Tests
- [ ] End-to-end testing of scan + generate workflow
- [ ] Test with real project structures
- [ ] Validate TODO.md output format
- [ ] Test error conditions and edge cases

### 7.3 Sample Data Creation
- [ ] Create `todo/sample-task.md` with proper format
- [ ] Add sample TODO comments to various source files
- [ ] Ensure validation examples work

## Phase 8: Git Integration & Automation

### 8.1 Git Hook Integration
- [ ] Create pre-commit hook that runs `autodomd generate`
- [ ] Automatically add updated TODO.md to commits
- [ ] Handle hook failures gracefully

### 8.2 CI/CD Integration
- [ ] Add TODO.md generation to GitHub Actions
- [ ] Validate that TODO.md is up-to-date in PRs
- [ ] Fail builds if TODO.md generation fails

## Phase 9: Advanced Features

### 9.1 Configuration File
- [ ] Support `.autodomd.toml` or `.autodomd.json`
- [ ] Configure scan paths, file types, categories
- [ ] Custom output formatting options

### 9.2 Incremental Updates
- [ ] Track file modification times
- [ ] Only re-scan modified files
- [ ] Cache parsing results for performance

### 9.3 Multiple Output Formats
- [ ] Support JSON output for integrations
- [ ] HTML output for web viewing
- [ ] Custom template support

## Phase 10: Documentation & Polish

### 10.1 README Updates
- [ ] Update with Rust-specific installation instructions
- [ ] Add usage examples for all commands
- [ ] Document configuration options

### 10.2 Error Messages
- [ ] Improve error messages with actionable suggestions
- [ ] Add help text for all CLI options
- [ ] Provide clear validation feedback

## Success Criteria

- [ ] `autodomd init` creates proper project structure
- [ ] `autodomd scan` finds all TODO comments and markdown tasks
- [ ] `autodomd generate` creates properly formatted TODO.md
- [ ] All tests pass with good coverage
- [ ] Git hooks work correctly
- [ ] CI/CD pipeline validates TODO.md generation
- [ ] Documentation is complete and accurate