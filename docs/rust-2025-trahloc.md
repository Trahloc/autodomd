# Rust Project Layout: rust-2026-Trahloc Microcrate Convention (r2026t)

## 0. Scope

The rust-2026-Trahloc Microcrate Convention (r2026t) defines a strict directory, naming, and architectural standard for Rust workspaces. It is designed exclusively to:

1. **Eliminate File Ambiguity**: Prevent AI context hallucinations caused by multiple files named `lib.rs`, `main.rs`, or `mod.rs`.
2. **Optimize Compilation**: Enforce boundaries that keep incremental builds under 2 seconds.
3. **Structure for AI Agents**: Provide a deterministic map where file location equals logical responsibility.
4. **Minimize Path Noise**: Flatten microcrate structure by eliminating unnecessary `src/` directories.

This is a **layout, naming, and architecture specification only**. It does not change Rust semantics, editions, or compiler behavior. It layers on top of Rust 2021 and later editions.

---

## 1. Core Principles

1. **Uniqueness by Top-Level Directory (TLD)**  
   For every logical component X, there is exactly one directory `X/` containing exactly one crate whose root source file is `X.rs`.

2. **Banned Filenames**  
   - `mod.rs` is **strictly forbidden**. Use the Rust 2018+ pattern only.  
   - `lib.rs` and `main.rs` are **strictly forbidden** in all contexts. Crate roots must use explicit `X.rs` naming.

3. **Flattened Microcrates**  
   All non-binary microcrates omit the `src/` directory. `Cargo.toml` lives alongside the root `X.rs` file. The `src/` directory is reserved for the binary crate only.

4. **One Binary Front Door**  
   Exactly one primary binary crate under `bin/`, responsible for the main executable. All other crates are libraries accessed via optional dependencies and feature flags.

5. **Strict Semantic Naming**  
   Names are maximally descriptive for zero-inference AI comprehension.

6. **Tooling Compatibility**  
   Tools must respect explicit `path` fields in `Cargo.toml`. No redirects or legacy filenames are permitted.

---

## 2. Workspace Structure

```
dotsetup/
├── Cargo.toml               # workspace root
├── bin/                     # binary crate (uses src/)
├── command/                 # command microcrates (flattened)
├── library/                 # shared library microcrates (flattened)
├── tests/                   # integration test crate (flattened)
└── docs/                    # documentation, specs
```

### 2.1 Workspace Cargo.toml

```toml
[workspace]
members = [
    "bin",
    "command/*",
    "library/*",
    "tests",
]
resolver = "2"

[profile.dev]
opt-level = 0
debug = true
incremental = true

[profile.dev.package."*"]
opt-level = 2  # optimize dependencies for faster iteration
```

---

## 3. Crate Naming and Directory Rules

### 3.1 Package Name Convention

Package name format: `<project>-<category>-<responsibility>`

- `<project>`: base project name (e.g., `dotsetup`)
- `<category>`: one of `binary`, `command`, `library`, `tests-integration`
- `<responsibility>`: the directory name (TLD)

Examples:
- `bin/` → `dotsetup-binary`
- `command/install/` → `dotsetup-command-install`
- `library/common/` → `dotsetup-library-common`
- `tests/` → `dotsetup-tests-integration`

### 3.2 Root Source File Rule

For any crate directory `X/`:
- The crate root **must** be named `X.rs`
- `Cargo.toml` **must** explicitly declare the path

### 3.3 Flattened Layout (Microcrates)

```text
command/install/
├── Cargo.toml
├── install.rs           # crate root + orchestrator
├── install_config.rs
└── install_steps.rs
```

### 3.4 Binary Crate Layout

```text
bin/
├── Cargo.toml
└── src/
    └── dotsetup.rs      # orchestrator (no main.rs)
```

### 3.5 Naming Restrictions

- Never name a microcrate `core` (shadows `std::core`). Use `common` or `shared`.
- Never use `mod.rs`.

---

## 4. Command Microcrate Layout

All CLI verbs live under `command/`.

### 4.1 Example

```text
command/install/
├── Cargo.toml                  # name = "dotsetup-command-install"
├── install.rs                  # root + orchestrator
├── install_config.rs
└── install_steps.rs
```

### 4.2 Orchestrator Pattern

```rust
// install.rs

mod install_config;
mod install_steps;

pub use install_config::Config;
pub use install_steps::execute;

pub fn run(config: Config) -> Result<(), Error> {
    // orchestration only
}
```

---

## 5. Library Microcrate Layout

Shared logic lives under `library/`.

```text
library/common/
├── Cargo.toml                  # name = "dotsetup-library-common"
├── common.rs                   # root
├── fs_utils.rs
├── xdg.rs
└── symlink.rs
```

Code moves here when needed by ≥2 other crates.

---

## 6. Binary Crate Layout

### 6.1 Structure

```text
bin/
├── Cargo.toml
└── src/
    └── dotsetup.rs             # orchestrator + entry point
```

### 6.2 Cargo.toml Example

```toml
[package]
name = "dotsetup-binary"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "dotsetup"
path = "src/dotsetup.rs"

[features]
default = ["install", "fix", "status"]
install = ["dotsetup-command-install"]
fix = ["dotsetup-command-fix"]
status = ["dotsetup-command-status"]
full = ["install", "fix", "status"]

[dependencies]
dotsetup-command-install = { path = "../command/install", optional = true }
dotsetup-command-fix = { path = "../command/fix", optional = true }
dotsetup-command-status = { path = "../command/status", optional = true }
dotsetup-library-common = { path = "../library/common" }
```

### 6.3 Orchestrator Example

```rust
// src/dotsetup.rs

use clap::Parser;

#[derive(Parser)]
enum Command {
    Install(dotsetup_command_install::Config),
    Fix(dotsetup_command_fix::Config),
    Status(dotsetup_command_status::Config),
}

fn main() {
    let cmd = Command::parse();
    match cmd {
        Command::Install(cfg) => dotsetup_command_install::run(cfg),
        Command::Fix(cfg) => dotsetup_command_fix::run(cfg),
        Command::Status(cfg) => dotsetup_command_status::run(cfg),
    }
}
```

---

## 7. Testing Strategy

### 7.1 Unit Tests (In-File)

Live inside the source file they test.

Benefits:
- AI sees code + tests in one context window
- Tests compile only when that crate is targeted
- Private functions testable without extra visibility

### 7.2 Integration Tests (Dedicated Crate)

`tests/` is a single crate (not Cargo's default per-file binaries) for fast compilation.

```text
tests/
├── Cargo.toml                      # name = "dotsetup-tests-integration"
├── integration.rs                  # root
├── install_flow.rs
└── fix_flow.rs
```

---

## 8. When to Split: The Microcrate Threshold

Split when **any** of:
1. ≥3 non-trivial public types/traits
2. Unique dependency isolation
3. Divergent test fixtures/setup
4. Independent versioning/feature needs
5. Clear collaboration boundary (multiple AI agents working simultaneously)

Goal: one file change → rebuild one crate → test in <2 seconds.

---

## 9. Nested Microcrates

Allowed for implementation details.

```text
command/install/
└── migration/
    ├── Cargo.toml                  # dotsetup-command-install-migration
    └── migration.rs
```

Promote to top-level `library/` when cross-referenced or conceptually independent.

---

## 10. Feature Flag Strategy

- Leaf crates own local optional features
- Binary crate aggregates and exposes user-facing features
- Features flow downward only

---

## 11. Iterative Development Tooling

### 11.1 Recommended Tools

- `cargo-watch` for continuous checking/testing
- `sccache` for cache
- `mold` linker (Linux) for speed

### 11.2 Workflow

1. `cargo watch -x check`
2. Edit → instant feedback
3. Targeted tests per crate
4. `cargo run -- <command>` for full binary

Microcrate boundaries ensure near-instant incremental builds.

---

## 12. The Uniqueness Invariant

> For every crate directory `X/` under the workspace, there exists exactly one crate whose root module is `X.rs` (flattened) or `src/dotsetup.rs` (binary only). The files `lib.rs`, `main.rs`, and `mod.rs` are strictly forbidden in all contexts.

---

## 13. Complete Directory Tree Reference

```
dotsetup/
├── Cargo.toml
├── docs/
│   └── r2026t-spec.md
├── bin/
│   ├── Cargo.toml                  # dotsetup-binary
│   └── src/
│       └── dotsetup.rs
├── command/
│   ├── install/
│   │   ├── Cargo.toml              # dotsetup-command-install
│   │   ├── install.rs
│   │   ├── install_config.rs
│   │   ├── install_steps.rs
│   │   └── migration/
│   │       ├── Cargo.toml          # dotsetup-command-install-migration
│   │       └── migration.rs
│   ├── fix/
│   │   ├── Cargo.toml              # dotsetup-command-fix
│   │   └── fix.rs
│   └── status/
│       ├── Cargo.toml              # dotsetup-command-status
│       └── status.rs
├── library/
│   └── common/
│       ├── Cargo.toml              # dotsetup-library-common
│       ├── common.rs
│       ├── fs_utils.rs
│       ├── xdg.rs
│       └── symlink.rs
└── tests/
    ├── Cargo.toml                  # dotsetup-tests-integration
    ├── integration.rs
    ├── install_flow.rs
    └── fix_flow.rs
```

---

## 14. Summary

**rust-2026-Trahloc (r2026t)** delivers:
1. **Unique Names**: Every crate root is `X.rs`. No `lib.rs`, `main.rs`, or `mod.rs`.
2. **Flattened Microcrates**: No `src/` except in the binary crate.
3. **Dedicated Test Crate**: Single integration crate for speed.
4. **AI-First Optimization**: Zero ambiguity, deterministic mapping, maximal descriptive naming.
5. **Compilation Speed**: Sub-2-second incremental rebuilds via strict boundaries.
6. **Single Binary**: Clap-based dispatch with optional command crates.

This convention trades Cargo defaults for extreme precision, compilation speed, and perfect AI-agent compatibility.