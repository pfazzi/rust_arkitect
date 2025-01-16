use rust_arkitect::dsl::{ArchitecturalRules, Arkitect, Project};

#[test]
fn test_architecture() {
    let project = Project::from_absolute_path("/Users/patrickfazzi/Prima/sbarter/");

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .component("Application")
            .located_at("application")
            .must_not_depend_on_anything()

        .component("Sbarter")
            .located_at("sbarter")
            .may_depend_on(&["SbarterLib", "Application"])

        .component("SbarterLib")
            .located_at("sbarter-lib")
            .must_not_depend_on_anything()

        .finalize();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(
        result.is_ok(),
        "Detected {} violations",
        result.err().unwrap().len()
    );
}
