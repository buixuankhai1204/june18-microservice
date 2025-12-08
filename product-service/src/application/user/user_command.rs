use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct AdminCreateUserCommand {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 25, message = "Mật khẩu phải từ 8 đến 25 ký tự"))]
    pub password: String,
    #[validate(length(min = 2, max = 30, message = "Họ và tên phải từ 2 đến 30 ký tự"))]
    pub fullname: String,
    #[validate(length(min = 3, max = 50, message = "Tên đăng nhập phải từ 3 đến 50 ký tự"))]
    pub username: String,
    pub birthday: Option<NaiveDate>,
    #[validate(url)]
    pub picture: Option<String>,
    pub gender: Option<String>,
    pub phone_number: Option<String>,
    pub address: Option<String>,
    pub language: Option<String>,
    #[validate(range(min = 0, max = 1))]
    pub status: Option<i16>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateUserCommand {
    #[validate(length(min = 2, max = 30))]
    pub fullname: Option<String>,
    pub username: Option<String>,
    #[validate(url)]
    pub picture: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub phone_number: Option<String>,
    pub address: Option<String>,
    pub language: Option<String>,
    pub birthday: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema, Default)]
pub struct UpdateProfileCommand {
    #[validate(length(min = 2, max = 30))]
    pub fullname: Option<String>,
    #[validate(url)]
    pub picture: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub address: Option<String>,
    pub language: Option<String>,
}
