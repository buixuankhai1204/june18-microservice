use crate::core::error::AppResult;

pub trait BusinessRuleInterface {
    fn check_broken(&self) -> AppResult<()>;
}
