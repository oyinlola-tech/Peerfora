use std::collections::HashSet;

use uuid::Uuid;

use crate::{enums::admin_auth_error::AdminAuthError, errors::app_error::AppError};

pub fn validate_role_key(role_key: &str) -> Result<(), AppError> {
    let valid = (3..=64).contains(&role_key.len())
        && role_key.as_bytes()[0].is_ascii_lowercase()
        && role_key.bytes().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == b'_'
        });
    if valid {
        Ok(())
    } else {
        Err(AdminAuthError::InvalidRoleKey.into())
    }
}

pub fn validate_role_text(display_name: &str, description: &str) -> Result<(), AppError> {
    if !(3..=80).contains(&display_name.trim().chars().count())
        || description.trim().is_empty()
        || description.chars().count() > 500
    {
        return Err(AdminAuthError::InvalidRoleDetails.into());
    }
    Ok(())
}

pub fn validate_permission_keys(permission_keys: &[String]) -> Result<(), AppError> {
    if permission_keys.len() > 100
        || permission_keys
            .iter()
            .any(|permission_key| permission_key.is_empty() || permission_key.len() > 128)
        || HashSet::<&str>::from_iter(permission_keys.iter().map(String::as_str)).len()
            != permission_keys.len()
    {
        return Err(AdminAuthError::InvalidPermissionSelection.into());
    }
    Ok(())
}

pub fn validate_role_ids(role_ids: &[Uuid]) -> Result<(), AppError> {
    if role_ids.len() > 50
        || HashSet::<Uuid>::from_iter(role_ids.iter().copied()).len() != role_ids.len()
    {
        return Err(AdminAuthError::InvalidRoleAssignment.into());
    }
    Ok(())
}
