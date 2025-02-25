#![cfg(test)]

use rust_arkitect::builtin_rules::must_not_depend_on::MustNotDependOnRule;
use rust_arkitect::dsl::arkitect::{Arkitect, Rules};
use rust_arkitect::dsl::project::Project;
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

#[test]
fn test_custom_rule_execution() {
    let project = Project::new();

    let rule = Box::new(TestRule::new("my_crate", &["a:crate::a_module"]));

    let result = Arkitect::ensure_that(project).complies_with(Rules::from_module_rules(vec![rule]));

    assert!(result.is_ok());
}

#[test]
fn test_may_depend_on_standalone() {
    let project = Project::new();

    let rule = MustNotDependOnRule::new(
        "conversion::domain".to_string(),
        vec!["a:crate::a_module".to_string()],
    );

    let result =
        Arkitect::ensure_that(project).complies_with(Rules::from_module_rules(vec![rule.into()]));

    assert!(result.is_ok());
}
