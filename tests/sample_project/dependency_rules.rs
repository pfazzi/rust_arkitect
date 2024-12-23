use rust_arkitect::builder::Architecture;
use rust_arkitect::validation::Rules;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum Components {
    Application,
    Domain,
    Infrastructure,
}

pub fn define_architecture() -> Rules {
    Architecture::with_components()
        .component(Components::Application).defined_by("./src/application.rs")
        .component(Components::Domain).defined_by("domain")
        .component(Components::Infrastructure).defined_by("infrastructure")

        .rules_for(Components::Domain).must_not_depend_on_anything()
        .rules_for(Components::Application).may_depend_on(&[Components::Domain])
        .rules_for(Components::Infrastructure).may_depend_on(&[Components::Domain, Components::Application])

        .build()
}
