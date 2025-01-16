use crate::engine::Engine;
use crate::rules::may_depend_on::MayDependOnRule;
use crate::rules::must_not_depend_on_anything::MustNotDependOnAnythingRule;
use crate::rules::rule::Rule;
use std::collections::HashMap;
use std::env;
use std::marker::PhantomData;
use std::path::Path;

pub struct Project {
    pub project_root: String,
}

impl Project {
    pub fn from_path(absolute_path: &str) -> Project {
        Project {
            project_root: absolute_path.to_string(),
        }
    }

    pub fn new() -> Project {
        let cargo_manifest_dir =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

        Project {
            project_root: cargo_manifest_dir,
        }
    }

    /// Creates a Project from a path relative to the given file.
    pub fn from_relative_path(current_file: &str, relative_path: &str) -> Project {
        let current_dir = Path::new(current_file)
            .parent()
            .expect("Failed to get parent directory");

        let derived_path = current_dir.join(relative_path);

        let absolute_path = derived_path.canonicalize().unwrap_or_else(|e| {
            panic!(
                "Failed to resolve absolute path:\n\
                 - Current file: '{}'\n\
                 - Relative path: '{}'\n\
                 - Derived path (before resolving): '{}'\n\
                 Cause: {}",
                current_file,
                relative_path,
                derived_path.display(),
                e
            )
        });

        Project {
            project_root: absolute_path
                .to_str()
                .expect("Failed to convert path to string")
                .to_string(),
        }
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

pub struct ArchitecturalRules<State> {
    state: PhantomData<State>,
    component: TemporaryComponent,
    component_map: HashMap<String, TemporaryComponent>,
}

#[derive(Debug, PartialEq)]
enum RuleType {
    MayDependOn,
    MustNotDependentOnAnything,
}

struct TemporaryComponent {
    name: Option<String>,
    located_at: Option<String>,
    allowed_external_dependencies: Vec<String>,
    allowed_dependencies: Vec<String>,
    rule_type: Option<RuleType>,
}

pub struct Begin;
pub struct ComponentStarted;
pub struct LocationDefined;
pub struct ExternalDependenciesDefined;
pub struct ComponentDefined;

impl ArchitecturalRules<Begin> {
    pub fn define() -> Self {
        ArchitecturalRules {
            state: PhantomData,
            component: TemporaryComponent {
                name: None,
                located_at: None,
                allowed_external_dependencies: Vec::new(),
                allowed_dependencies: Vec::new(),
                rule_type: None,
            },
            component_map: Default::default(),
        }
    }

    pub fn component(self, name: &str) -> ArchitecturalRules<ComponentStarted> {
        ArchitecturalRules {
            state: PhantomData,
            component: TemporaryComponent {
                name: Some(name.to_string()),
                ..self.component
            },
            component_map: self.component_map,
        }
    }
}

impl ArchitecturalRules<ComponentStarted> {
    pub fn located_at(self, module: &str) -> ArchitecturalRules<LocationDefined> {
        ArchitecturalRules {
            state: PhantomData,
            component: TemporaryComponent {
                located_at: Some(module.to_string()),
                ..self.component
            },
            component_map: self.component_map,
        }
    }
}

impl ArchitecturalRules<LocationDefined> {
    pub fn allow_external_dependencies(
        self,
        dependencies: &[&str],
    ) -> ArchitecturalRules<ExternalDependenciesDefined> {
        ArchitecturalRules {
            state: PhantomData,
            component: TemporaryComponent {
                allowed_external_dependencies: dependencies
                    .iter()
                    .map(|&s| s.to_string())
                    .collect(),
                ..self.component
            },
            component_map: self.component_map,
        }
    }

    pub fn may_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<ComponentDefined> {
        ArchitecturalRules {
            state: PhantomData,
            component: TemporaryComponent {
                allowed_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
                rule_type: Some(RuleType::MayDependOn),
                ..self.component
            },
            component_map: self.component_map,
        }
    }

    pub fn must_not_depend_on_anything(self) -> ArchitecturalRules<ComponentDefined> {
        ArchitecturalRules {
            state: PhantomData,
            component: TemporaryComponent {
                rule_type: Some(RuleType::MustNotDependentOnAnything),
                ..self.component
            },
            component_map: self.component_map,
        }
    }
}

impl ArchitecturalRules<ExternalDependenciesDefined> {
    pub fn may_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<ComponentDefined> {
        ArchitecturalRules {
            state: PhantomData,
            component: TemporaryComponent {
                allowed_dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
                rule_type: Some(RuleType::MayDependOn),
                ..self.component
            },
            component_map: self.component_map,
        }
    }

    pub fn must_not_depend_on_anything(self) -> ArchitecturalRules<ComponentDefined> {
        ArchitecturalRules {
            state: PhantomData,
            component: TemporaryComponent {
                rule_type: Some(RuleType::MustNotDependentOnAnything),
                ..self.component
            },
            component_map: self.component_map,
        }
    }
}

impl ArchitecturalRules<ComponentDefined> {
    pub fn component(self, name: &str) -> ArchitecturalRules<ComponentStarted> {
        let component = self.component;
        let component_name = component.name.clone().unwrap();

        let mut component_map = self.component_map;
        component_map.insert(component_name, component);

        ArchitecturalRules {
            state: PhantomData,
            component: TemporaryComponent {
                name: Some(name.to_string()),
                located_at: None,
                allowed_dependencies: Vec::new(),
                allowed_external_dependencies: Vec::new(),
                rule_type: None,
            },
            component_map,
        }
    }

    pub fn finalize(self) -> Vec<Box<dyn Rule>> {
        let component = self.component;
        let component_name = component.name.clone().unwrap();

        let mut component_map = self.component_map;
        component_map.insert(component_name, component);

        let alias_map: HashMap<String, String> = component_map
            .iter()
            .map(|(alias, component)| (alias.clone(), component.located_at.clone().unwrap()))
            .collect();

        component_map
            .into_iter()
            .map(|(alias, component)| -> Box<dyn Rule> {
                match component.rule_type {
                    Some(RuleType::MayDependOn) => Box::new(MayDependOnRule {
                        subject: alias_map.get(&alias).unwrap().clone(),
                        allowed_dependencies: component
                            .allowed_dependencies
                            .into_iter()
                            .map(|s| alias_map.get(&s).cloned().unwrap_or(s))
                            .collect(),
                        allowed_external_dependencies: component.allowed_external_dependencies,
                    }),
                    Some(RuleType::MustNotDependentOnAnything) => {
                        Box::new(MustNotDependOnAnythingRule {
                            subject: alias_map.get(&alias).unwrap().clone(),
                            allowed_external_dependencies: component.allowed_external_dependencies,
                        })
                    }
                    None => panic!("This should never happen"),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_items() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .component("TestComponent1")
                .located_at("crate::test_component_1")
                .allow_external_dependencies(&["ext1", "ext2"])
                .may_depend_on(&["dep1", "dep2"])
            .component("TestComponent2")
                .located_at("crate::test_component_2")
                .allow_external_dependencies(&["ext1", "ext2"])
                .must_not_depend_on_anything()
            .finalize();

        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_one_item() {
        #[rustfmt::skip]
        let rules = ArchitecturalRules::define()
            .component("TestComponent1")
                .located_at("crate::test_component_1")
                .allow_external_dependencies(&["ext1", "ext2"])
                .may_depend_on(&["dep1", "dep2"])
            .finalize();

        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_may_depend_on() {
        let rules = ArchitecturalRules::<Begin>::define()
            .component("Component1")
            .located_at("crate::test_component_1")
            .may_depend_on(&["dependency1", "dependency2"]);

        assert_eq!(
            rules.component.allowed_dependencies,
            vec!["dependency1".to_string(), "dependency2".to_string()]
        );

        assert_eq!(rules.component.rule_type, Some(RuleType::MayDependOn));
    }

    #[test]
    fn test_must_not_depend_on_anything() {
        let rules = ArchitecturalRules::<Begin>::define()
            .component("Component1")
            .located_at("crate::test_component_1")
            .must_not_depend_on_anything();

        assert!(rules.component.allowed_dependencies.is_empty());

        assert_eq!(
            rules.component.rule_type,
            Some(RuleType::MustNotDependentOnAnything)
        );
    }
}
