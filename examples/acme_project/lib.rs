pub mod architecture_tests {
    use rust_arkitect::dsl::ArchitecturalRules;
    use rust_arkitect::dsl::Arkitect;
    use rust_arkitect::dsl::Project;

    #[test]
    fn test_architectural_rules() {
        Arkitect::init_logger();

        let project = Project::from_relative_path(file!(), "./../../../acme_project/src");

        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .component("Utils")
                .located_at("crate::utils")
                .allow_external_dependencies(&[])
                .must_not_depend_on_anything()

            .component("Errors")
                .located_at("crate::errors")
                .allow_external_dependencies(&[])
                .must_not_depend_on_anything()

            .component("EventSourcing")
                .located_at("crate::event_sourcing")
                .allow_external_dependencies(&["std::fmt"])
                .must_not_depend_on_anything()

            .component("Application")
                .located_at("crate::application")
                .allow_external_dependencies(&["std"])
                .may_depend_on(&["Domain", "Errors", "Utils"])

            .component("Domain")
                .located_at("crate::domain")
                .allow_external_dependencies(&["std", "chrono", "log", "serde", "uuid"])
                .may_depend_on(&["EventSourcing", "Errors"])

            .component("Infrastructure")
                .located_at("crate::infrastructure")
                .allow_external_dependencies(&["serde"])
                .may_depend_on(&["Domain", "Application", "Errors", "EventSourcing", "Utils"])

            .finalize();

        let result = Arkitect::ensure_that(project).complies_with(rules);

        assert!(
            result.is_ok(),
            "Detected {} violations",
            result.err().unwrap().len()
        );
    }
}
