pub trait IsChild {
    fn is_child_of(&self, module: &str) -> bool;
}

impl IsChild for str {
    fn is_child_of(&self, module: &str) -> bool {
        if module.is_empty() {
            panic!("Module cannot be an empty string");
        }

        self == module || self.starts_with(&format!("{}::", module))
    }
}

impl IsChild for String {
    fn is_child_of(&self, module: &str) -> bool {
        self.as_str().is_child_of(module)
    }
}

#[cfg(test)]
mod tests {
    use super::IsChild;

    #[test]
    #[should_panic(expected = "Module cannot be an empty string")]
    fn test_str_is_child_of_with_empty_module() {
        "module::child".is_child_of(""); // Deve panicare
    }

    #[test]
    fn test_str_is_child_of() {
        assert!("module::child".is_child_of("module"));
        assert!("module::child::subchild".is_child_of("module"));

        assert!(!"other_module::child".is_child_of("module"));

        assert!("module".is_child_of("module"));

        assert!(!"modulesubstring".is_child_of("module"));
    }

    #[test]
    fn test_string_is_child_of() {
        assert!(String::from("module::child").is_child_of("module"));
        assert!(String::from("module::child::subchild").is_child_of("module"));

        assert!(!String::from("other_module::child").is_child_of("module"));

        assert!(String::from("module").is_child_of("module"));

        assert!(!String::from("modulesubstring").is_child_of("module"));
    }

    #[test]
    fn test_edge_cases() {
        assert!(!"mod".is_child_of("module::child"));
    }
}
