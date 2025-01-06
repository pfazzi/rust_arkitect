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

    let mut dependencies = Vec::new();
    let module = get_module(path).unwrap();

    for item in ast.items.iter() {
        if let Item::Use(ItemUse { tree, .. }) = item {
            collect_dependencies_from_tree(tree, &mut dependencies, "".to_string(), module.clone());
        }
    }

    dependencies
}

fn get_dependencies_in_str(s: &str, super_module: String) -> Vec<String> {
    let ast: File = match syn::parse_str(s) {
        Ok(ast) => ast,
        Err(e) => panic!("Failed to parse string '{}': {}", s, e),
    };

    let mut dependencies = Vec::new();

    for item in ast.items.iter() {
        if let Item::Use(ItemUse { tree, .. }) = item {
            collect_dependencies_from_tree(
                tree,
                &mut dependencies,
                "".to_string(),
                super_module.clone(),
            );
        }
    }

    dependencies
}

fn collect_dependencies_from_tree(
    tree: &UseTree,
    dependencies: &mut Vec<String>,
    prefix: String,
    super_module: String,
) {
    match tree {
        UseTree::Path(path) => {
            let ident = path.ident.to_string();
            let token = path.colon2_token.to_token_stream().to_string();
            if ident == "super" {
                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    super_module.rsplitn(2, "::").nth(1).unwrap().to_string(),
                    super_module,
                );
            } else if ident == "crate" {
                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    "crate".to_string(),
                    super_module,
                );
            } else {
                let prefix: String = if !prefix.is_empty() {
                    format!("{}{}{}", prefix, token, ident)
                } else {
                    ident
                };
                collect_dependencies_from_tree(
                    path.tree.deref(),
                    dependencies,
                    prefix,
                    super_module,
                );
            }
        }
        UseTree::Group(group) => {
            group.items.iter().for_each(|item| {
                collect_dependencies_from_tree(
                    item,
                    dependencies,
                    prefix.clone(),
                    super_module.clone(),
                );
            });
        }
        UseTree::Name(name) => {
            let ident = name.ident.to_string();
            let dep = format!("{}{}{}", prefix, "::", ident);
            dependencies.push(dep);
        }
        UseTree::Glob(glob) => {
            let ident = glob.to_token_stream().to_string();
            let dep = format!("{}{}{}", prefix, "::", ident);
            dependencies.push(dep);
        }
        UseTree::Rename(rename) => {
            let ident = rename.ident.to_string();
            let dep = format!("{}{}{}", prefix, "::", ident);
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

#[test]
pub fn test_parsing() {
    let dependencies =
        get_dependencies_in_file("./examples/sample_project/src/conversion/application.rs");
    assert_eq!(
        dependencies,
        vec![
            "crate::conversion::domain::domain_function_1",
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

    let dependencies = get_dependencies_in_str(source, "crate::domain".to_string());

    let expected_dependencies = vec![
        "crate::application::container::self".to_string(),
        "crate::application::container::AcmeContainer".to_string(),
        "crate::application::geographic_info::mock_geographic_info_default".to_string(),
        "crate::application::geographic_info::GeographicInfoService".to_string(),
        "crate::domain::aggregate::quote::FormType".to_string(),
        "crate::domain::aggregate::quote::ProductType".to_string(),
        "crate::domain::aggregate::quote::QuoteType".to_string(),
        "crate::domain::aggregate::quote::QuoteVersion".to_string(),
        "crate::domain::Policy::Policy".to_string(),
        "crate::domain::Policy::PolicyActive".to_string(),
        "crate::domain::Policy::PolicyActiveSubstatus".to_string(),
        "crate::domain::Policy::PolicyStatus".to_string(),
        "crate::domain::Policy::PolicySubstatusActive".to_string(),
        "crate::domain::Policy::PaymentMethod".to_string(),
        "crate::domain::price::PaymentFrequency".to_string(),
        "crate::domain::price::PriceValue".to_string(),
        "crate::domain::save::SavePurchasable".to_string(),
        "crate::domain::save::SaveStatus".to_string(),
        "crate::domain::services::PolicyService".to_string(),
        "crate::domain::types::UserType".to_string(),
        "crate::infrastructure::bridge::invoicing::mock_invoicing_service_default".to_string(),
        "crate::infrastructure::bridge::invoicing::InvoicingService".to_string(),
        "crate::infrastructure::bridge::payment::mock_payment_bridge".to_string(),
        "crate::infrastructure::bridge::payment::PaymentBridge".to_string(),
        "crate::infrastructure::bridge::s3_service::mock_s3_service".to_string(),
        "crate::infrastructure::bridge::s3_service::S3Service".to_string(),
        "crate::infrastructure::bridge::antifraud::mock_antifraud_service_default".to_string(),
        "crate::infrastructure::bridge::antifraud::AntifraudService".to_string(),
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

    let dependencies = get_dependencies_in_str(source, "crate::domain".to_string());

    let expected_dependencies = vec![
        "crate::dependency_parsing::get_dependencies_in_file".to_string(),
        "crate::dependency_parsing::get_module".to_string(),
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
            "crate::conversion::application::application_function"
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

    let expected_dependencies = vec![
        "crate::module::original_name".to_string(), // Rename dependency
    ];

    assert_eq!(expected_dependencies, dependencies);
}
