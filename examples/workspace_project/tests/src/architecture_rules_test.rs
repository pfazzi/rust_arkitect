use rust_arkitect::dsl::architectural_rules::ArchitecturalRules;
use rust_arkitect::dsl::arkitect::Arkitect;
use rust_arkitect::dsl::project::Project;

#[test]
fn test_vertical_slices_architecture_rules() {
    Arkitect::init_logger();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_crate("contracts")
            .must_not_depend_on_anything()

        .rules_for_crate("conversion")
            .allow_dependencies_on(&["contracts"])

        .rules_for_crate("policy_management")
            .allow_dependencies_on(&["contracts"])

        .rules_for_crate("application")
            .allow_dependencies_on(&["conversion", "policy_management"])

        .build();

    let project = Project::new();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok());
}

#[test]
fn test_mvc_architecture_rules() {
    Arkitect::init_logger();

    let project = Project::new();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_module("crate::policy_management::model")
            .must_not_depend_on_anything()

        .rules_for_module("crate::policy_management::repository")
            .allow_dependencies_on(&["crate::policy_management::model"])

        .rules_for_module("crate::policy_management::controller")
            .allow_dependencies_on(&["crate::policy_management::repository", "crate::policy_management::model"])

        .build();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok());
}

#[test]
fn test_three_tier_architecture() {
    Arkitect::init_logger();

    let project = Project::new();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_module("crate::conversion::application")
            .allow_dependencies_on(&["crate::conversion::domain"])

        .rules_for_module("crate::conversion::domain")
            .must_not_depend_on_anything()

        .rules_for_module("crate::conversion::infrastructure")
            .allow_dependencies_on(&["crate::conversion::domain", "crate::conversion::application"])
        .build();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok());
}
