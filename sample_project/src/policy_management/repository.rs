use crate::policy_management::model::Policy;

pub fn get() -> Policy {
    Policy {
        id: String::from("1234")
    }
}