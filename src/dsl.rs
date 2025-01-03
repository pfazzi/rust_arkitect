use crate::rules::{MayDependOnRule, MustNotDependOnAnythingRule, Rule};
use ansi_term::Color::RGB;
use ansi_term::Style;
use log::{debug, error, info};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Project {
    absolute_path: String,
}

impl Project {
    pub fn from_absolute_path(absolute_path: &str) -> Project {
        Project {
            absolute_path: absolute_path.to_string(),
        }
    }

    pub fn from_relative_path(current_file: &str, relative_path: &str) -> Project {
        let current_dir = Path::new(current_file)
            .parent()
            .expect("Failed to get parent directory");

        let derived_path = current_dir.join(relative_path);
        let absolute_path = derived_path.canonicalize().unwrap_or_else(|e| {
            panic!(
                "Failed to resolve absolute path. Derived path: '{}', from current file: '{}' and relative path: '{}'. Error: {}",
                derived_path.display(),
                current_file,
                relative_path,
                e
            )
        });

        Project {
            absolute_path: absolute_path
                .to_str()
                .expect("Failed to convert path to string")
                .to_string(),
        }
    }
}

pub struct Arkitect {
    project: Project,
}

impl Arkitect {
    pub fn init_logger() {
        let _ = env_logger::builder().is_test(false).try_init();
    }

    pub fn complies_with(&mut self, rules: Vec<Box<dyn Rule>>) -> Result<(), Vec<String>> {
        run(&self.project, rules)
    }
}

impl Arkitect {
    pub fn ensure_that(project: Project) -> Arkitect {
        Arkitect { project }
    }
}
pub struct ArchitecturalRules {
    components: HashMap<String, String>,
    rules: Vec<Box<dyn Rule>>,

    may_depend_on_rule: Vec<Box<MayDependOnRule>>,

    current_component: String,
    allowed_external_dependencies: Vec<String>,
    allowed_dependencies: Vec<String>,
}

impl ArchitecturalRules {
    pub fn allow_external_dependencies(mut self, external_dependencies: &[&str]) -> Self {
        self.allowed_external_dependencies = external_dependencies
            .iter()
            .map(|&s| s.to_string())
            .collect();

        self
    }
}

impl ArchitecturalRules {
    pub fn define() -> Self {
        Self {
            current_component: "".to_string(),
            components: HashMap::new(),
            rules: Vec::new(),
            may_depend_on_rule: Vec::new(),
            allowed_external_dependencies: Vec::new(),
            allowed_dependencies: Vec::new(),
        }
    }

    pub fn component(mut self, component: &str) -> Self {
        self.current_component = String::from(component);

        self
    }

    pub fn located_at(mut self, module: &str) -> Self {
        self.components
            .insert(self.current_component.clone(), String::from(module));

        self
    }

    pub fn must_not_depend_on_anything(mut self) -> Self {
        self.rules.push(Box::new(MustNotDependOnAnythingRule {
            subject: self
                .components
                .get(&self.current_component.clone())
                .expect("Component must not be empty")
                .clone(),
            allowed_external_dependencies: self.allowed_external_dependencies.clone(),
        }));

        self.reset();

        self
    }

    pub fn may_depend_on(mut self, dependencies: &[&str]) -> Self {
        let rule = MayDependOnRule {
            subject: self
                .components
                .get(&self.current_component.clone())
                .expect("Component must not be empty")
                .clone(),
            allowed_dependencies: dependencies
                .clone()
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            allowed_external_dependencies: self.allowed_external_dependencies.clone(),
        };

        self.may_depend_on_rule.push(Box::new(rule));

        self.reset();

        self
    }

    pub fn finalize(mut self) -> Vec<Box<dyn Rule>> {
        let may_depend_on_rules: Vec<Box<dyn Rule>> = self
            .may_depend_on_rule
            .iter()
            .map(|rule| {
                Box::new(MayDependOnRule {
                    subject: rule.subject.clone(),
                    allowed_external_dependencies: rule.allowed_external_dependencies.clone(),
                    allowed_dependencies: rule
                        .allowed_dependencies
                        .iter()
                        .map(|d| self.components.get(d).unwrap().clone())
                        .collect(),
                }) as Box<dyn Rule>
            })
            .collect();

        self.rules.extend(may_depend_on_rules);

        self.rules
    }

    fn reset(&mut self) {
        self.current_component = String::new();
        self.allowed_external_dependencies.clear();
        self.allowed_dependencies.clear();
    }
}

fn run(project: &Project, rules: Vec<Box<dyn Rule>>) -> Result<(), Vec<String>> {
    let mut violations = vec![];

    validate_dir(project.absolute_path.as_str(), &rules, &mut violations);

    if violations.is_empty() {
        return Ok(());
    }

    Err(violations)
}

fn apply_rules(file: std::path::PathBuf, rules: &[Box<dyn Rule>], violations: &mut Vec<String>) {
    let file_name = file.to_str().unwrap();
    let bold = Style::new().bold().fg(RGB(0, 255, 0));
    let absolute_file_name = file
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "Unknown file".to_string());

    info!("🛠️Applying rules to {}", bold.paint(absolute_file_name));
    for rule in rules {
        if rule.is_applicable(file_name) {
            debug!("🟢 Rule {} applied", rule);
            match rule.apply(file_name) {
                Ok(_) => info!("\u{2705} Rule {} respected", rule),
                Err(e) => {
                    error!("🟥 Rule {} violated: {}", rule, e.clone().to_lowercase());
                    violations.push(e)
                }
            }
        } else {
            debug!("❌ Rule {} not applied", rule);
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
                } else if file.path().extension().map_or(false, |ext| ext == "rs") {
                    apply_rules(file.path(), rules, violations);
                }
            }
            Err(_) => panic!("Error reading file"),
        }
    }
}
