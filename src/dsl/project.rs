use std::path::Path;
use std::{env, fs};

pub struct Project {
    pub project_root: String,
}

impl Project {
    pub fn from_path(absolute_path: &str) -> Project {
        Project {
            project_root: absolute_path.to_string(),
        }
    }

    pub fn new() -> Project {
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

        Project {
            project_root: cargo_manifest_dir,
        }
    }

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
