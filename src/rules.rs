use crate::dependency_parsing::{get_dependencies_in_file, get_module};
use ansi_term::Color::RGB;
use ansi_term::Style;
use log::debug;
use std::fmt::{Display, Formatter};

pub trait Rule: Display {
    fn apply(&self, file: &str) -> Result<(), String>;

    fn is_applicable(&self, file: &str) -> bool;
}

#[derive(Debug)]
pub struct MustNotDependOnAnythingRule {
    pub(crate) subject: String,
    pub(crate) allowed_external_dependencies: Vec<String>,
}

impl Display for MustNotDependOnAnythingRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut allowed_dependencies: Vec<String> = Vec::new();
        allowed_dependencies.extend(self.allowed_external_dependencies.clone());
        if allowed_dependencies.is_empty() {
            write!(f, "{} may not depend on any modules", self.subject)
        } else {
            let bold = Style::new().bold().fg(ansi_term::Color::RGB(255, 165, 0));
            write!(
                f,
                "{} may depend on {}",
                bold.paint(self.subject.clone()),
                bold.paint("[".to_string() + &allowed_dependencies.join(", ") + "]")
            )
        }
    }
}

impl Rule for MustNotDependOnAnythingRule {
    fn apply(&self, file: &str) -> Result<(), String> {
        let dependencies = get_dependencies_in_file(file);

        let forbidden_dependencies: Vec<String> = dependencies
            .iter()
            .filter(|&dependency| {
                !(is_child(self.subject.clone(), dependency.clone())
                    || self
                        .allowed_external_dependencies
                        .iter()
                        .any(|allowed| is_child(allowed.to_string(), dependency.clone())))
            })
            .cloned()
            .collect();

        if forbidden_dependencies.is_empty() {
            Ok(())
        } else {
            let red = Style::new().fg(RGB(255, 0, 0)).bold();
            Err(format!(
                "Forbidden dependencies to {} in file://{}",
                red.paint("[".to_string() + &forbidden_dependencies.join(", ") + "]"),
                file
            ))
        }
    }

    fn is_applicable(&self, file: &str) -> bool {
        let module = get_module(file);

        is_child(self.subject.clone(), module.unwrap())
    }
}

fn is_child(module: String, child: String) -> bool {
    child.starts_with(module.as_str())
}

#[derive(Debug)]
pub struct  MayDependOnRule {
    pub(crate) subject: String,
    pub(crate) allowed_dependencies: Vec<String>,
    pub(crate) allowed_external_dependencies: Vec<String>,
}

impl Display for MayDependOnRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut allowed_dependencies: Vec<String> = Vec::new();
        allowed_dependencies.extend(self.allowed_dependencies.clone());
        allowed_dependencies.extend(self.allowed_external_dependencies.clone());
        if allowed_dependencies.is_empty() {
            write!(f, "{} may not depend on any modules", self.subject)
        } else {
            let bold = Style::new().bold().fg(ansi_term::Color::RGB(255, 165, 0));
            write!(
                f,
                "{} may depend on {}",
                bold.paint(self.subject.clone()),
                bold.paint("[".to_string() + &allowed_dependencies.join(", ") + "]")
            )
        }
    }
}

impl Rule for MayDependOnRule {
    fn apply(&self, file: &str) -> Result<(), String> {
        let module = get_module(file).unwrap();
        let subject = if module.len() > self.subject.len() {
            self.subject.clone()
        } else {
            module
        };

        let dependencies = get_dependencies_in_file(file);

        let forbidden_dependencies: Vec<String> = dependencies
            .iter()
            .filter(|&dependency| {
                let is_child_of_subject = is_child(subject.clone(), dependency.clone());
                if !is_child_of_subject {
                    let is_allowed = self
                        .allowed_dependencies
                        .iter()
                        .any(|ad| is_child(ad.clone(), dependency.clone()));
                    let is_allowed_external = self
                        .allowed_external_dependencies
                        .iter()
                        .any(|ad| is_child(ad.clone(), dependency.clone()));
                    if !(is_allowed || is_allowed_external) {
                        return true;
                    }
                }

                false
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
        let orange = Style::new().bold().fg(ansi_term::Color::RGB(255, 165, 0));
        let green = Style::new().bold().fg(ansi_term::Color::RGB(0, 255, 0));
        let module = get_module(file).unwrap();
        debug!(
            "File {} mapped to module {}",
            green.paint(file),
            orange.paint(module.clone())
        );
        is_child(self.subject.clone(), module)
    }
}

#[test]
fn test_dependency_rule() {
    let rule = MayDependOnRule {
        subject: "policy_management::domain".to_string(),
        allowed_dependencies: vec!["conversion::domain::domain_function_1".to_string()],
        allowed_external_dependencies: vec!["chrono".to_string()],
    };

    let result =
        rule.apply("./../rust_arkitect/examples/sample_project/src/conversion/application.rs");

    assert!(result.is_err());
}
