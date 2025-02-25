use crate::dependency_parsing::get_dependencies_in_file;
use std::path::Path;
use syn::File;
use toml::Value;

pub struct RustFile {
    pub path: String,
    pub module_name: String,
    pub crate_name: String,
    pub logical_path: String,
    pub dependencies: Vec<String>,
    pub ast: File,
}

impl RustFile {
    pub fn from_file_system(path: &str) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => panic!("Failed to read file file://{}: {}", path, e),
        };

        let binding = parse_module_logical_path(path)
            .expect(&format!("Failed to compute module path {path}"));
        let logical_path = binding.as_str();

        Self::from_content(path, logical_path, &content)
    }

    pub fn from_content(path: &str, logical_path: &str, content: &str) -> Self {
        let ast = match syn::parse_str(&content) {
            Ok(ast) => ast,
            Err(e) => panic!("Failed to parse file file://{}: {}", path, e),
        };

        Self::from_ast(path, logical_path, ast)
    }

    pub fn from_ast(path: &str, logical_path: &str, ast: File) -> Self {
        let module_name = logical_path.split("::").last().unwrap_or("").to_string();
        let crate_name = logical_path.split("::").next().unwrap_or("").to_string();
        let dependencies = get_dependencies_in_file(&logical_path, &ast);

        RustFile {
            path: path.to_string(),
            logical_path: logical_path.to_string(),
            module_name,
            crate_name,
            dependencies,
            ast,
        }
    }
}

fn parse_module_logical_path(file_path: &str) -> Result<String, String> {
    let path = Path::new(file_path);

    if path.is_dir() {
        return Err(format!(
            "The specified path '{}' is a directory, not a file",
            file_path
        ));
    }

    if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return Err(format!(
            "Invalid file type: expected a Rust file (.rs), found '{}'",
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown")
        ));
    }

    let crate_root = path
        .ancestors()
        .find(|ancestor| ancestor.join("Cargo.toml").exists())
        .ok_or_else(|| format!("File is not part of a Rust crate: {}", file_path))?;

    let cargo_toml_path = crate_root.join("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path).map_err(|_| {
        format!(
            "Failed to read Cargo.toml in '{}'",
            cargo_toml_path.display()
        )
    })?;
    let crate_name = toml::from_str::<Value>(&cargo_toml_content)
        .and_then(|parsed| {
            parsed
                .get("package")
                .and_then(|pkg| pkg.get("name"))
                .and_then(|name| name.as_str())
                .map(str::to_string)
                .ok_or_else(|| serde::de::Error::custom("Missing 'package.name' in Cargo.toml"))
        })
        .map_err(|err| format!("Failed to parse crate name: {}", err))?;

    let relative_path = path.strip_prefix(crate_root).map_err(|_| {
        format!(
            "Failed to compute relative path for file '{}' in crate '{}'",
            file_path,
            crate_root.display()
        )
    })?;

    let mut comps = relative_path.components().peekable();

    if comps.clone().any(|c| c.as_os_str() == "src") {
        while let Some(c) = comps.next() {
            if c.as_os_str() == "src" {
                break;
            }
        }
    }

    let mut parts = vec![];
    for comp in comps {
        let s = comp.as_os_str().to_str().unwrap_or_default();
        parts.push(s.to_string());
    }

    if let Some(last) = parts.last_mut() {
        if last.ends_with(".rs") {
            *last = last.trim_end_matches(".rs").to_string();
        }
    }

    if parts.is_empty() {
        return Err(format!(
            "Failed to determine module path for '{}'",
            file_path
        ));
    }

    let module_path = parts.join("::");
    Ok(format!("{}::{}", crate_name, module_path))
}

#[cfg(test)]
mod tests {
    use crate::rust_file::{parse_module_logical_path, RustFile};

    #[test]
    fn test_rust_file_from_path() {
        let file = RustFile::from_file_system(file!());

        assert_eq!(file.path, "src/rust_file.rs".to_string());
        assert_eq!(file.logical_path, "rust_arkitect::rust_file".to_string());
        assert_eq!(file.crate_name, "rust_arkitect".to_string());
        assert_eq!(file.module_name, "rust_file".to_string());
    }

    #[test]
    fn test_get_module() {
        let module =
            parse_module_logical_path("./examples/workspace_project/conversion/src/application.rs")
                .unwrap();

        assert_eq!(module, "conversion::application")
    }

    #[test]
    fn test_get_module_on_a_random_file() {
        let module = parse_module_logical_path("./examples/workspace_project/assets/file_1.txt");

        assert_eq!(
            module,
            Err("Invalid file type: expected a Rust file (.rs), found 'txt'".to_string())
        );
    }

    #[test]
    fn test_get_module_with_a_file_in_folder_without_src() {
        let module = parse_module_logical_path("tests/test_architecture.rs");

        assert_eq!("rust_arkitect::tests::test_architecture", module.unwrap());
    }

    #[test]
    fn test_get_module_on_a_directory() {
        assert_eq!(
            parse_module_logical_path("./examples/workspace_project/"),
            Err(String::from(
                "The specified path './examples/workspace_project/' is a directory, not a file"
            ))
        );
    }
}
