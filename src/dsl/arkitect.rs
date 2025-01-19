use crate::dsl::project::Project;
use crate::engine::Engine;
use crate::rule::Rule;

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

    pub fn complies_with(&mut self, rules: Vec<Box<dyn Rule>>) -> Result<Vec<String>, Vec<String>> {
        let violations =
            Engine::new(self.project.project_root.as_str(), rules.as_slice()).get_violations();

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
