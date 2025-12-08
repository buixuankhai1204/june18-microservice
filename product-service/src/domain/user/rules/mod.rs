pub(crate) mod user_must_not_an_employee_before_become_an_employee;
pub mod email_must_be_unique;
pub mod username_must_be_unique;
pub mod user_must_have_at_least_one_address;

pub use email_must_be_unique::EmailMustBeUnique;
pub use username_must_be_unique::UsernameMustBeUnique;
pub use user_must_have_at_least_one_address::UserMustHaveAtLeastOneAddress;
