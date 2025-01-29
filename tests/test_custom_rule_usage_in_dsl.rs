#![cfg(test)]

use rust_arkitect::rule::Rule;
use rust_arkitect::rust_file::RustFile;
use std::fmt::{Display, Formatter};

struct TestRule;

impl TestRule {
    fn new(_subject: &str, _dependencies: &[&str; 1]) -> TestRule {
        Self {}
    }
}

impl Display for TestRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestRule applied")
    }
}

impl Rule for TestRule {
    fn apply(&self, _file: &RustFile) -> Result<(), String> {
        Ok(())
    }

    fn is_applicable(&self, _file: &RustFile) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::TestRule;
    use rust_arkitect::dsl::arkitect::{Arkitect, Rules};
    use rust_arkitect::dsl::project::Project;
    use rust_arkitect::rule::Rule;
    use rust_arkitect::rules::must_not_depend_on::MustNotDependOnRule;

    #[test]
    fn test_custom_rule_execution() {
        let project = Project::new();

        let rule = Box::new(TestRule::new("my_crate", &["a:crate::a_module"]));

        let rules: Vec<Box<dyn Rule>> = vec![rule];

        let rules = Rules::from_module_rules(rules);

        let result = Arkitect::ensure_that(project).complies_with(rules);

        assert!(result.is_ok());
    }

    #[test]
    fn test_may_depend_on_standalone() {
        let project = Project::new();

        let rule = MustNotDependOnRule::new(
            "conversion::domain".to_string(),
            vec!["a:crate::a_module".to_string()],
        );

        let rules: Vec<Box<dyn Rule>> = vec![rule.into()];

        let rules = Rules::from_module_rules(rules);

        let result = Arkitect::ensure_that(project).complies_with(rules);

        assert!(result.is_ok());
    }
}
