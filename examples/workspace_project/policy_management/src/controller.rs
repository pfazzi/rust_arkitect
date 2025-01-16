use crate::model::Policy;
use crate::repository::get;

pub fn controller() -> Policy {
    let policy = get();

    policy
}
