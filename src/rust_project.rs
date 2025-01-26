use crate::rust_file::RustFile;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use walkdir::WalkDir;
pub struct RustProject {
    pub files: Vec<RustFile>,
}

impl RustProject {
    pub fn from_directory(root_dir: &str) -> Result<Self, Box<dyn Error>> {
        let mut rust_files = Vec::new();

        for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if is_rust_file(path) {
                let path_str = path.to_string_lossy().to_string();
                let rust_file = RustFile::from_file_system(&path_str);
                rust_files.push(rust_file);
            }
        }

        if rust_files.is_empty() {
            panic!("No rust files found");
        }

        Ok(Self { files: rust_files })
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

fn is_rust_file(path: &Path) -> bool {
    path.extension().map(|ext| ext == "rs").unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use crate::rust_project::RustProject;

    #[test]
    fn test_rust_project_from_directory() {
        // Supponendo di avere una cartella di fixture di test con qualche file .rs
        // ad esempio: tests/fixtures/simple_project/
        let project_dir = "/Users/patrickfazzi/Projects/rust_arkitect/examples/workspace_project";

        let project = RustProject::from_directory(project_dir)
            .expect("Should scan directory and build RustProject");

        // Stampiamo quanti file ha trovato
        println!("Found {} Rust files", project.files.len());
        for f in &project.files {
            println!(" - {} -> logical_path = {}", f.path, f.logical_path);
            // E pure le dipendenze
            println!("   dependencies: {:?}", f.dependencies);
        }

        // Creiamo il grafo e verifichiamo che contenga tante entry
        let graph = project.to_dependency_graph();
        assert!(graph.len() > 0);
        assert_eq!(graph.len(), project.files.len());
    }
}
