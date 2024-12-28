use crate::validation::{MayDependOnRule, MustNotDependOnAnythingRule, Rule};
use std::collections::HashMap;

pub struct ArchitecturalRules {
    current_component: String,
    components: HashMap<String, String>,
    rules: Vec<Box<dyn Rule>>,
    may_depend_on_rule: Vec<Box<MayDependOnRule>>,
}

impl ArchitecturalRules {
    pub fn define() -> Self {
        Self {
            current_component: "".to_string(),
            components: HashMap::new(),
            rules: Vec::new(),
            may_depend_on_rule: Vec::new(),
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
        }));

        self
    }

    pub fn may_depend_on(mut self, dependencies: &[&str]) -> Self {
        self.may_depend_on_rule.push(Box::new(MayDependOnRule {
            subject: self
                .components
                .get(&self.current_component.clone())
                .expect("Component must not be empty")
                .clone(),
            allowed_dependencies: dependencies.iter().map(|&s| String::from(s)).collect(),
        }));

        self
    }

    pub fn finalize(mut self) -> Vec<Box<dyn Rule>> {
        let may_depend_on_rules: Vec<Box<dyn Rule>> = self
            .may_depend_on_rule
            .iter()
            .map(|rule| {
                Box::new(MayDependOnRule {
                    subject: rule.subject.clone(),
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
}
