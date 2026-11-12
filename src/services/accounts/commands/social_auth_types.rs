use crate::models::account::Account;

pub struct SocialAuthorizationStart {
    pub authorization_url: String,
    pub state: String,
}

pub enum SocialAuthorizationResult {
    SignedIn {
        account: Account,
        session_token: String,
    },
    NeedsBirthday {
        onboarding_token: String,
    },
}
