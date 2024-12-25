use rust_arkitect::builder::Architecture;
use rust_arkitect::validation::Rules;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum Components {
    Application,
    Domain,
    Infrastructure,
}

pub fn define_architecture() -> Rules {
    Architecture::define()
        .component(Components::Application)
            .defined_by("rust_arkitect::sample_project::application")
        .component(Components::Domain)
            .defined_by("rust_arkitect::sample_project::application")
        .component(Components::Infrastructure)
            .defined_by("rust_arkitect::sample_project::application")

        .rules_for(Components::Domain).must_not_depend_on_anything()
        .rules_for(Components::Application).may_depend_on(&[Components::Domain])
        .rules_for(Components::Infrastructure).may_depend_on(&[Components::Domain, Components::Application])

        .build()
}

#[test]
fn test_architecture_rules() {
    let architecture = define_architecture();

    let result = architecture.validate();

    assert_eq!(result, Ok(()))
}