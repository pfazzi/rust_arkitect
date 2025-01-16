use std::fmt::Display;

pub trait Rule: Display {
    fn apply(&self, file: &str) -> Result<(), String>;

    fn is_applicable(&self, file: &str) -> bool;
}
