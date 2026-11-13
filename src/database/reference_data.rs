use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::errors::app_error::AppError;

const COUNTRY_SNAPSHOT: &str =
    include_str!("../../data/reference/countries-iso-codes-4.20.1-1.json");
const EXPECTED_COUNTRY_COUNT: usize = 249;
const EXPECTED_COUNTRY_SOURCE_VERSION: &str = "4.20.1-1";

#[derive(Deserialize)]
struct CountrySnapshot {
    source: String,
    source_version: String,
    records: Vec<CountryRecord>,
}

#[derive(Deserialize, Serialize)]
struct CountryRecord {
    alpha_2: String,
    alpha_3: String,
    display_name: String,
    official_name: Option<String>,
}

pub async fn install_country_snapshot(pool: &PgPool) -> Result<(), AppError> {
    let snapshot: CountrySnapshot =
        serde_json::from_str(COUNTRY_SNAPSHOT).map_err(|_| AppError::DatabaseMigrationFailed)?;
    validate_country_snapshot(&snapshot)?;
    let records =
        serde_json::to_value(&snapshot.records).map_err(|_| AppError::DatabaseMigrationFailed)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| AppError::DatabaseMigrationFailed)?;
    sqlx::query(
        "UPDATE countries SET is_active = FALSE WHERE source = $1 AND source_version <> $2",
    )
    .bind(&snapshot.source)
    .bind(&snapshot.source_version)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::DatabaseMigrationFailed)?;
    sqlx::query(
        "INSERT INTO countries (
            country_code, alpha_3, display_name, official_name,
            source, source_version, is_active
        )
        SELECT imported.alpha_2, imported.alpha_3, imported.display_name,
            imported.official_name, $2, $3, TRUE
        FROM jsonb_to_recordset($1) AS imported(
            alpha_2 TEXT, alpha_3 TEXT, display_name TEXT, official_name TEXT
        )
        ON CONFLICT (country_code) DO UPDATE SET
            alpha_3 = EXCLUDED.alpha_3,
            display_name = EXCLUDED.display_name,
            official_name = EXCLUDED.official_name,
            source = EXCLUDED.source,
            source_version = EXCLUDED.source_version,
            is_active = TRUE",
    )
    .bind(records)
    .bind(&snapshot.source)
    .bind(&snapshot.source_version)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::DatabaseMigrationFailed)?;
    sqlx::query(
        "INSERT INTO country_name_aliases (
            country_code, display_name, language_tag, source, source_version
        )
        SELECT country_code, display_name, 'und', source, source_version FROM countries
        WHERE source = $1 AND source_version = $2
        UNION
        SELECT country_code, official_name, 'und', source, source_version FROM countries
        WHERE source = $1 AND source_version = $2 AND official_name IS NOT NULL
        ON CONFLICT (country_code, display_name, language_tag) DO UPDATE SET
            source = EXCLUDED.source, source_version = EXCLUDED.source_version",
    )
    .bind(&snapshot.source)
    .bind(&snapshot.source_version)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::DatabaseMigrationFailed)?;
    sqlx::query(
        "INSERT INTO account_preferences (account_id, country_code)
        SELECT id, eligibility_country_code FROM accounts
        ON CONFLICT (account_id) DO NOTHING",
    )
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::DatabaseMigrationFailed)?;
    sqlx::query("ALTER TABLE accounts VALIDATE CONSTRAINT accounts_country_code_fk")
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::DatabaseMigrationFailed)?;
    sqlx::query(
        "ALTER TABLE account_preferences VALIDATE CONSTRAINT account_preferences_country_code_fk",
    )
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::DatabaseMigrationFailed)?;
    transaction
        .commit()
        .await
        .map_err(|_| AppError::DatabaseMigrationFailed)
}

fn validate_country_snapshot(snapshot: &CountrySnapshot) -> Result<(), AppError> {
    let mut alpha_2_codes = std::collections::HashSet::new();
    let mut alpha_3_codes = std::collections::HashSet::new();
    let valid = snapshot.source == "iso-codes"
        && snapshot.source_version == EXPECTED_COUNTRY_SOURCE_VERSION
        && snapshot.records.len() == EXPECTED_COUNTRY_COUNT
        && snapshot.records.iter().all(|record| {
            record.alpha_2.len() == 2
                && record.alpha_2.bytes().all(|byte| byte.is_ascii_uppercase())
                && record.alpha_3.len() == 3
                && record.alpha_3.bytes().all(|byte| byte.is_ascii_uppercase())
                && !record.display_name.trim().is_empty()
                && alpha_2_codes.insert(record.alpha_2.as_str())
                && alpha_3_codes.insert(record.alpha_3.as_str())
        });
    if valid {
        Ok(())
    } else {
        Err(AppError::DatabaseMigrationFailed)
    }
}
