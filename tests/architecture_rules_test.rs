use rust_arkitect::builder::Architecture;
use rust_arkitect::validation::Rules;

pub fn define_architecture() -> Rules {
    Architecture::define()
        .component("Application")
            .defined_by("rust_arkitect::sample_project::application")
        .component("Domain")
            .defined_by("rust_arkitect::sample_project::application")
        .component("Infrastructure")
            .defined_by("rust_arkitect::sample_project::application")

        .rules_for("Domain").must_not_depend_on_anything()
        .rules_for("Application").may_depend_on(&["Domain"])
        .rules_for("Infrastructure").may_depend_on(&["Domain", "Application"])

        .build()
}

#[test]
fn test_architecture_rules() {
    let architecture = define_architecture();

    let result = architecture.validate();

    assert_eq!(result, Ok(()))
}