#![cfg(test)]

use rust_arkitect::dsl::arkitect::Arkitect;
use rust_arkitect::dsl::project::Project;
use rust_arkitect::rules::must_not_depend_on::MustNotDependOnRule;
use rust_arkitect::rules::rule::Rule;
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
    fn apply(&self, _file: &str) -> Result<(), String> {
        Ok(())
    }

    fn is_applicable(&self, _file: &str) -> bool {
        true
    }
}

#[test]
fn test_custom_rule_execution() {
    let project = Project::new();

    let rule = Box::new(TestRule::new("my_crate", &["a:crate::a_module"]));

    let result = Arkitect::ensure_that(project).complies_with(vec![rule]);

    assert!(result.is_ok());
}

#[test]
fn test_may_depend_on_standalone() {
    let project = Project::new();

    let rule = MustNotDependOnRule::new(
        "conversion::domain".to_string(),
        vec!["a:crate::a_module".to_string()],
    );

    let result = Arkitect::ensure_that(project).complies_with(vec![rule.into()]);

    assert!(result.is_ok());
}
