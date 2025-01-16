use quote::ToTokens;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use syn::{File, Item, ItemUse, UseTree};

pub fn get_dependencies_in_file(path: &str) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => panic!("Failed to read file file://{}: {}", path, e),
    };

    let ast = match syn::parse_file(&content) {
        Ok(ast) => ast,
        Err(e) => panic!("Failed to parse file file://{}: {}", path, e),
    };

    match get_module(path) {
        Ok(module) => get_dependencies_in_ast(ast, module),
        Err(_error) => vec![],
    }
}

fn parse_module_item(item: &Item, dependencies: &mut Vec<String>, current_module: String) {
    match item {
        Item::Use(ItemUse { tree, .. }) => {
            collect_dependencies_from_tree(
                tree,
                dependencies,
                current_module.clone(),
                "".to_string(),
            );
        }
        Item::Mod(mod_item) => {
            if let Some((_, items)) = &mod_item.content {
                let new_prefix = format!("{}::{}", current_module.clone(), mod_item.ident);
                for sub_item in items.iter() {
                    parse_module_item(sub_item, dependencies, current_module.clone());
                }
            }
        }
        _ => {}
    }
}

fn get_dependencies_in_str(s: &str, module: String) -> Vec<String> {
    let ast: File = match syn::parse_str(s) {
        Ok(ast) => ast,
        Err(e) => panic!("Failed to parse string '{}': {}", s, e),
    };

    get_dependencies_in_ast(ast, module)
}

fn get_dependencies_in_ast(ast: File, current_module: String) -> Vec<String> {
    let mut dependencies = Vec::new();

    for item in ast.items.iter() {
        match item {
            Item::Use(ItemUse { tree, .. }) => {
                collect_dependencies_from_tree(
                    tree,
                    &mut dependencies,
                    current_module.clone(),
                    "".to_string(),
                );
            }
            Item::Mod(mod_item) => {
                if let Some((_, items)) = &mod_item.content {
                    let ident = mod_item.ident.clone().to_string();
                    let module = format!("{}::{}", current_module, ident);
                    for sub_item in items.iter() {
                        parse_module_item(sub_item, &mut dependencies, module.clone());
                    }
                }
            }
            _ => {}
        }
    }

    dependencies
}

fn collect_dependencies_from_tree(
    tree: &UseTree,
    dependencies: &mut Vec<String>,
    current_module: String,
    prefix: String,
) {
    let crate_name = current_module.split("::").next().unwrap_or("").to_string();

    match tree {
        UseTree::Path(path) => {
            let ident = path.ident.to_string();
            if ident == "super" {
                let parent_prefix = current_module
                    .rsplitn(2, "::")
                    .nth(1)
                    .unwrap_or("")
                    .to_string();

                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    current_module,
                    parent_prefix.clone(),
                );
            } else if ident == "crate" {
                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    current_module,
                    crate_name.clone(),
                );
            } else {
                let ident = if !prefix.is_empty() {
                    format!("{}::{}", prefix.clone(), ident)
                } else {
                    ident.clone()
                };
                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    current_module,
                    ident,
                );
            }
        }
        UseTree::Group(group) => {
            for item in group.items.iter() {
                collect_dependencies_from_tree(
                    item,
                    dependencies,
                    current_module.clone(),
                    prefix.clone(),
                );
            }
        }
        UseTree::Name(name) => {
            let ident = name.ident.to_string();
            let dep = format!("{}::{}", prefix.clone(), ident);
            dependencies.push(dep);
        }
        UseTree::Glob(_) => {
            let dep = format!("{}::*", prefix);
            dependencies.push(dep);
        }
        UseTree::Rename(rename) => {
            let ident = rename.ident.to_string();
            dependencies.push(format!("{}::{}", prefix.clone(), ident));
        }
    }
}

pub fn get_module(file_path: &str) -> Result<String, String> {
    let path = Path::new(file_path);

    // Controlla se il file ha l'estensione .rs
    if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return Err(format!(
            "Invalid file type: expected a Rust file (.rs), found '{}'",
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown")
        ));
    }

    // Trova la directory radice del crate cercando il Cargo.toml
    let crate_root = path
        .ancestors()
        .find(|ancestor| ancestor.join("Cargo.toml").exists());

    if crate_root.is_none() {
        return Err(format!(
            "File is not part of a Rust crate: {}",
            file_path
        ));
    }

    let crate_root = crate_root.unwrap();

    // Controlla che la directory `src` esista
    if !crate_root.join("src").is_dir() {
        return Err(format!(
            "Rust crate '{}' does not have a 'src' directory",
            crate_root.display()
        ));
    }

    // Trova il percorso relativo alla directory del crate
    let relative_path = path.strip_prefix(crate_root).map_err(|_| {
        format!(
            "Failed to compute relative path for file '{}' in crate '{}'",
            file_path,
            crate_root.display()
        )
    })?;

    // Cerca `src` nella struttura del percorso
    let src_relative = relative_path
        .components()
        .skip_while(|comp| comp.as_os_str() != "src")
        .skip(1) // Salta "src"
        .collect::<PathBuf>();

    if src_relative.as_os_str().is_empty() {
        return Err(format!(
            "Failed to find module path: prefix 'src' not found in {}",
            file_path
        ));
    }

    let mut without_extension = src_relative.with_extension("");

    // Gestisce file speciali come `mod.rs` e `lib.rs`
    if without_extension.file_name() == Some("mod".as_ref()) {
        without_extension = without_extension
            .parent()
            .ok_or_else(|| format!("Failed to find parent for mod.rs in {}", file_path))?
            .to_path_buf();
    } else if without_extension.file_name() == Some("lib".as_ref()) {
        let crate_name = crate_root
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("Failed to determine crate name for {}", file_path))?;
        return Ok(crate_name.to_string());
    }

    // Converte il percorso in formato modulo
    let module_path = without_extension
        .components()
        .filter_map(|comp| comp.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("::");

    let crate_name = crate_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Failed to determine crate name for {}", file_path))?;

    Ok(format!("{}::{}", crate_name, module_path))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_module() {
        let module =
            get_module("./examples/workspace_project/conversion/src/application.rs").unwrap();

        assert_eq!(module, "conversion::application")
    }

    #[test]
    fn test_get_module_on_a_random_file() {
        let module =
            get_module("./examples/workspace_project/assets/file_1.txt");

        assert_eq!(module, Err("Invalid file type: expected a Rust file (.rs), found 'txt'".to_string()));
    }

    #[test]
    pub fn test_parsing() {
        let dependencies =
            get_dependencies_in_file("./examples/sample_project/src/conversion/application.rs");
        assert_eq!(
            dependencies,
            vec![
                "sample_project::conversion::domain::domain_function_1",
                "sample_project::conversion::domain::domain_function_2",
                "sample_project::conversion::domain::domain_function_2",
            ]
        );
    }

    #[test]
    pub fn test_workspace_parsing() {
        let dependencies =
            get_dependencies_in_file("./examples/workspace_project/conversion/src/application.rs");
        assert_eq!(
            dependencies,
            vec![
                "conversion::domain::domain_function_1",
                "conversion::domain::domain_function_2",
                "conversion::domain::domain_function_2",
            ]
        );
    }

    #[test]
    fn test_file_path() {
        assert_eq!(
            get_module("./examples/sample_project/src/conversion/application.rs"),
            Ok(String::from("sample_project::conversion::application"))
        );
    }

    #[test]
    fn test_dependencies() {
        let source = r#"
            use crate::{
                application::{
                    container::{self, AcmeContainer},
                    geographic_info::{mock_geographic_info_default, GeographicInfoService},
                },
                domain::{
                    aggregate::quote::{FormType, ProductType, QuoteType, QuoteVersion},
                    Policy::{
                        Policy, PolicyActive, PolicyActiveSubstatus, PolicyStatus,
                        PolicySubstatusActive, PaymentMethod,
                    },
                    price::{PaymentFrequency, PriceValue},
                    save::{SavePurchasable, SaveStatus},
                    services::PolicyService,
                    types::UserType,
                },
                infrastructure::bridge::{
                    invoicing::{mock_invoicing_service_default, InvoicingService},
                    payment::{mock_payment_bridge, PaymentBridge},
                    s3_service::{mock_s3_service, S3Service},
                    antifraud::{mock_antifraud_service_default, AntifraudService},
                },
            };
        "#;

        let dependencies = get_dependencies_in_str(source, "my_application::domain".to_string());

        let expected_dependencies = vec![
            "my_application::application::container::self".to_string(),
            "my_application::application::container::AcmeContainer".to_string(),
            "my_application::application::geographic_info::mock_geographic_info_default"
                .to_string(),
            "my_application::application::geographic_info::GeographicInfoService".to_string(),
            "my_application::domain::aggregate::quote::FormType".to_string(),
            "my_application::domain::aggregate::quote::ProductType".to_string(),
            "my_application::domain::aggregate::quote::QuoteType".to_string(),
            "my_application::domain::aggregate::quote::QuoteVersion".to_string(),
            "my_application::domain::Policy::Policy".to_string(),
            "my_application::domain::Policy::PolicyActive".to_string(),
            "my_application::domain::Policy::PolicyActiveSubstatus".to_string(),
            "my_application::domain::Policy::PolicyStatus".to_string(),
            "my_application::domain::Policy::PolicySubstatusActive".to_string(),
            "my_application::domain::Policy::PaymentMethod".to_string(),
            "my_application::domain::price::PaymentFrequency".to_string(),
            "my_application::domain::price::PriceValue".to_string(),
            "my_application::domain::save::SavePurchasable".to_string(),
            "my_application::domain::save::SaveStatus".to_string(),
            "my_application::domain::services::PolicyService".to_string(),
            "my_application::domain::types::UserType".to_string(),
            "my_application::infrastructure::bridge::invoicing::mock_invoicing_service_default"
                .to_string(),
            "my_application::infrastructure::bridge::invoicing::InvoicingService".to_string(),
            "my_application::infrastructure::bridge::payment::mock_payment_bridge".to_string(),
            "my_application::infrastructure::bridge::payment::PaymentBridge".to_string(),
            "my_application::infrastructure::bridge::s3_service::mock_s3_service".to_string(),
            "my_application::infrastructure::bridge::s3_service::S3Service".to_string(),
            "my_application::infrastructure::bridge::antifraud::mock_antifraud_service_default"
                .to_string(),
            "my_application::infrastructure::bridge::antifraud::AntifraudService".to_string(),
        ];

        assert_eq!(expected_dependencies, dependencies);
    }

    #[test]
    fn test_external_dependencies() {
        let source = r#"
        use ansi_term::Color::RGB;
        use ansi_term::Style;
        use log::debug;
        use std::fmt::{Display, Formatter};
        "#;

        let dependencies = get_dependencies_in_str(source, "app::domain".to_string());

        let expected_dependencies = vec![
            "ansi_term::Color::RGB".to_string(),
            "ansi_term::Style".to_string(),
            "log::debug".to_string(),
            "std::fmt::Display".to_string(),
            "std::fmt::Formatter".to_string(),
        ];

        assert_eq!(expected_dependencies, dependencies);
    }

    #[test]
    fn test_super_dependencies() {
        assert_eq!(
            get_dependencies_in_file("./examples/sample_project/src/conversion/infrastructure.rs"),
            vec![String::from(
                "sample_project::conversion::application::application_function"
            )]
        );
    }

    #[test]
    fn test_glob_dependencies() {
        let source = r#"
        use crate::module::*;
        "#;

        let dependencies = get_dependencies_in_str(source, "crate::module".to_string());

        let expected_dependencies = vec!["crate::module::*".to_string()];

        assert_eq!(expected_dependencies, dependencies);
    }

    #[test]
    fn test_rename_dependencies() {
        let source = r#"
        use crate::module::original_name as alias_name;
        "#;

        let dependencies = get_dependencies_in_str(source, "crate::module".to_string());

        let expected_dependencies = vec!["crate::module::original_name".to_string()];

        assert_eq!(expected_dependencies, dependencies);
    }

    #[test]
    fn test_inline_module_multiple_dependencies() {
        let source = r#"
        mod submodule {
            use crate::some::dependency;
            use crate::another::dependency;
        }
        "#;

        let dependencies = get_dependencies_in_str(source, "crate".to_string());

        let expected_dependencies = vec![
            "crate::some::dependency".to_string(),
            "crate::another::dependency".to_string(),
        ];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_inline_nested_modules() {
        let source = r#"
        mod submodule {
            mod nested {
                use crate::nested::dependency;
            }
        }
        "#;

        let dependencies = get_dependencies_in_str(source, "crate".to_string());

        let expected_dependencies = vec!["crate::nested::dependency".to_string()];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_inline_empty_module() {
        let source = r#"
        mod submodule {}
        "#;

        let dependencies = get_dependencies_in_str(source, "crate".to_string());

        let expected_dependencies: Vec<String> = vec![];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_inline_complex_modules() {
        let source = r#"
        mod submodule {
            use crate::some::dependency;

            mod nested {
                use crate::nested::dependency;
            }
        }
        "#;

        let dependencies = get_dependencies_in_str(source, "crate".to_string());

        let expected_dependencies = vec![
            "crate::some::dependency".to_string(),
            "crate::nested::dependency".to_string(),
        ];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_inline_super_modules() {
        let source = r#"
            mod tests {
                use super::*;
            }
            "#;

        let dependencies =
            get_dependencies_in_str(source, "my_application::application::use_case".to_string());

        let expected_dependencies = vec!["my_application::application::use_case::*".to_string()];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_super_modules() {
        let source = r#"
            use crate::some::dependency;
            use super::query;
            "#;

        let dependencies =
            get_dependencies_in_str(source, "crate::application::use_case".to_string());

        let expected_dependencies = vec![
            "crate::some::dependency".to_string(),
            "crate::application::query".to_string(),
        ];

        assert_eq!(dependencies, expected_dependencies);
    }
}
