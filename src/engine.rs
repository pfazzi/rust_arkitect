use crate::rules::Rule;
use ansi_term::Color::RGB;
use ansi_term::Style;
use log::{debug, error, info};
use std::fs;
use std::path::Path;

pub(crate) fn run(absolute_path: &str, rules: Vec<Box<dyn Rule>>) -> Result<(), Vec<String>> {
    let mut violations = vec![];

    validate_dir(absolute_path, &rules, &mut violations);

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
                    error!("🟥 Rule {} violated: {}", rule, e.clone());
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