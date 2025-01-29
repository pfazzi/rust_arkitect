use crate::rule::Rule;
use crate::rules::utils::IsChild;
use crate::rust_file::RustFile;
use ansi_term::Color::RGB;
use ansi_term::Style;
use log::debug;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct MayDependOnRule {
    pub subject: String,
    pub allowed_dependencies: Vec<String>,
}

impl Display for MayDependOnRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut allowed_dependencies: Vec<String> = Vec::new();
        allowed_dependencies.extend(self.allowed_dependencies.clone());
        let bold = Style::new().bold().fg(RGB(255, 165, 0));
        if allowed_dependencies.is_empty() {
            write!(
                f,
                "{} may not depend on any modules",
                bold.paint(&self.subject)
            )
        } else {
            write!(
                f,
                "{} may depend on {}",
                bold.paint(&self.subject),
                bold.paint("[".to_string() + &allowed_dependencies.join(", ") + "]")
            )
        }
    }
}

impl Rule for MayDependOnRule {
    fn apply(&self, file: &RustFile) -> Result<(), String> {
        let forbidden_dependencies: Vec<String> = file
            .dependencies
            .iter()
            .filter(|&dependency| {
                let is_child_of_subject = dependency.is_child_of(&self.subject);
                if !is_child_of_subject {
                    let is_allowed = self
                        .allowed_dependencies
                        .iter()
                        .any(|ad| dependency.is_child_of(ad));
                    if !is_allowed {
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
                file.path
            ));
        }

        Ok(())
    }

    fn is_applicable(&self, file: &RustFile) -> bool {
        let orange = Style::new().bold().fg(RGB(255, 165, 0));
        let green = Style::new().bold().fg(RGB(0, 255, 0));
        debug!(
            "File {} mapped to module {}",
            green.paint(&file.path),
            orange.paint(&file.logical_path)
        );
        file.logical_path.is_child_of(&self.subject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust_file::RustFile;

    #[test]
    fn test_dependency_rule() {
        let rule = MayDependOnRule {
            subject: "policy_management::domain".to_string(),
            allowed_dependencies: vec!["conversion::domain::domain_function_1".to_string()],
        };

        let result = rule.apply(&RustFile::from_file_system(
            "./../rust_arkitect/examples/sample_project/src/conversion/application.rs",
        ));

        assert!(result.is_err());
    }

    #[test]
    fn test_display_may_depend_on_with_dependencies() {
        use ansi_term::Color::RGB;
        use ansi_term::Style;

        let rule = MayDependOnRule {
            subject: "module_3".to_string(),
            allowed_dependencies: vec!["dependency_a".to_string(), "dependency_b".to_string()],
        };

        let bold_orange = Style::new().bold().fg(RGB(255, 165, 0));
        let expected = format!(
            "{} may depend on {}",
            bold_orange.paint("module_3"),
            bold_orange.paint("[dependency_a, dependency_b]")
        );
        assert_eq!(format!("{}", rule), expected);
    }

    #[test]
    fn test_display_may_depend_on_no_dependencies() {
        use ansi_term::Color::RGB;
        use ansi_term::Style;

        let rule = MayDependOnRule {
            subject: "module_4".to_string(),
            allowed_dependencies: vec![],
        };

        let bold_orange = Style::new().bold().fg(RGB(255, 165, 0));
        let expected = format!(
            "{} may not depend on any modules",
            bold_orange.paint("module_4")
        );
        assert_eq!(format!("{}", rule), expected);
    }
}
