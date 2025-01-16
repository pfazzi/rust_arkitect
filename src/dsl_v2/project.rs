use crate::engine::Engine;
use crate::rules::may_depend_on::MayDependOnRule;
use crate::rules::must_not_depend_on_anything::MustNotDependOnAnythingRule;
use crate::rules::rule::Rule;
use std::collections::HashMap;
use std::env;
use std::marker::PhantomData;
use std::path::Path;

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

        Project {
            project_root: cargo_manifest_dir,
        }
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
