use rust_arkitect::arkitect::{Arkitect, Project};
use rust_arkitect::builder::Architecture;

#[test]
fn test_vertical_slices_architecture_rules() {
    let rules = Architecture::define()
        .component("Conversion")
        .defined_as("crate::conversion")
        .may_depend_on(&["Contracts"])
        .component("PolicyManagement")
        .defined_as("crate::policy_management")
        .may_depend_on(&["Contracts"])
        .component("Contracts")
        .defined_as("crate::contracts")
        .must_not_depend_on_anything()
        .rules();

    let project = Project::load("/Users/patrickfazzi/Projects/rust_arkitect/sample_project/src");

    let result = Arkitect::validate(project).against(rules);

    assert_eq!(result, Ok(()))
}

#[test]
fn test_mvc_architecture_rules() {
    let rules = Architecture::define()
        .component("Model")
        .defined_as("crate::policy_management::model")
        .must_not_depend_on_anything()
        .component("Repository")
        .defined_as("crate::policy_management::repository")
        .may_depend_on(&["Model"])
        .component("Controller")
        .defined_as("crate::policy_management::controller")
        .may_depend_on(&["Repository", "Model"])
        .rules();

    let project = Project::load("/Users/patrickfazzi/Projects/rust_arkitect/sample_project/src");

    let result = Arkitect::validate(project).against(rules);

    assert_eq!(result, Ok(()))
}

#[test]
fn test_three_tier_architecture() {
    let rules = Architecture::define()
        .component("Application")
        .defined_as("crate::conversion::application")
        .may_depend_on(&["Domain"])
        .component("Domain")
        .defined_as("crate::conversion::domain")
        .must_not_depend_on_anything()
        .component("Infrastructure")
        .defined_as("crate::conversion::infrastructure")
        .may_depend_on(&["Domain", "Application"])
        .rules();

    let project = Project::load("/Users/patrickfazzi/Projects/rust_arkitect/sample_project/src");

    let result = Arkitect::validate(project).against(rules);

    assert_eq!(result, Ok(()))
}
