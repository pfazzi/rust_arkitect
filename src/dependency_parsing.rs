use crate::rust_file::RustFile;
use std::collections::HashSet;
use std::ops::Deref;
use syn::{Item, ItemUse, UseTree};

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

pub fn get_dependencies_in_file(file: &RustFile) -> Vec<String> {
    let mut dependencies = Vec::new();

    for item in file.ast.items.iter() {
        match item {
            Item::Use(ItemUse { tree, .. }) => {
                collect_dependencies_from_tree(tree, &mut dependencies, &file.logical_path, "");
            }
            Item::Mod(mod_item) => {
                if let Some((_, items)) = &mod_item.content {
                    let module = format!("{}::{}", &file.logical_path, mod_item.ident);
                    for sub_item in items.iter() {
                        parse_module_item(sub_item, &mut dependencies, &module);
                    }
                }
            }
            _ => {}
        }
    }

    let mut unique_set = HashSet::new();
    dependencies
        .into_iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parsing() {
        let dependencies = get_dependencies_in_file(&RustFile::from_file_system(
            "./examples/sample_project/src/conversion/application.rs",
        ));
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
        let dependencies = get_dependencies_in_file(&RustFile::from_file_system(
            "./examples/workspace_project/conversion/src/application.rs",
        ));
        assert_eq!(
            dependencies,
            vec![
                "conversion::domain::domain_function_1",
                "conversion::domain::domain_function_2",
            ]
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

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::domain",
            source,
        ));

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

        let dependencies =
            get_dependencies_in_file(&RustFile::from_content("/src/my_app.rs", "my_app", source));

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
            get_dependencies_in_file(&RustFile::from_file_system(
                "./examples/sample_project/src/conversion/infrastructure.rs"
            )),
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

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::module",
            source,
        ));

        let expected_dependencies = vec!["crate::module::*"];

        assert_eq!(expected_dependencies, dependencies);
    }

    #[test]
    fn test_rename_dependencies() {
        let source = r#"
        use crate::module::original_name as alias_name;
        "#;

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::module",
            source,
        ));

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

        let dependencies =
            get_dependencies_in_file(&RustFile::from_content("/src/domain.rs", "crate", source));

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

        let dependencies =
            get_dependencies_in_file(&RustFile::from_content("/src/domain.rs", "crate", source));

        let expected_dependencies = vec!["crate::nested::dependency".to_string()];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_inline_empty_module() {
        let source = r#"
        mod submodule {}
        "#;

        let dependencies =
            get_dependencies_in_file(&RustFile::from_content("/src/domain.rs", "crate", source));

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

        let dependencies =
            get_dependencies_in_file(&RustFile::from_content("/src/domain.rs", "crate", source));

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

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::application::use_case",
            source,
        ));

        let expected_dependencies = vec!["crate::application::use_case::*"];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_super_modules() {
        let source = r#"
            use crate::some::dependency;
            use super::query;
            "#;

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::application::use_case",
            source,
        ));

        let expected_dependencies = vec!["crate::some::dependency", "crate::application::query"];

        assert_eq!(dependencies, expected_dependencies);
    }
}
