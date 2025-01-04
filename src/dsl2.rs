use crate::rules::Rule;
use std::marker::PhantomData;

struct ArchitecturalRules<State> {
    state: PhantomData<State>,
}

struct Begin;
struct ComponentStarted;
struct LocationDefined;
struct ExternalDependenciesDefined;
struct ComponentDefined;

impl ArchitecturalRules<Begin> {
    pub fn new() -> Self {
        ArchitecturalRules { state: PhantomData }
    }

    pub fn component(self, component: &str) -> ArchitecturalRules<ComponentStarted> {
        ArchitecturalRules { state: PhantomData }
    }
}

impl ArchitecturalRules<ComponentStarted> {
    pub fn located_at(self, module: &str) -> ArchitecturalRules<LocationDefined> {
        ArchitecturalRules { state: PhantomData }
    }
}

impl ArchitecturalRules<LocationDefined> {
    pub fn allow_external_dependencies(
        self,
        external_dependencies: &[&str],
    ) -> ArchitecturalRules<ExternalDependenciesDefined> {
        ArchitecturalRules { state: PhantomData }
    }
}

impl ArchitecturalRules<ExternalDependenciesDefined> {
    pub fn may_depend_on(self, dependencies: &[&str]) -> ArchitecturalRules<ComponentDefined> {
        ArchitecturalRules { state: PhantomData }
    }

    pub fn must_not_depend_on_anything(self) -> ArchitecturalRules<ComponentDefined> {
        ArchitecturalRules { state: PhantomData }
    }
}

impl ArchitecturalRules<ComponentDefined> {
    pub fn component(self, component: &str) -> ArchitecturalRules<ComponentStarted> {
        ArchitecturalRules { state: PhantomData }
    }

    pub fn finalize(self) -> Vec<Box<dyn Rule>> {
        vec![]
    }
}

mod test {
    use crate::dsl2::*;

    #[test]
    fn test_dsl() {
        let rules = ArchitecturalRules::new()
            .component("TestComponent1")
            .located_at("crate::test_component_1")
            .allow_external_dependencies(&["ext1", "ext2"])
            .may_depend_on(&["dep1", "dep2"])
            .component("TestComponent2")
            .located_at("crate::test_component_2")
            .allow_external_dependencies(&["ext1", "ext2"])
            .must_not_depend_on_anything()
            .finalize();
    }
}
