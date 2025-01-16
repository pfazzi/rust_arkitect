#![cfg(test)]

use rust_arkitect::dsl::Arkitect;
use rust_arkitect::dsl::Project;
use rust_arkitect::rules::Rule;
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
fn test_rule_execution() {
    let project = Project::from_absolute_path(
        "/Users/patrickfazzi/Projects/rust_arkitect/examples/sample_project",
    );

    let rule = Box::new(TestRule::new("my_crate", &["a:crate::a_module"]));

    let result = Arkitect::ensure_that(project).complies_with(vec![rule]);

    assert!(result.is_ok());
}
