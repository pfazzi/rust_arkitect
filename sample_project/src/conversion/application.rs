use crate::conversion::domain::{domain_function_1, domain_function_2};

pub fn application_function() {
    domain_function_1();
    domain_function_2();
}

mod use_cases {
    use crate::conversion::domain::domain_function_2;
    use crate::conversion::infrastructure::infrastructure_function;

    #[allow(dead_code)]
    fn application_use_case() {
        domain_function_2();
        infrastructure_function(); // TODO: parsing submodules
    }
}
