use crate::rules::may_depend_on::MayDependOnRule;
use crate::rules::must_not_depend_on::MustNotDependOnRule;
use crate::rules::must_not_depend_on_anything::MustNotDependOnAnythingRule;
use crate::rules::rule::Rule;
use std::marker::PhantomData;

pub struct Begin;
pub struct SubjectDefined;
pub struct RulesDefined;
pub struct ArchitecturalRules<State> {
    state: PhantomData<State>,
    current_subject: Option<String>,
    rules: Vec<Box<dyn Rule>>,
}

impl ArchitecturalRules<Begin> {
    pub fn define() -> Self {
        Self {
            state: PhantomData,
            current_subject: None,
            rules: vec![],
        }
    }

    pub fn rules_for_crate(self, crate_name: &str) -> ArchitecturalRules<SubjectDefined> {
        ArchitecturalRules {
            state: PhantomData,
            current_subject: Some(String::from(crate_name)),
            rules: self.rules,
        }
    }

    pub fn rules_for_module(self, crate_name: &str) -> ArchitecturalRules<SubjectDefined> {
        ArchitecturalRules {
            state: PhantomData,
            current_subject: Some(String::from(crate_name)),
            rules: self.rules,
        }
    }
}

impl ArchitecturalRules<SubjectDefined> {
    pub fn may_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MayDependOnRule {
            subject: self.current_subject.clone().unwrap(),
            allowed_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
            allowed_external_dependencies: vec![],
        });

        let mut rules = self.rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules,
        }
    }

    pub fn must_not_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MustNotDependOnRule {
            subject: self.current_subject.clone().unwrap(),
            forbidden_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
        });

        let mut rules = self.rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules,
        }
    }

    pub fn must_not_depend_on_anything(self) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MustNotDependOnAnythingRule {
            subject: self.current_subject.clone().unwrap(),
            allowed_external_dependencies: vec![],
        });

        let mut rules = self.rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules,
        }
    }

    pub fn add_rule(self, rule: Box<dyn Rule>) -> ArchitecturalRules<RulesDefined> {
        let mut rules = self.rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules,
        }
    }
}

impl ArchitecturalRules<RulesDefined> {
    pub fn may_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MayDependOnRule {
            subject: self.current_subject.clone().unwrap(),
            allowed_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
            allowed_external_dependencies: vec![],
        });

        let mut rules = self.rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules,
        }
    }

    pub fn must_not_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MustNotDependOnRule {
            subject: self.current_subject.clone().unwrap(),
            forbidden_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
        });

        let mut rules = self.rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules,
        }
    }

    pub fn must_not_depend_on_anything(self) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MustNotDependOnAnythingRule {
            subject: self.current_subject.clone().unwrap(),
            allowed_external_dependencies: vec![],
        });

        let mut rules = self.rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules,
        }
    }

    pub fn add_rule(self, rule: Box<dyn Rule>) -> ArchitecturalRules<RulesDefined> {
        let mut rules = self.rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules,
        }
    }

    pub fn rules_for_crate(self, crate_name: &str) -> ArchitecturalRules<SubjectDefined> {
        ArchitecturalRules {
            state: PhantomData,
            current_subject: Some(String::from(crate_name)),
            rules: self.rules,
        }
    }

    pub fn rules_for_module(self, crate_name: &str) -> ArchitecturalRules<SubjectDefined> {
        ArchitecturalRules {
            state: PhantomData,
            current_subject: Some(String::from(crate_name)),
            rules: self.rules,
        }
    }

    pub fn build(self) -> Vec<Box<dyn Rule>> {
        self.rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_rules_for_crate() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
                .rules_for_crate("application")
                .may_depend_on(&["std::fmt", "domain"])
            .build();

        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_define_rules_for_module() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
                .rules_for_module("domain::services")
                .may_depend_on(&["std::sync", "application"])
            .build();

        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_module_isolation() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .rules_for_module("domain::models")
                .must_not_depend_on(&["std::sync", "application"])
            .build();

        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_with_custom_rules() {
        let custom_rule = Box::new(MayDependOnRule {
            subject: "my_app".to_string(),
            allowed_dependencies: vec![],
            allowed_external_dependencies: vec![],
        });

        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .rules_for_crate("application")
                .may_depend_on(&["my_app", "domain"])
                .add_rule(custom_rule)
            .build();

        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_complex_rule_set() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .rules_for_crate("application")
                .may_depend_on(&["std::fmt", "domain"])
            .rules_for_module("domain::services")
                .may_depend_on(&["std::sync", "application"])
            .rules_for_module("domain::models")
                .must_not_depend_on_anything()
            .build();

        assert_eq!(rules.len(), 3);
    }
}
