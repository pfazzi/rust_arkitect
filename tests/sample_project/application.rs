use crate::sample_project::{domain, infrastructure};

#[allow(dead_code)]
pub fn application_function() {
    domain::domain_function();
    infrastructure::infrastructure_function();
}