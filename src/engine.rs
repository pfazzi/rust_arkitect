use crate::rules::Rule;
use ansi_term::Color::RGB;
use ansi_term::Style;
use log::{debug, error, info};
use std::fs;

pub(crate) struct Engine<'a> {
    absolute_path: &'a str,
    rules: &'a [Box<dyn Rule>],
    violations: Vec<String>,
}

impl<'a> Engine<'a> {
    pub(crate) fn new(absolute_path: &'a str, rules: &'a [Box<dyn Rule>]) -> Self {
        Self {
            absolute_path,
            rules,
            violations: Default::default(),
        }
    }

    pub(crate) fn get_violations(mut self) -> Vec<String> {
        self.validate_dir(self.absolute_path);

        self.violations
    }

    fn validate_dir(&mut self, dir: &str) {
        let entries =
            fs::read_dir(dir).unwrap_or_else(|_| panic!("Error reading root directory '{}'", dir));

        for file in entries {
            match file {
                Ok(file) => {
                    if file.metadata().unwrap().is_dir() {
                        self.validate_dir(file.path().to_str().unwrap());
                    } else if file.path().extension().map_or(false, |ext| ext == "rs") {
                        self.apply_rules(file.path());
                    }
                }
                Err(_) => panic!("Error reading file"),
            }
        }
    }

    fn apply_rules(&mut self, file: std::path::PathBuf) {
        let file_name = file.to_str().unwrap();
        let bold = Style::new().bold().fg(RGB(0, 255, 0));
        let absolute_file_name = file
            .canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| "Unknown file".to_string());

        info!("🛠️Applying rules to {}", bold.paint(absolute_file_name));
        for rule in self.rules {
            if rule.is_applicable(file_name) {
                debug!("🟢 Rule {} applied", rule);
                match rule.apply(file_name) {
                    Ok(_) => info!("\u{2705} Rule {} respected", rule),
                    Err(e) => {
                        error!("🟥 Rule {} violated: {}", rule, e);
                        self.violations.push(e)
                    }
                }
            } else {
                debug!("❌ Rule {} not applied", rule);
            }
        }
    }
}
