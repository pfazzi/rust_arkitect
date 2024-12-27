use crate::validation::Rules;

pub struct Architecture {
}

impl Architecture {
    pub fn define() -> Self {
        Self {}
    }

    pub fn component(self, _component: &str) -> Self {
        self
    }

    pub fn defined_by(self,  _module: &str) -> Self {
        self
    }

    pub fn rules_for(self, _component: &str) -> Self {
        self
    }

    pub fn must_not_depend_on_anything(self)-> Self {
        self
    }

    pub fn may_depend_on(self, _dependencies: &[&str]) -> Self {
        self
    }

    pub fn build(self) -> Rules {
        Rules {}
    }
}