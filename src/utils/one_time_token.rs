use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

use crate::constants::one_time_token::ONE_TIME_TOKEN_RANDOM_BYTE_LENGTH;

pub fn create_one_time_token() -> String {
    let mut token = [0_u8; ONE_TIME_TOKEN_RANDOM_BYTE_LENGTH];
    OsRng.fill_bytes(&mut token);
    hex::encode(token)
}

pub fn hash_one_time_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
