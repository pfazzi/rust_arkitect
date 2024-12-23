use crate::validation::Rules;

pub struct Architecture<C> {
    _marker: std::marker::PhantomData<C>,
}

impl<C> Architecture<C> where C: std::hash::Hash{
    pub fn with_components() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    pub fn component(self, _component: C) -> Self {
        self
    }

    pub fn defined_by(self,  _module: &str) -> Self {
        self
    }

    pub fn rules_for(self, _component: C) -> Self {
        self
    }

    pub fn must_not_depend_on_anything(self)-> Self {
        self
    }

    pub fn depends_on(self, _dependencies: &[C]) -> Self {
        self
    }

    pub fn build(self) -> Rules {
        Rules {}
    }
}