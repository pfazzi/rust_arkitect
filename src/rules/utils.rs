pub trait IsChild {
    fn is_child_of(&self, module: &str) -> bool;
}

impl IsChild for str {
    fn is_child_of(&self, module: &str) -> bool {
        self.starts_with(module)
    }
}

impl IsChild for String {
    fn is_child_of(&self, module: &str) -> bool {
        self.starts_with(module)
    }
}
