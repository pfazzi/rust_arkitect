#![cfg(test)]

use rust_arkitect::dsl::architectural_rules::ArchitecturalRules;
use rust_arkitect::dsl::arkitect::Arkitect;
use rust_arkitect::dsl::project::Project;

#[test]
fn test_architectural_rules() {
    Arkitect::init_logger();

    let project = Project::from_current_crate();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_module("rust_arkitect::dsl")
            .it_may_depend_on(&[
                "rust_arkitect::engine",
                "rust_arkitect::rules",
                "rust_arkitect::rule",
                "rust_arkitect::rust_file",
                "std::collections",
                "std::marker::PhantomData",
                "std::path",
                "std::fmt",
                "std::env",
                "std::fs"
            ])

        .rules_for_module("rust_arkitect::engine")
            .it_may_depend_on(&[
                "rust_arkitect::rule",
                "rust_arkitect::rust_file",
                "ansi_term",
                "log",
                "std::env",
                "std::fs",
                "std::path",
                "toml"
            ])

        .rules_for_module("rust_arkitect::rules")
            .it_may_depend_on(&[
                "rust_arkitect::rust_file",
                "rust_arkitect::rule",
                "rust_arkitect::dependency_parsing",
                "ansi_term",
                "log",
                "std::fmt"
            ])

        .rules_for_crate("rust_arkitect::rule")
            .it_may_depend_on(&[
                "rust_arkitect::rust_file",
                "std::fmt",
            ])

        .rules_for_crate("rust_arkitect::dependency_parsing")
            .it_may_depend_on(&[
                "rust_arkitect::rust_file",
                "syn",
                "std::collections",
                "std::path",
                "std::ops",
                "std::fs"
            ])

        .rules_for_crate("rust_arkitect::rust_file")
            .it_may_depend_on(&[
                "std::path",
                "syn",
                "toml", // Why?
            ])

        .build();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(
        result.is_ok(),
        "Detected {} violations",
        result.err().unwrap().len()
    );
}
