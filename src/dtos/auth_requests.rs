use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAccountRequest {
    pub email: String,
    pub password: String,
    pub username: String,
    pub display_name: Option<String>,
    pub date_of_birth: String,
    pub country_code: String,
    pub accepted_terms: bool,
}

#[derive(Deserialize)]
pub struct LoginAccountRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RequestPasswordResetRequest {
    pub email: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPasswordResetRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}
#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}
