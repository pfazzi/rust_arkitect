use crate::rules::Rule;
use ansi_term::Color::RGB;
use ansi_term::Style;
use log::{debug, error, info};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

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
        if is_workspace(self.absolute_path).is_ok() {
            self.validate_workspace(self.absolute_path);
        } else if is_crate(self.absolute_path).is_ok() {
            self.validate_dir(self.absolute_path);
        } else {
            panic!("The path '{}' is not a workspace or crate", self.absolute_path);
        }

        self.violations
    }

    fn validate_workspace(&mut self, workspace_path: &str) {
        let cargo_toml_path = Path::new(workspace_path).join("Cargo.toml");

        let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
            .unwrap_or_else(|_| panic!("Failed to read Cargo.toml in '{}'", workspace_path));

        let parsed: Value = toml::from_str(&cargo_toml_content)
            .unwrap_or_else(|_| panic!("Failed to parse Cargo.toml in '{}'", workspace_path));

        let members = parsed
            .get("workspace")
            .and_then(|workspace| workspace.get("members"))
            .and_then(|members| members.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|member| member.as_str())
            .map(String::from)
            .collect::<Vec<String>>();

        for member in members {
            let member_path = Path::new(workspace_path).join(&member);
            if member_path.is_dir() {
                if is_crate(member_path.to_str().unwrap()).is_ok() {
                    self.validate_dir(member_path.to_str().unwrap());
                } else {
                    debug!("Skipping invalid crate '{}'", member_path.display());
                }
            }
        }
    }

    fn validate_dir(&mut self, dir: &str) {
        let entries = fs::read_dir(dir).unwrap_or_else(|_| panic!("Error reading root directory '{}'", dir));

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

    fn apply_rules(&mut self, file: PathBuf) {
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

fn is_crate(path: &str) -> Result<(), String> {
    let dir_path = Path::new(path);

    is_directory(path)?;

    if !dir_path.join("Cargo.toml").exists() {
        return Err(format!("'{}' is not a valid Rust crate (missing Cargo.toml)", path));
    }

    Ok(())
}

fn is_workspace(path: &str) -> Result<(), String> {
    let dir_path = Path::new(path);

    is_directory(path)?;

    let cargo_toml_path = dir_path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(format!("'{}' is not a valid Rust workspace (missing Cargo.toml)", path));
    }

    let cargo_toml_content = fs::read_to_string(cargo_toml_path)
        .map_err(|_| format!("Failed to read Cargo.toml in '{}'", path))?;
    if !cargo_toml_content.contains("[workspace]") {
        return Err(format!(
            "'{}' is not a Rust workspace (missing [workspace] key in Cargo.toml)",
            path
        ));
    }

    Ok(())
}

fn is_directory(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !path.is_dir() {
        return Err(format!("'{}' is not a valid directory", path.display()));
    }
    Ok(())
}
