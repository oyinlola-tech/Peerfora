use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::developer::{
    DeveloperApplicationReviewDecision, DeveloperApplicationScope, DeveloperApplicationScopeStatus,
    DeveloperApplicationStatus, DeveloperApplicationType, DeveloperProfileStatus,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActivateDeveloperProfileRequest {
    pub display_name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegisterDeveloperApplicationRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub application_type: DeveloperApplicationType,
    pub requested_scopes: Vec<DeveloperApplicationScope>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeDeveloperApplicationQuery {
    pub status: Option<DeveloperApplicationStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewDeveloperApplicationRequest {
    pub decision: DeveloperApplicationReviewDecision,
    pub reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperProfileResponse {
    pub account_id: Uuid,
    pub display_name: String,
    pub description: String,
    pub profile_status: DeveloperProfileStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperApplicationResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub application_type: DeveloperApplicationType,
    pub application_status: DeveloperApplicationStatus,
    pub scopes: Vec<DeveloperApplicationScopeItem>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperApplicationScopeItem {
    pub scope: DeveloperApplicationScope,
    pub status: DeveloperApplicationScopeStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperApplicationsResponse {
    pub items: Vec<DeveloperApplicationResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeDeveloperApplicationItem {
    pub id: Uuid,
    pub owner_account_id: Uuid,
    pub owner_username: String,
    pub name: String,
    pub description: String,
    pub application_type: DeveloperApplicationType,
    pub application_status: DeveloperApplicationStatus,
    pub scopes: Vec<DeveloperApplicationScopeItem>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeDeveloperApplicationsResponse {
    pub items: Vec<AdministrativeDeveloperApplicationItem>,
    pub total_count: i64,
    pub limit: u32,
    pub offset: u32,
}

impl TryFrom<crate::models::developer::DeveloperApplicationRecord>
    for DeveloperApplicationResponse
{
    type Error = crate::errors::app_error::AppError;

    fn try_from(
        record: crate::models::developer::DeveloperApplicationRecord,
    ) -> Result<Self, Self::Error> {
        let application_type =
            DeveloperApplicationType::from_database_value(&record.application_type)
                .ok_or(crate::errors::app_error::AppError::Internal)?;
        let application_status =
            DeveloperApplicationStatus::from_database_value(&record.application_status)
                .ok_or(crate::errors::app_error::AppError::Internal)?;
        let scopes = record
            .scopes
            .0
            .into_iter()
            .map(|scope| {
                Ok(DeveloperApplicationScopeItem {
                    scope: DeveloperApplicationScope::from_database_value(&scope.scope_key)
                        .ok_or(crate::errors::app_error::AppError::Internal)?,
                    status: DeveloperApplicationScopeStatus::from_database_value(
                        &scope.scope_status,
                    )
                    .ok_or(crate::errors::app_error::AppError::Internal)?,
                })
            })
            .collect::<Result<Vec<_>, crate::errors::app_error::AppError>>()?;
        Ok(Self {
            id: record.id,
            name: record.name,
            description: record.description,
            application_type,
            application_status,
            scopes,
            submitted_at: record.submitted_at,
            reviewed_at: record.reviewed_at,
            review_reason: record.review_reason,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}

impl TryFrom<crate::models::developer::AdministrativeDeveloperApplicationRecord>
    for AdministrativeDeveloperApplicationItem
{
    type Error = crate::errors::app_error::AppError;

    fn try_from(
        record: crate::models::developer::AdministrativeDeveloperApplicationRecord,
    ) -> Result<Self, Self::Error> {
        let application_type =
            DeveloperApplicationType::from_database_value(&record.application_type)
                .ok_or(crate::errors::app_error::AppError::Internal)?;
        let application_status =
            DeveloperApplicationStatus::from_database_value(&record.application_status)
                .ok_or(crate::errors::app_error::AppError::Internal)?;
        let scopes = record
            .scopes
            .0
            .into_iter()
            .map(|scope| {
                Ok(DeveloperApplicationScopeItem {
                    scope: DeveloperApplicationScope::from_database_value(&scope.scope_key)
                        .ok_or(crate::errors::app_error::AppError::Internal)?,
                    status: DeveloperApplicationScopeStatus::from_database_value(
                        &scope.scope_status,
                    )
                    .ok_or(crate::errors::app_error::AppError::Internal)?,
                })
            })
            .collect::<Result<Vec<_>, crate::errors::app_error::AppError>>()?;
        Ok(Self {
            id: record.id,
            owner_account_id: record.owner_account_id,
            owner_username: record.owner_username,
            name: record.name,
            description: record.description,
            application_type,
            application_status,
            scopes,
            submitted_at: record.submitted_at,
            created_at: record.created_at,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IssueDeveloperCredentialRequest {
    pub name: String,
    pub expires_in_days: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdministrativeDeveloperCredentialRevocationRequest {
    pub reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperCredentialItem {
    pub id: Uuid,
    pub application_id: Uuid,
    pub name: String,
    pub environment: crate::enums::developer::DeveloperCredentialEnvironment,
    pub scopes: Vec<DeveloperApplicationScope>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperCredentialListResponse {
    pub items: Vec<DeveloperCredentialItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuedDeveloperCredentialResponse {
    pub credential: DeveloperCredentialItem,
    pub secret: String,
}

impl TryFrom<crate::models::developer::DeveloperCredentialRecord> for DeveloperCredentialItem {
    type Error = crate::errors::app_error::AppError;

    fn try_from(
        record: crate::models::developer::DeveloperCredentialRecord,
    ) -> Result<Self, Self::Error> {
        let environment =
            crate::enums::developer::DeveloperCredentialEnvironment::from_database_value(
                &record.environment,
            )
            .ok_or(crate::errors::app_error::AppError::Internal)?;
        let scopes = record
            .scopes
            .iter()
            .map(|scope| {
                DeveloperApplicationScope::from_database_value(scope)
                    .ok_or(crate::errors::app_error::AppError::Internal)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id: record.id,
            application_id: record.application_id,
            name: record.name,
            environment,
            scopes,
            created_at: record.created_at,
            expires_at: record.expires_at,
            revoked_at: record.revoked_at,
            last_used_at: record.last_used_at,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RotateDeveloperCredentialRequest {
    pub expires_in_days: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeveloperApplicationEnforcementRequest {
    pub reason: String,
}
