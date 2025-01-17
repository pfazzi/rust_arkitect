#![cfg(test)]

use rust_arkitect::dsl::architectural_rules::ArchitecturalRules;
use rust_arkitect::dsl::arkitect::Arkitect;
use rust_arkitect::dsl::project::Project;

#[test]
fn test_architectural_rules() {
    Arkitect::init_logger();

    let project = Project::new();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_module("crate::dsl")
            .allow_dependencies_on(&["crate::engine", "crate::rules", "std::collections", "std::marker::PhantomData", "std::path"])

        .rules_for_module("crate::engine")
            .allow_dependencies_on(&["crate::rules", "ansi_term", "log", "std::fs"])

        .rules_for_module("crate::rules")
            .allow_dependencies_on(&["crate::dependency_parsing", "ansi_term", "log", "std::fmt"])

        .rules_for_crate("crate::dependency_parsing")
            .allow_dependencies_on(&["syn", "quote", "std::path", "std::ops", "std::fs"])

        .build();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(
        result.is_ok(),
        "Detected {} violations",
        result.err().unwrap().len()
    );
}
