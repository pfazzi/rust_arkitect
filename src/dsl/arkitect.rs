use crate::dsl::project::Project;
use crate::engine::Engine;
use crate::rule::{ProjectRule, Rule};

pub struct Rules {
    pub module_rules: Vec<Box<dyn Rule>>,
    pub project_rules: Vec<Box<dyn ProjectRule>>,
}

impl Rules {
    pub fn from(
        module_rules: Vec<Box<dyn Rule>>,
        project_rules: Vec<Box<dyn ProjectRule>>,
    ) -> Self {
        Rules {
            module_rules,
            project_rules,
        }
    }

    pub fn from_module_rules(module_rules: Vec<Box<dyn Rule>>) -> Self {
        let project_rules: Vec<Box<dyn ProjectRule>> = vec![];

        Rules {
            module_rules,
            project_rules,
        }
    }

    pub fn len(&self) -> usize {
        self.module_rules.len() + self.project_rules.len()
    }
}

pub struct Arkitect {
    project: Project,
    baseline: usize,
}

impl Arkitect {
    pub fn init_logger() {
        let _ = env_logger::builder().is_test(false).try_init();
    }

    pub fn with_baseline(self, baseline: usize) -> Self {
        Self { baseline, ..self }
    }

    pub fn complies_with(&mut self, rules: Rules) -> Result<Vec<String>, Vec<String>> {
        let violations = Engine::new(
            self.project.project_root.as_str(),
            &rules.module_rules,
            &rules.project_rules,
        )
        .compute_violations();

        if violations.len() <= self.baseline {
            Ok(violations)
        } else {
            Err(violations)
        }
    }
}

impl Arkitect {
    pub fn ensure_that(project: Project) -> Arkitect {
        Arkitect {
            project,
            baseline: 0,
        }
    }
}
