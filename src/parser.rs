use quote::ToTokens;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use syn::{Item, ItemUse, UseTree};

pub fn parse_dependencies(path: &str) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => panic!("Failed to read file: {}", e),
    };

    let ast = match syn::parse_file(&content) {
        Ok(ast) => ast,
        Err(e) => panic!("Failed to parse file: {}", e),
    };

    let mut dependencies = Vec::new();

    for item in ast.items.iter() {
        if let Item::Use(ItemUse { tree, .. }) = item {
            collect_dependencies_from_tree(tree, &mut dependencies, "".to_string());
        }
    }

    dependencies
}

fn collect_dependencies_from_tree(tree: &UseTree, dependencies: &mut Vec<String>, prefix: String) {
    match tree {
        UseTree::Path(path) => {
            let ident = path.ident.to_string();
            let token = path.colon2_token.to_token_stream().to_string();
            if ident == "crate" {
                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    "crate".to_string(),
                );
            } else {
                let prefix: String = if !prefix.is_empty() {
                    format!("{}{}{}", prefix, token, ident)
                } else {
                    ident
                };
                collect_dependencies_from_tree(path.tree.deref(), dependencies, prefix);
            }
        }
        UseTree::Group(group) => {
            for item in group.items.iter() {
                let ident = item.to_token_stream().to_string();
                let dep = format!("{}{}{}", prefix, "::", ident);
                dependencies.push(dep);
            }
        }
        UseTree::Name(name) => {
            let ident = name.ident.to_string();
            let dep = format!("{}{}{}", prefix, "::", ident);
            dependencies.push(dep);
        }
        UseTree::Glob(glob) => {
            panic!(
                "{} imports (e.g., `{}`) are not supported.",
                "Glob",
                glob.to_token_stream().to_string()
            );
        }
        UseTree::Rename(rename) => {
            panic!(
                "{} imports (e.g., `{}`) are not supported.",
                "Rename",
                rename.to_token_stream().to_string()
            );
        }
    }
}

pub fn get_module(file_path: &str) -> Result<String, String> {
    let path = Path::new(file_path);

    // Trova il percorso relativo a `src`
    let relative_path = path
        .components()
        .skip_while(|comp| comp.as_os_str() != "src")
        .skip(1) // Salta "src"
        .collect::<PathBuf>();

    if relative_path.as_os_str().is_empty() {
        return Err(format!(
            "Failed to find module path: prefix 'src' not found in {}",
            file_path
        ));
    }

    let mut without_extension = relative_path.with_extension("");

    // Gestisce file speciali come `mod.rs` e `lib.rs`
    if without_extension.file_name() == Some("mod".as_ref()) {
        without_extension = without_extension
            .parent()
            .ok_or_else(|| format!("Failed to find parent for mod.rs in {}", file_path))?
            .to_path_buf();
    } else if without_extension.file_name() == Some("lib".as_ref()) {
        return Ok("crate".to_string());
    }

    // Converte il percorso in formato modulo
    let module_path = without_extension
        .components()
        .filter_map(|comp| comp.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("::");

    Ok(format!("crate::{}", module_path))
}

#[test]
pub fn test_parsing() {
    let dependencies = parse_dependencies("./sample_project/src/conversion/application.rs");

    assert_eq!(
        dependencies,
        vec![
            "crate::conversion::domain::domain_function_1",
            "crate::conversion::domain::domain_function_2",
            "crate::conversion::infrastructure::infrastructure_function"
        ]
    );
}

#[test]
fn test_file_path() {
    assert_eq!(
        get_module(
            "/users/reandom/projects/rust_arkitect/sample_project/src/conversion/application.rs"
        ),
        Ok(String::from("crate::conversion::application"))
    );

    assert_eq!(
        get_module("/users/reandom/projects/rust_arkitect/sample_project/src/conversion"),
        Ok(String::from("crate::conversion"))
    );
}
