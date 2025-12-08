use crate::infrastructure::error::AppResult;

pub trait BusinessRuleInterface {
    fn check_broken(&self) -> AppResult<()>;
}
