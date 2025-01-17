use rust_arkitect::dsl::architectural_rules::ArchitecturalRules;
use rust_arkitect::dsl::arkitect::Arkitect;
use rust_arkitect::dsl::project::Project;

#[test]
fn test_vertical_slices_architecture_rules() {
    Arkitect::init_logger();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_module("sample_project::conversion")
            .allow_dependencies_on(&["sample_project::contracts"])

        .rules_for_module("sample_project::policy_management")
            .allow_dependencies_on(&["sample_project::contracts"])

        .rules_for_module("sample_project::contracts")
            .must_not_depend_on_anything()

        .build();

    let project = Project::new();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok());
}

#[test]
fn test_mvc_architecture_rules() {
    Arkitect::init_logger();

    let project = Project::from_relative_path(file!(), "./../");

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_module("sample_project::policy_management::model")
            .must_not_depend_on_anything()

        .rules_for_module("sample_project::policy_management::repository")
            .allow_dependencies_on(&["sample_project::policy_management::model"])

        .rules_for_module("sample_project::policy_management::controller")
            .allow_dependencies_on(&["sample_project::policy_management::repository", "sample_project::policy_management::model"])

        .build();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok())
}

#[test]
fn test_three_tier_architecture() {
    Arkitect::init_logger();

    let project = Project::from_relative_path(file!(), "./../");

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_module("sample_project::conversion::application")
            .allow_dependencies_on(&["sample_project::conversion::domain", "sample_project::contract"])

        .rules_for_module("sample_project::conversion::domain")
            .must_not_depend_on_anything()

        .rules_for_module("sample_project::conversion::infrastructure")
            .allow_dependencies_on(&["sample_project::conversion::domain", "sample_project::conversion::application"])

        .build();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok());
}
