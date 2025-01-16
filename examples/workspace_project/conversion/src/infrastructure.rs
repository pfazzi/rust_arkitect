use super::application::application_function;
use contracts::external_services::service_call_one;

pub fn infrastructure_function() {
    application_function();
    println!("Infrastructure function");
    service_call_one();
}
