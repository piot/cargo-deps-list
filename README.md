# 🚀 Cargo Deps Order 🦀

![Crates.io](https://img.shields.io/crates/v/cargo-deps-list.svg)
![License](https://img.shields.io/crates/l/cargo-deps-list.svg)
![Downloads](https://img.shields.io/crates/d/cargo-deps-list.svg)

**Cargo Deps Order** is a powerful Cargo subcommand designed to **list** all dependencies of your Rust project or workspace and **execute custom commands** on each dependency seamlessly. Whether you're managing a large workspace or a single project, this utility streamlines your workflow with ease and efficiency.

## 📖 Features

- **List Dependencies**: Display all dependencies of your Cargo project or workspace in a clear, leaf-first order.
- **Execute Custom Commands**: Run any shell command on each dependency with dynamic placeholders.
- **Wait Between Commands**: Specify a pause between command executions to manage resource usage or pacing.

## 🛠 Installation

### 📦 Using `cargo install`

You can easily install `cargo-deps-list` using Cargo's install command:

```bash
cargo install cargo-deps-order
```

## 📋 Available Options

- `--workspace-only`
  **Description:** Show only dependencies within the workspace.
  **Usage:**

```bash
cargo deps-order --workspace-only
```

- `--exec <COMMAND>`
**Description:** Command to execute for each dependency. Use {} as a placeholder for the dependency name and {version} for the dependency version.
**Usage:**

```bash
cargo deps-order --exec "echo {} version {version}"
```

- `--wait <SECONDS>`
**Description:** Number of seconds to wait between executing commands for each dependency.
**Usage:**

```bash
cargo deps-order --exec "echo {}" --wait 2
```

- `-h, --help`
**Description:** Print help information.
**Usage:**

```bash
cargo deps-order --help
```

- `-V, --version`
**Description:** Print version information.
**Usage:**

```bash
cargo deps-order --version
```
