use crate::builtin_rules::utils::IsChild;
use crate::rule::Rule;
use crate::rust_file::RustFile;
use ansi_term::Color::RGB;
use ansi_term::Style;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct MustNotDependOnAnythingRule {
    pub subject: String,
    pub allowed_external_dependencies: Vec<String>,
}

impl Display for MustNotDependOnAnythingRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut allowed_dependencies: Vec<String> = Vec::new();
        allowed_dependencies.extend(self.allowed_external_dependencies.clone());
        let bold = Style::new().bold().fg(ansi_term::Color::RGB(255, 165, 0));
        if allowed_dependencies.is_empty() {
            write!(
                f,
                "{} may not depend on any modules",
                bold.paint(&self.subject),
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

impl Rule for MustNotDependOnAnythingRule {
    fn apply(&self, file: &RustFile) -> Result<(), String> {
        let forbidden_dependencies: Vec<String> = file
            .dependencies
            .iter()
            .filter(|&dependency| {
                !(dependency.is_child_of(&self.subject)
                    || self
                        .allowed_external_dependencies
                        .iter()
                        .any(|allowed| dependency.is_child_of(allowed)))
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
                file.path
            ))
        }
    }

    fn is_applicable(&self, file: &RustFile) -> bool {
        file.logical_path.is_child_of(&self.subject)
    }
}

#[cfg(test)]
mod tests {
    use crate::builtin_rules::must_not_depend_on_anything::MustNotDependOnAnythingRule;

    #[test]
    fn test_display_must_not_depend_on_anything_no_dependencies() {
        use ansi_term::Color::RGB;
        use ansi_term::Style;

        let rule = MustNotDependOnAnythingRule {
            subject: "module_2".to_string(),
            allowed_external_dependencies: vec![],
        };

        let bold_orange = Style::new().bold().fg(RGB(255, 165, 0));
        let expected = format!(
            "{} may not depend on any modules",
            bold_orange.paint("module_2")
        );
        assert_eq!(format!("{}", rule), expected);
    }

    #[test]
    fn test_display_must_not_depend_on_anything_with_dependencies() {
        use ansi_term::Color::RGB;
        use ansi_term::Style;

        let rule = MustNotDependOnAnythingRule {
            subject: "module_1".to_string(),
            allowed_external_dependencies: vec![
                "dependency_1".to_string(),
                "dependency_2".to_string(),
            ],
        };

        let bold_orange = Style::new().bold().fg(RGB(255, 165, 0));
        let expected = format!(
            "{} may depend on {}",
            bold_orange.paint("module_1"),
            bold_orange.paint("[dependency_1, dependency_2]")
        );
        assert_eq!(format!("{}", rule), expected);
    }
}
