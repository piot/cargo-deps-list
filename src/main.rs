use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand, Node};
use clap::{arg, Parser};
use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// A Cargo subcommand to list dependencies in a project or workspace and execute commands on them.
#[derive(Parser, Debug)]
#[command(
    name = "cargo-deps-list",
    version,
    about = "Lists dependencies for Cargo projects and executes specified commands on each dependency.",
    long_about = None,
)]
struct Args {
    /// Show only dependencies within the workspace
    #[arg(long)]
    workspace_only: bool,

    /// Command to execute for each dependency. Use '{}', '{version}' and '{path}' to replace with the name, version and path of the dependency.
    #[arg(long, value_name = "COMMAND")]
    exec: Option<String>,

    /// Number of seconds to wait between executing commands for each dependency.
    #[arg(long, value_name = "SECONDS", value_parser = clap::value_parser!(u64).range(0..))]
    wait: Option<u64>,
}

struct Dependency {
    name: String,
    version: String,
    path: std::path::PathBuf,
}

// Function to recursively visit dependencies and order them leaf-first
fn visit_dep<'a>(
    node: &'a Node,
    dep_graph: &HashMap<&'a str, &'a Node>,
    workspace_members: &HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
    output: &mut Vec<&'a str>,
) {
    if visited.contains(node.id.repr.as_str()) {
        return;
    }

    if !workspace_members.contains(node.id.repr.as_str()) {
        return;
    }

    visited.insert(node.id.repr.as_str());

    // Visit all its dependencies (children)
    for dep in &node.deps {
        if let Some(dep_node) = dep_graph.get(dep.pkg.repr.as_str()) {
            visit_dep(dep_node, dep_graph, workspace_members, visited, output);
        }
    }

    output.push(node.id.repr.as_str());
}

fn list_dependencies(metadata: &Metadata, workspace_only: bool) -> Vec<Dependency> {
    let packages: HashSet<&str> = if workspace_only {
        metadata
            .workspace_members
            .iter()
            .map(|id| id.repr.as_str())
            .collect()
    } else {
        metadata
            .packages
            .iter()
            .map(|pkg| pkg.id.repr.as_str())
            .collect()
    };

    let resolve = metadata
        .resolve
        .as_ref()
        .expect("Failed to resolve dependencies");

    let dep_graph: HashMap<_, _> = resolve
        .nodes
        .iter()
        .map(|node| (node.id.repr.as_str(), node))
        .collect();

    let mut visited = HashSet::new();
    let mut output = Vec::new();

    for package in &metadata.packages {
        // If workspace_only is set, skip packages not in the workspace
        if workspace_only && !packages.contains(package.id.repr.as_str()) {
            continue;
        }

        if let Some(root_node) = dep_graph.get(package.id.repr.as_str()) {
            visit_dep(root_node, &dep_graph, &packages, &mut visited, &mut output);
        }
    }

    output
        .into_iter()
        .filter_map(|pkg_id| {
            metadata
                .packages
                .iter()
                .find(|pkg| {
                    pkg.id.repr == pkg_id
                        && (!workspace_only || packages.contains(pkg.id.repr.as_str()))
                })
                .map(|pkg| Dependency {
                    name: pkg.name.clone(),
                    version: pkg.version.to_string(),
                    path: pkg.manifest_path.parent().unwrap().to_path_buf().into(),
                })
        })
        .collect()
}

fn execute_command(command_template: &str, dependency: &Dependency) -> Result<()> {
    // Replace additional placeholders as needed
    let command_str = command_template
        .replace("{}", &dependency.name)
        .replace("{version}", &dependency.version)
        .replace("{path}", dependency.path.to_str().unwrap_or(""));

    // Determine the shell based on the OS
    #[cfg(target_family = "unix")]
    let shell = "sh";
    #[cfg(target_family = "unix")]
    let shell_arg = "-c";

    #[cfg(target_family = "windows")]
    let shell = "cmd";
    #[cfg(target_family = "windows")]
    let shell_arg = "/C";

    let status = Command::new(shell)
        .arg(shell_arg)
        .arg(&command_str)
        .current_dir(&dependency.path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to execute command: {command_str}"))?;

    if !status.success() {
        anyhow::bail!("Command '{command_str}' exited with status {status}");
    }

    Ok(())
}

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();
    let workspace_only = args.workspace_only;
    let exec_command = args.exec;
    let wait_seconds = args.wait;

    let metadata = MetadataCommand::new()
        .exec()
        .context("Failed to retrieve cargo metadata")?;

    // Get the dependencies in leaf-first order
    let dependencies = list_dependencies(&metadata, workspace_only);

    // Print out the dependencies in the required order
    println!("Dependencies in leaf-first order:");
    for dep in &dependencies {
        println!("{}", dep.name);
    }

    // If exec_command is provided, execute it for each dependency
    if let Some(command) = exec_command {
        for dep in &dependencies {
            println!("# Executing command for dependency '{}':", dep.name);
            if let Err(e) = execute_command(&command, dep) {
                eprintln!("Error executing command for '{}': {}", dep.name, e);
            }

            // If wait_seconds is specified and greater than 0, sleep for the given duration
            if let Some(seconds) = wait_seconds {
                if seconds > 0 {
                    println!("Waiting for {seconds} seconds before next command...");
                    thread::sleep(Duration::from_secs(seconds));
                }
            }
        }
    }

    Ok(())
}
