use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngCore, rngs::OsRng};

use crate::constants::{agents::AGENT_CREDENTIAL_PREFIX, developers::DEVELOPER_CREDENTIAL_PREFIX};

pub fn create_agent_machine_credential() -> String {
    create_machine_credential(AGENT_CREDENTIAL_PREFIX)
}

pub fn create_developer_machine_credential() -> String {
    create_machine_credential(DEVELOPER_CREDENTIAL_PREFIX)
}

fn create_machine_credential(prefix: &str) -> String {
    let mut secret = [0_u8; 32];
    OsRng.fill_bytes(&mut secret);
    format!("{prefix}{}", URL_SAFE_NO_PAD.encode(secret))
}
