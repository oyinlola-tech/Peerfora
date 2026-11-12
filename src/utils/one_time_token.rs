use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn create_one_time_token() -> String {
    Uuid::new_v4().simple().to_string()
}

pub fn hash_one_time_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
