use crate::builtin_rules::may_depend_on::MayDependOnRule;
use crate::builtin_rules::must_not_depend_on::MustNotDependOnRule;
use crate::builtin_rules::must_not_depend_on_anything::MustNotDependOnAnythingRule;
use crate::builtin_rules::must_not_have_circular_dependencies::MustNotHaveCircularDependencies;
use crate::dsl::arkitect::Rules;
use crate::rule::Rule;
use std::marker::PhantomData;

pub struct Begin;
pub struct SubjectDefined;
pub struct ProjectDefined;
pub struct ProjectBegin;
pub struct RulesDefined;

pub struct ArchitecturalRules<State> {
    state: PhantomData<State>,
    current_subject: Option<String>,
    rules: Rules,
}

pub trait SubjectInjectableRuleBuilder {
    fn for_subject(&self, subject: &str) -> Box<dyn Rule>;
}

impl ArchitecturalRules<Begin> {
    pub fn define() -> Self {
        Self {
            state: PhantomData,
            current_subject: None,
            rules: Rules {
                module_rules: vec![],
                project_rules: vec![],
            },
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

    pub fn rules_for_project(self) -> ArchitecturalRules<ProjectBegin> {
        ArchitecturalRules {
            state: PhantomData,
            current_subject: None,
            rules: self.rules,
        }
    }
}

impl ArchitecturalRules<ProjectBegin> {
    pub fn it_must_not_have_circular_dependencies(
        self,
        max_depth: usize,
    ) -> ArchitecturalRules<ProjectDefined> {
        let rule = Box::new(MustNotHaveCircularDependencies { max_depth });

        let mut project_rules = self.rules.project_rules;
        project_rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules: Rules {
                project_rules,
                ..self.rules
            },
        }
    }
}

impl ArchitecturalRules<ProjectDefined> {
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

    pub fn build(self) -> Rules {
        self.rules
    }
}

impl ArchitecturalRules<SubjectDefined> {
    pub fn it_may_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MayDependOnRule {
            subject: self.current_subject.clone().unwrap(),
            allowed_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
        });

        let mut rules = self.rules.module_rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules: Rules {
                module_rules: rules,
                ..self.rules
            },
        }
    }

    pub fn it_must_not_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MustNotDependOnRule {
            subject: self.current_subject.clone().unwrap(),
            forbidden_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
        });

        let mut rules = self.rules.module_rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules: Rules {
                module_rules: rules,
                ..self.rules
            },
        }
    }

    pub fn it_must_not_depend_on_anything(self) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MustNotDependOnAnythingRule {
            subject: self.current_subject.clone().unwrap(),
            allowed_external_dependencies: vec![],
        });

        let mut rules = self.rules.module_rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules: Rules {
                module_rules: rules,
                ..self.rules
            },
        }
    }

    pub fn it(
        self,
        rule: Box<dyn SubjectInjectableRuleBuilder>,
    ) -> ArchitecturalRules<RulesDefined> {
        let rule = rule.for_subject(self.current_subject.as_ref().unwrap());

        let mut rules = self.rules.module_rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules: Rules {
                module_rules: rules,
                ..self.rules
            },
        }
    }
}

impl ArchitecturalRules<RulesDefined> {
    pub fn and_it_may_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MayDependOnRule {
            subject: self.current_subject.clone().unwrap(),
            allowed_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
        });

        let mut rules = self.rules.module_rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules: Rules {
                module_rules: rules,
                ..self.rules
            },
        }
    }

    pub fn and_must_not_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MustNotDependOnRule {
            subject: self.current_subject.clone().unwrap(),
            forbidden_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
        });

        let mut rules = self.rules.module_rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules: Rules {
                module_rules: rules,
                ..self.rules
            },
        }
    }

    pub fn and_it_must_not_depend_on_anything(self) -> ArchitecturalRules<RulesDefined> {
        let rule = Box::new(MustNotDependOnAnythingRule {
            subject: self.current_subject.clone().unwrap(),
            allowed_external_dependencies: vec![],
        });

        let mut rules = self.rules.module_rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules: Rules {
                module_rules: rules,
                ..self.rules
            },
        }
    }

    pub fn and_it(
        self,
        rule: Box<dyn SubjectInjectableRuleBuilder>,
    ) -> ArchitecturalRules<RulesDefined> {
        let rule = rule.for_subject(self.current_subject.as_ref().unwrap());

        let mut rules = self.rules.module_rules;
        rules.push(rule);

        ArchitecturalRules {
            state: PhantomData,
            current_subject: self.current_subject,
            rules: Rules {
                module_rules: rules,
                ..self.rules
            },
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

    pub fn build(self) -> Rules {
        self.rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust_file::RustFile;
    use std::fmt::{Display, Formatter};

    #[test]
    fn test_define_rules_for_crate() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
                .rules_for_crate("application")
                .it_may_depend_on(&["std::fmt", "domain"])
            .build();

        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_define_rules_for_module() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
                .rules_for_module("domain::services")
                .it_may_depend_on(&["std::sync", "application"])
            .build();

        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_module_isolation() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .rules_for_module("domain::models")
                .it_must_not_depend_on(&["std::sync", "application"])
            .build();

        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_with_custom_rules() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .rules_for_crate("application")
                .it_may_depend_on(&["my_app", "domain"])
                .and_it(MustNotContainAttribute::new("#[a]"))
            .build();

        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_complex_rule_set() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .rules_for_crate("application")
                .it_may_depend_on(&["std::fmt", "domain"])
            .rules_for_module("domain::services")
                .it_may_depend_on(&["std::sync", "application"])
            .rules_for_module("domain::models")
                .it_must_not_depend_on_anything()
            .build();

        assert_eq!(rules.len(), 3);
    }

    #[allow(dead_code)]
    struct MustNotContainAttributeRule {
        subject: String,
        attribute: String,
    }

    struct MustNotContainAttribute {
        attribute: String,
    }

    impl MustNotContainAttribute {
        fn new(attribute: &str) -> Box<dyn SubjectInjectableRuleBuilder> {
            Box::new(MustNotContainAttribute {
                attribute: attribute.to_string(),
            })
        }
    }

    impl Display for MustNotContainAttributeRule {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Example Rule")
        }
    }

    impl Rule for MustNotContainAttributeRule {
        fn apply(&self, _file: &RustFile) -> Result<(), String> {
            Ok(())
        }

        fn is_applicable(&self, _file: &RustFile) -> bool {
            true
        }
    }

    impl SubjectInjectableRuleBuilder for MustNotContainAttribute {
        fn for_subject(&self, subject: &str) -> Box<dyn Rule> {
            Box::new(MustNotContainAttributeRule {
                subject: subject.to_string(),
                attribute: self.attribute.clone(),
            })
        }
    }

    #[test]
    fn test_subject_injection() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .rules_for_project()
                .it_must_not_have_circular_dependencies(3)
            .rules_for_crate("a_crate")
                .it(MustNotContainAttribute::new("#[test]"))
                .and_it(MustNotContainAttribute::new("#[rustfmt::skip]"))
                .and_it_may_depend_on(&["some::module"])
            .rules_for_module("my_crate::utils")
                .it_must_not_depend_on(&["some::module"])
                .and_it(MustNotContainAttribute::new("#[test]"))
            .rules_for_module("services::auth")
                .it_may_depend_on(&["some::module"])
                .and_it(MustNotContainAttribute::new("#[test]"))
            .rules_for_module("domain::entities")
                .it_must_not_depend_on_anything()
                .and_it(MustNotContainAttribute::new("#[test]"))
            .rules_for_module("models::product")
                .it(MustNotContainAttribute::new("#[test]"))
                .and_it_must_not_depend_on_anything()
            .rules_for_module("a_crate::another_module")
                .it_must_not_depend_on_anything()
            .build();

        assert_eq!(rules.len(), 13);
    }
}
