use crate::dependency_parsing::{get_dependencies_in_file, get_module};
use crate::rules::rule::Rule;
use crate::rules::utils::IsChild;
use ansi_term::Color::RGB;
use ansi_term::Style;
use log::debug;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct MustNotDependOnRule {
    pub(crate) subject: String,
    pub(crate) forbidden_dependencies: Vec<String>,
}

impl MustNotDependOnRule {
    pub fn new(subject: String, forbidden_dependencies: Vec<String>) -> Self {
        Self {
            subject,
            forbidden_dependencies,
        }
    }
}

impl Display for MustNotDependOnRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bold = Style::new().bold().fg(RGB(255, 165, 0));
        if self.forbidden_dependencies.is_empty() {
            write!(f, "{} may depend on any module", bold.paint(&self.subject))
        } else {
            write!(
                f,
                "{} must not depend on {}",
                bold.paint(&self.subject),
                bold.paint("[".to_string() + &self.forbidden_dependencies.join(", ") + "]")
            )
        }
    }
}

impl Rule for MustNotDependOnRule {
    fn apply(&self, file: &str) -> Result<(), String> {
        let dependencies = get_dependencies_in_file(file);

        let forbidden_dependencies: Vec<String> = dependencies
            .iter()
            .filter(|&dependency| {
                self.forbidden_dependencies
                    .iter()
                    .any(|ad| dependency.is_child_of(ad))
            })
            .cloned()
            .collect();

        if !forbidden_dependencies.is_empty() {
            let red = Style::new().fg(RGB(255, 0, 0)).bold();
            return Err(format!(
                "Forbidden dependencies to {} in file://{}",
                red.paint("[".to_string() + &forbidden_dependencies.join(", ") + "]"),
                file
            ));
        }

        Ok(())
    }

    fn is_applicable(&self, file: &str) -> bool {
        let orange = Style::new().bold().fg(RGB(255, 165, 0));
        let green = Style::new().bold().fg(RGB(0, 255, 0));
        let module = get_module(file).unwrap();
        debug!(
            "File {} mapped to module {}",
            green.paint(file),
            orange.paint(&module)
        );
        module.is_child_of(&self.subject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_rule_err() {
        let rule = MustNotDependOnRule {
            subject: "sample_project::conversion".to_string(),
            forbidden_dependencies: vec!["sample_project::contracts".to_string()],
        };

        let result =
            rule.apply("./../rust_arkitect/examples/sample_project/src/conversion/application.rs");

        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_rule_ok() {
        let rule = MustNotDependOnRule {
            subject: "sample_project::conversion".to_string(),
            forbidden_dependencies: vec!["sample_project::policy_management".to_string()],
        };

        let result =
            rule.apply("./../rust_arkitect/examples/sample_project/src/conversion/application.rs");

        assert!(result.is_ok());
    }

    #[test]
    fn test_display_may_depend_on_with_dependencies() {
        use ansi_term::Color::RGB;
        use ansi_term::Style;

        let rule = MustNotDependOnRule {
            subject: "module_3".to_string(),
            forbidden_dependencies: vec!["dependency_a".to_string(), "dependency_b".to_string()],
        };

        let bold_orange = Style::new().bold().fg(RGB(255, 165, 0));
        let expected = format!(
            "{} must not depend on {}",
            bold_orange.paint("module_3"),
            bold_orange.paint("[dependency_a, dependency_b]")
        );
        assert_eq!(format!("{}", rule), expected);
    }

    #[test]
    fn test_display_may_depend_on_no_dependencies() {
        use ansi_term::Color::RGB;
        use ansi_term::Style;

        let rule = MustNotDependOnRule {
            subject: "module_4".to_string(),
            forbidden_dependencies: vec![],
        };

        let bold_orange = Style::new().bold().fg(RGB(255, 165, 0));
        let expected = format!("{} may depend on any module", bold_orange.paint("module_4"));
        assert_eq!(format!("{}", rule), expected);
    }
}
