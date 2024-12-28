use crate::dependency_parsing::{get_module, parse_dependencies};
use ansi_term::Style;
use std::fmt::{Display, Formatter};

pub trait Rule: Display {
    fn apply(&self, file: &str) -> Result<(), String>;

    fn is_applicable(&self, file: &str) -> bool;
}

pub struct MustNotDependOnAnythingRule {
    pub subject: String,
}

impl Display for MustNotDependOnAnythingRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bold = Style::new().bold().fg(ansi_term::Color::RGB(255, 165, 0));
        write!(
            f,
            "{} must not depend on any modules",
            bold.paint(self.subject.clone())
        )
    }
}

impl Rule for MustNotDependOnAnythingRule {
    fn apply(&self, file: &str) -> Result<(), String> {
        let dependencies = parse_dependencies(file);

        match dependencies.is_empty() {
            true => Ok(()),
            false => Err(format!(
                "Dependencies are not allowed on anything for \"{}\"",
                file
            )),
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

pub struct MayDependOnRule {
    pub(crate) subject: String,
    pub(crate) allowed_dependencies: Vec<String>,
}

impl Display for MayDependOnRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.allowed_dependencies.is_empty() {
            write!(f, "{} may not depend on any modules", self.subject)
        } else {
            let bold = Style::new().bold().fg(ansi_term::Color::RGB(255, 165, 0));
            write!(
                f,
                "{} may depend on {}",
                bold.paint(self.subject.clone()),
                bold.paint("[".to_string() + &self.allowed_dependencies.join(", ") + "]")
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

        let dependencies = parse_dependencies(file);

        for dependency in dependencies {
            let is_child_of_subject = is_child(subject.clone(), dependency.clone());
            if !is_child_of_subject {
                let allowed: Option<&String> = self
                    .allowed_dependencies
                    .iter()
                    .find(|&ad| is_child(ad.clone(), dependency.clone()));
                if allowed.is_none() {
                    return Err(String::from(format!(
                        "dependency not allowed on {} in file file://{}",
                        dependency, file
                    )));
                }
            }
        }

        Ok(())
    }

    fn is_applicable(&self, file: &str) -> bool {
        let orange = Style::new().bold().fg(ansi_term::Color::RGB(255, 165, 0));
        let green = Style::new().bold().fg(ansi_term::Color::RGB(0, 255, 0));
        let module = get_module(file).unwrap();
        println!(
            "ℹ File {} mapped to module {}",
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
    };

    let result = rule.apply(
        "/Users/patrickfazzi/Projects/rust_arkitect/sample_project/src/conversion/application.rs",
    );

    assert!(result.is_err());
}
