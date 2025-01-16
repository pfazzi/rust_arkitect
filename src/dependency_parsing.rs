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

    let module = get_module(path).unwrap();

    get_dependencies_in_ast(ast, &module)
}

fn parse_module_item(item: &Item, dependencies: &mut Vec<String>, module_prefix: &str) {
    match item {
        Item::Use(ItemUse { tree, .. }) => {
            collect_dependencies_from_tree(tree, dependencies, "", module_prefix);
        }
        Item::Mod(mod_item) => {
            if let Some((_, items)) = &mod_item.content {
                let new_prefix = format!("{}::{}", module_prefix, mod_item.ident);
                for sub_item in items.iter() {
                    parse_module_item(sub_item, dependencies, &new_prefix);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
fn get_dependencies_in_str(s: &str, super_module: &str) -> Vec<String> {
    let ast: File = match syn::parse_str(s) {
        Ok(ast) => ast,
        Err(e) => panic!("Failed to parse string '{}': {}", s, e),
    };

    get_dependencies_in_ast(ast, super_module)
}

fn get_dependencies_in_ast(ast: File, module: &str) -> Vec<String> {
    let mut dependencies = Vec::new();

    for item in ast.items.iter() {
        match item {
            Item::Use(ItemUse { tree, .. }) => {
                collect_dependencies_from_tree(tree, &mut dependencies, "", module);
            }
            Item::Mod(mod_item) => {
                if let Some((_, items)) = &mod_item.content {
                    let module = format!("{}::{}", module, mod_item.ident);
                    for sub_item in items.iter() {
                        parse_module_item(sub_item, &mut dependencies, &module);
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
    prefix: &str,
    parent_module: &str,
) {
    match tree {
        UseTree::Path(path) => {
            let ident = path.ident.to_string();
            let token = path.colon2_token.to_token_stream().to_string();
            if ident == "super" {
                // Calcola il prefisso del modulo padre
                let parent_prefix = parent_module.rsplit_once("::").map(|x| x.0).unwrap_or("");

                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    parent_prefix,
                    parent_prefix,
                );
            } else if ident == "crate" {
                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    "crate",
                    parent_module,
                );
            } else {
                let new_prefix = if !prefix.is_empty() {
                    format!("{}{}{}", prefix, token, ident)
                } else {
                    ident
                };

                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    &new_prefix,
                    parent_module,
                );
            }
        }
        UseTree::Group(group) => {
            group.items.iter().for_each(|item| {
                collect_dependencies_from_tree(item, dependencies, prefix, parent_module);
            });
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
            let dep = format!("{}::{}", prefix, rename.ident);
            dependencies.push(dep);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parsing() {
        let dependencies =
            get_dependencies_in_file("./examples/sample_project/src/conversion/application.rs");
        assert_eq!(
            dependencies,
            vec![
                "crate::conversion::domain::domain_function_1",
                "crate::conversion::domain::domain_function_2",
                "crate::conversion::domain::domain_function_2",
            ]
        );
    }

    #[test]
    fn test_file_path() {
        assert_eq!(
            get_module("./examples/sample_project/src/conversion/application.rs"),
            Ok(String::from("crate::conversion::application"))
        );

        assert_eq!(
            get_module(
                "/users/reandom/projects/rust_arkitect/sample_project/sample_project/src/conversion"
            ),
            Ok(String::from("crate::conversion"))
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
        use crate::dependency_parsing::{get_dependencies_in_file, get_module};
        use ansi_term::Color::RGB;
        use ansi_term::Style;
        use log::debug;
        use std::fmt::{Display, Formatter};
        "#;

        let dependencies = get_dependencies_in_str(source, "crate::domain");

        let expected_dependencies = vec![
            "crate::dependency_parsing::get_dependencies_in_file",
            "crate::dependency_parsing::get_module",
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
                "crate::conversion::application::application_function"
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
}
