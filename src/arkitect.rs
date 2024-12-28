use crate::validation::Rule;
use ansi_term::Color::RGB;
use ansi_term::Style;
use std::fs;

pub struct Project {
    path: String,
}

impl Project {
    pub fn load(path: &str) -> Project {
        Project {
            path: path.to_string(),
        }
    }
}

pub struct Arkitect {
    project: Project,
}

impl Arkitect {
    pub fn complies_with(&mut self, rules: Vec<Box<dyn Rule>>) -> Result<(), Vec<String>> {
        run(&self.project, rules)
    }
}

impl Arkitect {
    pub fn ensure_that(project: Project) -> Arkitect {
        Arkitect { project }
    }
}

fn run(project: &Project, rules: Vec<Box<dyn Rule>>) -> Result<(), Vec<String>> {
    let mut violations = vec![];

    validate_dir(project.path.as_str(), &rules, &mut violations);

    if violations.is_empty() {
        return Ok(());
    }

    Err(violations)
}

fn apply_rules(file: std::path::PathBuf, rules: &[Box<dyn Rule>], violations: &mut Vec<String>) {
    let file_name = file.to_str().unwrap();
    let bold = Style::new().bold().fg(RGB(0, 255, 0));
    let red = Style::new().fg(RGB(255, 0, 0));
    println!("🛠️ Applying rules to {}", bold.paint(file_name));
    for rule in rules {
        if rule.is_applicable(file_name) {
            println!("\t🟢 Rule {} applied", rule);
            match rule.apply(file_name) {
                Ok(_) => println!("\t\u{2705} Rule {} respected", rule),
                Err(e) => {
                    println!("\t🟥 Rule {} violated: {}", rule, red.paint(e.clone()));
                    violations.push(e)
                }
            }
        } else {
            println!("\t❌ Rule {} not applied", rule);
        }
    }
}

fn validate_dir(dir: &str, rules: &[Box<dyn Rule>], violations: &mut Vec<String>) {
    let entries =
        fs::read_dir(dir).expect(format!("Error reading root directory '{}'", dir).as_str());

    for file in entries {
        match file {
            Ok(file) => {
                if file.metadata().unwrap().is_dir() {
                    validate_dir(file.path().to_str().unwrap(), rules, violations);
                } else {
                    apply_rules(file.path(), rules, violations);
                }
            }
            Err(_) => panic!("Error reading file"),
        }
    }
}
