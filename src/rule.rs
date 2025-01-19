use crate::rust_file::RustFile;
use std::fmt::Display;

pub trait Rule: Display {
    fn apply(&self, file: &RustFile) -> Result<(), String>;

    fn is_applicable(&self, file: &RustFile) -> bool;
}
