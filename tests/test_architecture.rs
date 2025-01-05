#[cfg(test)]
mod tests {
    use rust_arkitect::dsl::{ArchitecturalRules, Arkitect, Project};
    use std::result;

    #[test]
    fn test_compliance() {
        Arkitect::init_logger();

        let project = Project::from_relative_path(file!(), "./../src");

        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .component("DSL")
                .located_at("crate::dsl")
                .may_depend_on(&["Engine", "Rules"])

            .component("Engine")
                .located_at("crate::engine")
                .may_depend_on(&["Rules"])

            .component("Rules")
                .located_at("crate::rules")
                .may_depend_on(&["DependencyParsing"])

            .component("DependencyParsing")
                .located_at("crate::dependency_parsing")
                .must_not_depend_on_anything()

            .finalize();

        let result = Arkitect::ensure_that(project).complies_with(rules);

        assert!(
            result.is_ok(),
            "Detected {} violations",
            result.err().unwrap().len()
        );
    }
}
