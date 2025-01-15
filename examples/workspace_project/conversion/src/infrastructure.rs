use super::application::application_function;

pub fn infrastructure_function() {
    application_function();
    println!("Infrastructure function");
}
