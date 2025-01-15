use crate::policy_management::model::Policy;
use crate::policy_management::repository::get;

pub fn controller() -> Policy {
    let policy = get();

    policy
}
