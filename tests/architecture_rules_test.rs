use rust_arkitect::dsl::{ArchitecturalRules, Arkitect, Project};

#[test]
fn test_vertical_slices_architecture_rules() {
    Arkitect::init_logger();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .component("Conversion")
            .located_at("crate::conversion")
            .may_depend_on(&["Contracts"])

        .component("PolicyManagement")
            .located_at("crate::policy_management")
            .may_depend_on(&["Contracts"])

        .component("Contracts")
            .located_at("crate::contracts")
            .must_not_depend_on_anything()

        .finalize();

    let project = Project::load("./../rust_arkitect/sample_project/src");

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert_eq!(result, Ok(()))
}

#[test]
fn test_mvc_architecture_rules() {
    Arkitect::init_logger();

    let project = Project::load("./../rust_arkitect/sample_project/src");

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .component("Model")
            .located_at("crate::policy_management::model")
            .must_not_depend_on_anything()

        .component("Repository")
            .located_at("crate::policy_management::repository")
            .may_depend_on(&["Model"])

        .component("Controller")
            .located_at("crate::policy_management::controller")
            .may_depend_on(&["Repository", "Model"])
        .finalize();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert_eq!(result, Ok(()))
}

#[test]
fn test_three_tier_architecture() {
    Arkitect::init_logger();

    let project = Project::load("./../rust_arkitect/sample_project/src");

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .component("Application")
            .located_at("crate::conversion::application")
            .may_depend_on(&["Domain"])

        .component("Domain")
            .located_at("crate::conversion::domain")
            .must_not_depend_on_anything()

        .component("Infrastructure")
            .located_at("crate::conversion::infrastructure")
            .may_depend_on(&["Domain", "Application"])

        .finalize();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok());
}
