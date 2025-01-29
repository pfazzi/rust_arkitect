use std::path::Path;
use std::{env, fs};

pub struct Project {
    pub project_root: String,
}

impl Project {
    pub fn from_path(absolute_path: &str) -> Project {
        let path = Path::new(absolute_path);
        if !path.exists() {
            panic!("The provided path '{}' does not exist.", absolute_path);
        }

        let cargo_toml_path = path.join("Cargo.toml");

        if !cargo_toml_path.exists() {
            panic!(
                "The provided path '{}' does not contain a `Cargo.toml` file.",
                absolute_path
            );
        }

        if let Ok(contents) = fs::read_to_string(&cargo_toml_path) {
            if !contents.contains("[package]") && !contents.contains("[workspace]") {
                panic!(
                    "`Cargo.toml` at '{}' is invalid: it must contain `[package]` or `[workspace]`.",
                    cargo_toml_path.display()
                );
            }
        } else {
            panic!(
                "Failed to read the `Cargo.toml` file at '{}'.",
                cargo_toml_path.display()
            );
        }

        Project {
            project_root: absolute_path.to_string(),
        }
    }

    /// Creates a Project rooted at the crate's directory.
    pub fn from_current_crate() -> Project {
        let cargo_manifest_dir =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

        Project {
            project_root: cargo_manifest_dir,
        }
    }

    /// Creates a Project rooted at the workspace's root directory.
    /// Panics if the current crate is not part of a workspace.
    pub fn from_current_workspace() -> Project {
        let cargo_manifest_dir =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

        let crate_path = Path::new(&cargo_manifest_dir);

        if Self::is_workspace_root(crate_path) {
            return Project {
                project_root: cargo_manifest_dir,
            };
        }

        if let Some(parent_path) = crate_path.parent() {
            if Self::is_workspace_root(parent_path) {
                return Project {
                    project_root: parent_path.to_string_lossy().into_owned(),
                };
            }
        }

        panic!("Current crate is not part of a workspace or workspace root not found.");
    }

    /// Method that creates a Project determining whether the current context is a workspace or crate.
    pub fn new() -> Project {
        let cargo_manifest_dir =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

        let crate_path = Path::new(&cargo_manifest_dir);

        if Self::is_workspace_root(crate_path) {
            return Self::from_current_workspace();
        }

        if let Some(parent_path) = crate_path.parent() {
            if Self::is_workspace_root(parent_path) {
                return Self::from_current_workspace();
            }
        }

        Self::from_current_crate()
    }

    /// Checks if the given path is a workspace root by inspecting its `Cargo.toml`.
    fn is_workspace_root(path: &Path) -> bool {
        let cargo_toml = path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return false;
        }

        if let Ok(contents) = fs::read_to_string(cargo_toml) {
            return contents.contains("[workspace]");
        }

        false
    }

    /// Creates a Project from a path relative to the given file.
    pub fn from_relative_path(current_file: &str, relative_path: &str) -> Project {
        let current_dir = Path::new(current_file)
            .parent()
            .expect("Failed to get parent directory");

        let derived_path = current_dir.join(relative_path);

        let absolute_path = derived_path.canonicalize().unwrap_or_else(|e| {
            panic!(
                "Failed to resolve absolute path:\n\
                 - Current file: '{}'\n\
                 - Relative path: '{}'\n\
                 - Derived path (before resolving): '{}'\n\
                 Cause: {}",
                current_file,
                relative_path,
                derived_path.display(),
                e
            )
        });

        Project {
            project_root: absolute_path
                .to_str()
                .expect("Failed to convert path to string")
                .to_string(),
        }
    }
}
