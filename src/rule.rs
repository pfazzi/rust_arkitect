use crate::rust_file::RustFile;
use crate::rust_project::RustProject;
use std::fmt::Display;

pub trait Rule: Display {
    fn apply(&self, file: &RustFile) -> Result<(), String>;

    fn is_applicable(&self, file: &RustFile) -> bool;
}

pub trait ProjectRule: Display {
    fn apply(&self, file: &RustProject) -> Result<(), String>;
}
