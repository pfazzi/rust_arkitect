use std::collections::HashSet;
use std::fs;
use std::ops::Deref;
use std::path::Path;
use syn::{File, Item, ItemUse, UseTree};
use toml::Value;

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
        Ok(module) => get_dependencies_in_ast(&ast, &module),
        Err(_e) => vec![],
    }
}

fn parse_module_item(item: &Item, dependencies: &mut Vec<String>, current_module: &str) {
    match item {
        Item::Use(ItemUse { tree, .. }) => {
            collect_dependencies_from_tree(tree, dependencies, current_module, "");
        }
        Item::Mod(mod_item) => {
            if let Some((_, items)) = &mod_item.content {
                for sub_item in items.iter() {
                    parse_module_item(sub_item, dependencies, current_module);
                }
            }
        }
        _ => {}
    }
}

#[allow(dead_code)]
fn get_dependencies_in_str(s: &str, module: &str) -> Vec<String> {
    let ast: File = match syn::parse_str(s) {
        Ok(ast) => ast,
        Err(e) => panic!("Failed to parse string '{}': {}", s, e),
    };

    get_dependencies_in_ast(&ast, module)
}

pub fn get_dependencies_in_ast(ast: &File, current_module: &str) -> Vec<String> {
    let mut dependencies = Vec::new();

    for item in ast.items.iter() {
        match item {
            Item::Use(ItemUse { tree, .. }) => {
                collect_dependencies_from_tree(tree, &mut dependencies, current_module, "");
            }
            Item::Mod(mod_item) => {
                if let Some((_, items)) = &mod_item.content {
                    let module = format!("{}::{}", current_module, mod_item.ident);
                    for sub_item in items.iter() {
                        parse_module_item(sub_item, &mut dependencies, &module);
                    }
                }
            }
            _ => {}
        }
    }

    unique_values(dependencies)
}

fn unique_values<T: std::hash::Hash + Eq + Clone>(vec: Vec<T>) -> Vec<T> {
    let mut unique_set = HashSet::new();
    vec.into_iter()
        .filter(|item| unique_set.insert(item.clone()))
        .collect()
}

fn collect_dependencies_from_tree(
    tree: &UseTree,
    dependencies: &mut Vec<String>,
    current_module: &str,
    prefix: &str,
) {
    let crate_name = current_module.split("::").next().unwrap_or("");

    match tree {
        UseTree::Path(path) => {
            let ident = path.ident.to_string();
            if ident == "super" {
                let super_module = current_module.rsplitn(2, "::").nth(1).unwrap_or("");

                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    current_module,
                    super_module,
                );
            } else if ident == "crate" {
                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    current_module,
                    crate_name,
                );
            } else {
                let ident = if !prefix.is_empty() {
                    format!("{}::{}", prefix, ident)
                } else {
                    ident
                };
                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    current_module,
                    ident.as_str(),
                );
            }
        }
        UseTree::Group(group) => {
            for item in group.items.iter() {
                collect_dependencies_from_tree(item, dependencies, current_module, prefix);
            }
        }
        UseTree::Name(name) => {
            let dep = format!("{}::{}", prefix, name.ident);
            dependencies.push(dep);
        }
        UseTree::Glob(_) => {
            let dep = format!("{}::*", prefix);
            dependencies.push(dep);
        }
        UseTree::Rename(rename) => {
            let ident = format!("{}::{}", prefix, rename.ident);
            dependencies.push(ident);
        }
    }
}

pub fn get_module(file_path: &str) -> Result<String, String> {
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
    use super::*;

    #[test]
    fn test_get_module() {
        let module =
            get_module("./examples/workspace_project/conversion/src/application.rs").unwrap();

        assert_eq!(module, "conversion::application")
    }

    #[test]
    fn test_get_module_on_a_random_file() {
        let module = get_module("./examples/workspace_project/assets/file_1.txt");

        assert_eq!(
            module,
            Err("Invalid file type: expected a Rust file (.rs), found 'txt'".to_string())
        );
    }

    #[test]
    pub fn test_parsing() {
        let dependencies =
            get_dependencies_in_file("./examples/sample_project/src/conversion/application.rs");
        assert_eq!(
            dependencies,
            vec![
                "sample_project::contracts::external_services::service_call_one",
                "sample_project::conversion::domain::domain_function_1",
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
            ]
        );
    }

    #[test]
    fn test_get_module_on_a_directory() {
        assert_eq!(
            get_module("./examples/workspace_project/"),
            Err(String::from(
                "The specified path './examples/workspace_project/' is a directory, not a file"
            ))
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

        let dependencies = get_dependencies_in_str(source, "crate::domain");

        let expected_dependencies = vec![
            "crate::application::container::self",
            "crate::application::container::AcmeContainer",
            "crate::application::geographic_info::mock_geographic_info_default",
            "crate::application::geographic_info::GeographicInfoService",
            "crate::domain::aggregate::quote::FormType",
            "crate::domain::aggregate::quote::ProductType",
            "crate::domain::aggregate::quote::QuoteType",
            "crate::domain::aggregate::quote::QuoteVersion",
            "crate::domain::Policy::Policy",
            "crate::domain::Policy::PolicyActive",
            "crate::domain::Policy::PolicyActiveSubstatus",
            "crate::domain::Policy::PolicyStatus",
            "crate::domain::Policy::PolicySubstatusActive",
            "crate::domain::Policy::PaymentMethod",
            "crate::domain::price::PaymentFrequency",
            "crate::domain::price::PriceValue",
            "crate::domain::save::SavePurchasable",
            "crate::domain::save::SaveStatus",
            "crate::domain::services::PolicyService",
            "crate::domain::types::UserType",
            "crate::infrastructure::bridge::invoicing::mock_invoicing_service_default",
            "crate::infrastructure::bridge::invoicing::InvoicingService",
            "crate::infrastructure::bridge::payment::mock_payment_bridge",
            "crate::infrastructure::bridge::payment::PaymentBridge",
            "crate::infrastructure::bridge::s3_service::mock_s3_service",
            "crate::infrastructure::bridge::s3_service::S3Service",
            "crate::infrastructure::bridge::antifraud::mock_antifraud_service_default",
            "crate::infrastructure::bridge::antifraud::AntifraudService",
        ];

        assert_eq!(expected_dependencies, dependencies);
    }

    #[test]
    fn test_external_dependencies() {
        let source = r#"
        use crate::dependency_parsing::get_dependencies_in_file;
        use crate::dependency_parsing::get_module;
        use ansi_term::Color::RGB;
        use ansi_term::Style;
        use log::debug;
        use std::fmt::{Display, Formatter};
        "#;

        let dependencies = get_dependencies_in_str(source, "my_app");

        let expected_dependencies = vec![
            "my_app::dependency_parsing::get_dependencies_in_file",
            "my_app::dependency_parsing::get_module",
            "ansi_term::Color::RGB",
            "ansi_term::Style",
            "log::debug",
            "std::fmt::Display",
            "std::fmt::Formatter",
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

        let dependencies = get_dependencies_in_str(source, "crate::module");

        let expected_dependencies = vec!["crate::module::*"];

        assert_eq!(expected_dependencies, dependencies);
    }

    #[test]
    fn test_rename_dependencies() {
        let source = r#"
        use crate::module::original_name as alias_name;
        "#;

        let dependencies = get_dependencies_in_str(source, "crate::module");

        let expected_dependencies = vec!["crate::module::original_name"];

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

        let dependencies = get_dependencies_in_str(source, "crate");

        let expected_dependencies = vec!["crate::some::dependency", "crate::another::dependency"];

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

        let dependencies = get_dependencies_in_str(source, "crate");

        let expected_dependencies = vec!["crate::nested::dependency".to_string()];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_inline_empty_module() {
        let source = r#"
        mod submodule {}
        "#;

        let dependencies = get_dependencies_in_str(source, "crate");

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

        let dependencies = get_dependencies_in_str(source, "crate");

        let expected_dependencies = vec!["crate::some::dependency", "crate::nested::dependency"];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_inline_super_modules() {
        let source = r#"
            mod tests {
                use super::*;
            }
            "#;

        let dependencies = get_dependencies_in_str(source, "crate::application::use_case");

        let expected_dependencies = vec!["crate::application::use_case::*"];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_super_modules() {
        let source = r#"
            use crate::some::dependency;
            use super::query;
            "#;

        let dependencies = get_dependencies_in_str(source, "crate::application::use_case");

        let expected_dependencies = vec!["crate::some::dependency", "crate::application::query"];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_get_module_with_a_file_in_folder_without_src() {
        let module = get_module("tests/test_architecture.rs");

        assert_eq!("rust_arkitect::tests::test_architecture", module.unwrap());
    }
}
