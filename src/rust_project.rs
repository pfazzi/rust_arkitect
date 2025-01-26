use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;
use walkdir::WalkDir;

use crate::rust_file::RustFile;
pub struct RustProject {
    pub files: Vec<RustFile>,
}

impl RustProject {
    pub fn from_directory(root_dir: &str) -> Result<Self, Box<dyn Error>> {
        // 1. Troviamo e leggiamo il `Cargo.toml`
        let cargo_toml_path = Path::new(root_dir).join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err(format!("No `Cargo.toml` found in `{}`", root_dir).into());
        }

        let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;
        let cargo_toml: Value = toml::from_str(&cargo_toml_content)?;

        // 2. Determiniamo se è un crate singolo o un workspace
        let mut source_dirs = Vec::new();

        if let Some(workspace) = cargo_toml.get("workspace") {
            // È un workspace: troviamo i membri e analizziamo i loro `Cargo.toml`
            if let Some(members) = workspace.get("members") {
                for member in members
                    .as_array()
                    .ok_or("Invalid workspace.members format")?
                {
                    let member_path = Path::new(root_dir).join(member.as_str().unwrap());
                    source_dirs.push(Self::find_source_dir(&member_path)?);
                }
            }
        } else {
            // È un crate singolo: cerchiamo la directory sorgente
            source_dirs.push(Self::find_source_dir(Path::new(root_dir))?);
        }

        // 3. Cerchiamo tutti i file `.rs` nelle directory sorgenti
        let mut rust_files = Vec::new();
        for src_dir in source_dirs {
            for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if Self::is_rust_file(path) {
                    let path_str = path.to_string_lossy().to_string();
                    let rust_file = RustFile::from_file_system(&path_str);
                    rust_files.push(rust_file);
                }
            }
        }

        if rust_files.is_empty() {
            return Err("No Rust source files found.".into());
        }

        Ok(Self { files: rust_files })
    }

    /// Determina la directory sorgente di un crate leggendo il suo `Cargo.toml`.
    fn find_source_dir(crate_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
        let cargo_toml_path = crate_dir.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err(format!(
                "No `Cargo.toml` found in crate directory `{}`",
                crate_dir.display()
            )
            .into());
        }

        let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;
        let cargo_toml: Value = toml::from_str(&cargo_toml_content)?;

        // Per default, la directory sorgente è `src/`
        if let Some(package) = cargo_toml.get("package") {
            if let Some(metadata) = package.get("metadata") {
                if let Some(custom_source) = metadata.get("source") {
                    let custom_dir = crate_dir.join(custom_source.as_str().unwrap());
                    return Ok(custom_dir);
                }
            }
        }

        Ok(crate_dir.join("src")) // Default directory sorgente
    }

    fn is_rust_file(path: &Path) -> bool {
        path.extension().map(|ext| ext == "rs").unwrap_or(false)
    }

    /// TODO: fixme
    pub fn to_dependency_graph(&self) -> HashMap<String, Vec<String>> {
        let mut graph = HashMap::new();
        for f in &self.files {
            graph.insert(f.logical_path.clone(), f.dependencies.clone());
        }
        graph
    }
}

#[cfg(test)]
mod tests {
    use crate::rust_project::RustProject;

    #[test]
    fn test_rust_project_from_directory() {
        let project_dir = get_workspace_project_path();

        let project = RustProject::from_directory(&project_dir)
            .expect("Should scan directory and build RustProject");

        let graph = project.to_dependency_graph();

        assert!(graph.len() > 0);
        assert_eq!(graph.len(), project.files.len());
    }

    fn get_workspace_project_path() -> String {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let project_dir = current_dir.join("examples/workspace_project");
        let project_dir_str = project_dir
            .to_str()
            .expect("Failed to convert path to string");

        String::from(project_dir_str)
    }
}
