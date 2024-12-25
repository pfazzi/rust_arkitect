mod conversion;
mod policy_management;

use crate::conversion::application::application_function;
use crate::policy_management::controller::controller;

#[allow(dead_code)]
fn main() {
    application_function();
    controller();
}