use crate::rust_file::RustFile;
use std::collections::{HashMap, HashSet};
use syn::{
    visit::{self, Visit},
    ExprPath, Item, ItemMod, Path, TypePath, UseTree,
};

/// Restituisce tutte le dipendenze (use, path, ecc.) in un `RustFile`.
pub fn get_dependencies_in_file(file: &RustFile) -> Vec<String> {
    // 1) Raccogliamo le dipendenze dichiarate con `use` (anche nei mod inline).
    let mut dependencies = Vec::new();
    let mut aliases = HashMap::new();

    for item in &file.ast.items {
        match item {
            // Se troviamo un `use`, analizziamo la sua struttura (UseTree).
            Item::Use(use_item) => {
                collect_dependencies_from_tree(
                    &use_item.tree,
                    &mut dependencies,
                    &mut aliases,
                    &file.logical_path,
                    "",
                );
            }
            // Se troviamo un modulo inline, analizziamo i suoi item ricorsivamente.
            Item::Mod(mod_item) => {
                parse_inline_module(
                    mod_item,
                    &mut dependencies,
                    &mut aliases,
                    &file.logical_path,
                );
            }
            _ => {}
        }
    }

    // 2) Raccogliamo le dipendenze presenti nei riferimenti (expr path, type path) dentro il codice.
    let mut collector = DependencyVisitor {
        dependencies: Vec::new(),
        aliases: &aliases,
        current_module: &file.logical_path,
    };
    visit::visit_file(&mut collector, &file.ast);
    dependencies.extend(collector.dependencies);

    // 3) Rimuoviamo i duplicati (mantenendo l'ordine di apparizione).
    let mut unique_set = HashSet::new();
    dependencies
        .into_iter()
        .filter(|dep| unique_set.insert(dep.clone()))
        .collect()
}

/// Analizza in modo ricorsivo un modulo inline, collezionando `use` e altri mod.
fn parse_inline_module(
    mod_item: &ItemMod,
    dependencies: &mut Vec<String>,
    aliases: &mut HashMap<String, String>,
    current_module: &str,
) {
    // Se non è un modulo inline con `content`, ignoriamo.
    if let Some((_, items)) = &mod_item.content {
        // Costruiamo il percorso logico di questo modulo inline.
        let module_path = format!("{}::{}", current_module, mod_item.ident);
        for item in items {
            match item {
                Item::Use(use_item) => {
                    collect_dependencies_from_tree(
                        &use_item.tree,
                        dependencies,
                        aliases,
                        &module_path,
                        "",
                    );
                }
                Item::Mod(nested_mod) => {
                    // Ricorsione: i mod possono essere annidati.
                    parse_inline_module(nested_mod, dependencies, aliases, &module_path);
                }
                _ => {}
            }
        }
    }
}

/// Visita un albero di `UseTree` (come `use crate::...`) e raccoglie le dipendenze.
fn collect_dependencies_from_tree(
    tree: &UseTree,
    dependencies: &mut Vec<String>,
    aliases: &mut HashMap<String, String>,
    current_module: &str,
    prefix: &str,
) {
    // Nome della crate di base: se `current_module` è `crate::domain`,
    // la crate sarà "crate". Altrimenti potrebbe essere `sample_project`, ecc.
    let crate_name = current_module.split("::").next().unwrap_or("").to_string();

    match tree {
        UseTree::Path(use_path) => {
            let ident_str = use_path.ident.to_string();
            if ident_str == "super" {
                // Risolvi "super" come "modulo padre"
                let super_module = current_module.rsplitn(2, "::").nth(1).unwrap_or("");
                collect_dependencies_from_tree(
                    &use_path.tree,
                    dependencies,
                    aliases,
                    current_module,
                    super_module,
                );
            } else if ident_str == "crate" {
                // Risolvi "crate" come crate_name
                collect_dependencies_from_tree(
                    &use_path.tree,
                    dependencies,
                    aliases,
                    current_module,
                    &crate_name,
                );
            } else {
                // Aggiungiamo il prefisso (se presente)
                let new_prefix = if prefix.is_empty() {
                    ident_str
                } else {
                    format!("{}::{}", prefix, ident_str)
                };
                collect_dependencies_from_tree(
                    &use_path.tree,
                    dependencies,
                    aliases,
                    current_module,
                    &new_prefix,
                );
            }
        }
        UseTree::Group(group) => {
            // Se abbiamo `use something::{A, B, C}`, iteriamo su A, B, C
            for item in &group.items {
                collect_dependencies_from_tree(item, dependencies, aliases, current_module, prefix);
            }
        }
        UseTree::Name(use_name) => {
            // Caso `use something::Name;`
            let dep = format!("{}::{}", prefix, use_name.ident);
            dependencies.push(dep.clone());
            aliases.insert(use_name.ident.to_string(), dep);
        }
        UseTree::Glob(_) => {
            // Caso `use something::*;`
            let dep = format!("{}::*", prefix);
            dependencies.push(dep);
        }
        UseTree::Rename(rename) => {
            // Caso `use something::Original as Alias;`
            let dep = format!("{}::{}", prefix, rename.ident);
            dependencies.push(dep.clone());
            aliases.insert(rename.rename.to_string(), dep);
        }
    }
}

/// Struttura che visita l'AST con Syn per raccogliere i riferimenti usati in path (ExprPath, TypePath, ecc.).
struct DependencyVisitor<'a> {
    /// Dipendenze estratte dai path durante la visita.
    pub dependencies: Vec<String>,
    /// Mappa di alias per risolvere i path (es: `use crate::mymod as alias;`).
    pub aliases: &'a HashMap<String, String>,
    /// Modulo in cui ci troviamo (es: "crate::domain").
    pub current_module: &'a str,
}

impl<'ast, 'a> Visit<'ast> for DependencyVisitor<'a> {
    /// Visita un ExprPath come `crate::qualcosa::fun()`.
    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        let path_str = path_to_string(&node.path);

        if let Some(first_segment) = node.path.segments.first() {
            let first_ident = first_segment.ident.to_string();

            match first_ident.as_str() {
                "crate" => {
                    // Se inizia con `crate`, aggiungiamo direttamente.
                    self.dependencies.push(path_str);
                }
                "super" => {
                    // Risolvi "super" in base al modulo corrente.
                    let resolved = resolve_super_path(&node.path, self.current_module);
                    self.dependencies.push(resolved);
                }
                other => {
                    // Verifichiamo se c'è un alias (e.g. "alias" -> "some_library::stuff")
                    if let Some(full_path) = self.aliases.get(other) {
                        let resolved = rejoin_alias_with_rest(full_path, &node.path);
                        self.dependencies.push(resolved);
                    }
                }
            }
        }

        // Visita generica, così i figli non vengono saltati.
        visit::visit_expr_path(self, node);
    }

    /// Visita un TypePath come `crate::qualcosa::Tipo`.
    fn visit_type_path(&mut self, node: &'ast TypePath) {
        let path_str = path_to_string(&node.path);

        // Se ha un solo segmento (es: `String`, `Self`, ecc.) saltiamo: di solito non è una dipendenza esterna.
        if node.path.segments.len() == 1 {
            return visit::visit_type_path(self, node);
        }

        if let Some(first_segment) = node.path.segments.first() {
            let first_ident = first_segment.ident.to_string();

            match first_ident.as_str() {
                "crate" => {
                    self.dependencies.push(path_str);
                }
                "super" => {
                    let resolved = resolve_super_path(&node.path, self.current_module);
                    self.dependencies.push(resolved);
                }
                other => {
                    if let Some(full_path) = self.aliases.get(other) {
                        let resolved = rejoin_alias_with_rest(full_path, &node.path);
                        self.dependencies.push(resolved);
                    } else {
                        // Altrimenti aggiungiamo il path così com'è.
                        self.dependencies.push(path_str);
                    }
                }
            }
        }

        // Visita generica
        visit::visit_type_path(self, node);
    }
}

/// Converte un `Path` (es: `crate::some::path`) in una stringa (`"crate::some::path"`).
fn path_to_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Risolve la sintassi `super::qualcosa` portandola al modulo padre.
fn resolve_super_path(path: &Path, current_module: &str) -> String {
    // Trova il modulo padre. Se `current_module` = "crate::my_mod::sub_mod",
    // allora "super" dovrebbe diventare "crate::my_mod".
    let parent_module = current_module.rsplitn(2, "::").nth(1).unwrap_or("");
    let rest = path
        .segments
        .iter()
        .skip(1)
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    if rest.is_empty() {
        parent_module.to_string()
    } else {
        format!("{}::{}", parent_module, rest)
    }
}

/// Se abbiamo un alias (es: `use crate::mod::Original as Alias`) e troviamo un path `Alias::rest`,
/// questo ricostruisce la stringa corretta concatenando `Alias` con il resto.
fn rejoin_alias_with_rest(alias_full: &str, path: &Path) -> String {
    let mut segs = path.segments.iter();
    segs.next(); // saltiamo il primo segmento (alias)
    let rest = segs
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    if rest.is_empty() {
        alias_full.to_owned()
    } else {
        format!("{}::{}", alias_full, rest)
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

    #[test]
    fn test_dependencies_in_file_body() {
        let source = r#"
        use crate::some::dependency;
        use crate::other_dependency;

        fn example() {
            crate::other::module::function();
            crate::some::dependency::function();
            other_dependency::function();
        }
    "#;

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::domain",
            source,
        ));

        let expected_dependencies = vec![
            "crate::some::dependency",
            "crate::other_dependency",
            "crate::other::module::function",
            "crate::some::dependency::function",
            "crate::other_dependency::function",
        ];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_dependencies_in_file_struct_declaration() {
        let source = r#"
        struct SomeStruct {
            a_field: some_library::some::dependency::SomeType
        }
        "#;

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::domain",
            source,
        ));

        let expected_dependencies = vec!["some_library::some::dependency::SomeType"];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_dependencies_in_file_struct_declaration_with_crate() {
        let source = r#"
        struct SomeStruct {
            a_field: crate::some::dependency::SomeType
        }
        "#;

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::domain",
            source,
        ));

        let expected_dependencies = vec!["crate::some::dependency::SomeType"];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_standard_types_are_ignored() {
        let source = r#"
        struct SomeStruct {
            a_field: Self,
            another_field: Vec<String>
        }
        "#;

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::domain",
            source,
        ));

        let expected_dependencies: Vec<String> = vec![];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_aliasing_and_standard_types() {
        let source = r#"
        use std::collections::HashMap as MyMap;

        struct SomeStruct {
            a_field: Vec<String>,
            b_field: MyMap<i32, bool>,
            c_field: some_library::dependency::CustomType,
        }
        "#;

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::domain",
            source,
        ));

        let expected_dependencies = vec![
            "std::collections::HashMap",
            "some_library::dependency::CustomType",
        ];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_aliasing_and_standard_types_2() {
        let source = r#"
        use some_library::collections as collections;

        struct SomeStruct {
            b_field: collections::MyMap<i32, bool>,
            c_field: some_library::dependency::CustomType,
        }
        "#;

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::domain",
            source,
        ));

        let expected_dependencies = vec![
            "some_library::collections",
            "some_library::collections::MyMap",
            "some_library::dependency::CustomType",
        ];

        assert_eq!(dependencies, expected_dependencies);
    }

    #[test]
    fn test_return_types() {
        let source = r#"
        use some_library::collections::RandomCollection as RandCollections;
        use another_lib::utils as utils;

        fn a_function() -> RandCollections {
            todo!()
        }

        fn b_function() -> utils::Hello {
            todo!()
        }

        fn b_function(a: utils::Ciao) -> utils::Hello {
            todo!()
        }
        "#;

        let dependencies = get_dependencies_in_file(&RustFile::from_content(
            "/src/domain.rs",
            "crate::domain",
            source,
        ));

        let expected_dependencies = vec![
            "some_library::collections::RandomCollection",
            "another_lib::utils",
            "another_lib::utils::Hello",
            "another_lib::utils::Ciao",
        ];

        assert_eq!(dependencies, expected_dependencies);
    }
}
