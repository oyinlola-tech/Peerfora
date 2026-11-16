use chrono::{Duration, Utc};
use peerfora_api::{
    database::connect_database,
    repositories::{account_password_reset_repository, account_repository},
};
use sqlx::PgPool;
use std::time::Duration as StdDuration;
use tokio::time::timeout;
use uuid::Uuid;

async fn fixture(pool: &PgPool) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!("reset-security-{id}@example.test");
    let username = format!("r_{}", &id.simple().to_string()[..16]);
    sqlx::query(
        "INSERT INTO accounts (id, email, username, password_hash, email_verified_at,
         eligibility_country_code, age_policy_version, terms_accepted_at)
         VALUES ($1, $2, $3, 'old-hash', now(), 'NG', 'test', now())",
    )
    .bind(id)
    .bind(&email)
    .bind(username)
    .execute(pool)
    .await
    .unwrap();
    (id, email)
}

#[tokio::test]
async fn reset_issuance_serializes_with_suspension() {
    let Ok(url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let pool = connect_database(&url).await.unwrap();
    let (id, email) = fixture(&pool).await;
    let mut suspension = pool.begin().await.unwrap();
    sqlx::query("UPDATE accounts SET account_status = 'suspended' WHERE id = $1")
        .bind(id)
        .execute(&mut *suspension)
        .await
        .unwrap();
    let token = format!("issuance-{id}");
    let mut issuance = Box::pin(
        account_password_reset_repository::create_password_reset_token(&pool, &email, &token),
    );
    assert!(
        timeout(StdDuration::from_millis(100), &mut issuance)
            .await
            .is_err()
    );
    suspension.commit().await.unwrap();
    assert!(issuance.await.unwrap().is_none());
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM password_reset_tokens WHERE account_id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn reset_consumption_serializes_with_suspension() {
    let Ok(url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let pool = connect_database(&url).await.unwrap();
    let (id, email) = fixture(&pool).await;
    let token = format!("reset-{id}");
    account_password_reset_repository::create_password_reset_token(&pool, &email, &token)
        .await
        .unwrap()
        .unwrap();
    let mut suspension = pool.begin().await.unwrap();
    sqlx::query("UPDATE accounts SET account_status = 'suspended' WHERE id = $1")
        .bind(id)
        .execute(&mut *suspension)
        .await
        .unwrap();
    let mut reset = Box::pin(account_password_reset_repository::reset_account_password(
        &pool, &token, "new-hash",
    ));
    assert!(
        timeout(StdDuration::from_millis(100), &mut reset)
            .await
            .is_err()
    );
    suspension.commit().await.unwrap();
    assert!(reset.await.is_err());
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM accounts WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(hash, "old-hash");
}

#[tokio::test]
async fn reset_tokens_are_replaced_and_stale_password_cannot_start_a_session() {
    let Ok(url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let pool = connect_database(&url).await.unwrap();
    let (id, email) = fixture(&pool).await;
    account_repository::create_session(
        &pool,
        id,
        &format!("existing-{id}"),
        Utc::now() + Duration::hours(1),
        "old-hash",
    )
    .await
    .unwrap();
    let token_one = format!("replace-one-{id}");
    let token_two = format!("replace-two-{id}");
    let (first, second) = tokio::join!(
        account_password_reset_repository::create_password_reset_token(&pool, &email, &token_one),
        account_password_reset_repository::create_password_reset_token(&pool, &email, &token_two),
    );
    assert!(first.unwrap().is_some() && second.unwrap().is_some());
    let hashes: Vec<String> =
        sqlx::query_scalar("SELECT token_hash FROM password_reset_tokens WHERE account_id = $1")
            .bind(id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(hashes.len(), 1);
    assert!(hashes[0] == token_one || hashes[0] == token_two);
    let (first_reset, second_reset) = tokio::join!(
        account_password_reset_repository::reset_account_password(&pool, &hashes[0], "new-hash"),
        account_password_reset_repository::reset_account_password(&pool, &hashes[0], "new-hash"),
    );
    assert_ne!(first_reset.is_ok(), second_reset.is_ok());
    let sessions: i64 =
        sqlx::query_scalar("SELECT count(*) FROM account_sessions WHERE account_id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(sessions, 0);
    let stale = account_repository::create_session(
        &pool,
        id,
        &format!("stale-{id}"),
        Utc::now() + Duration::hours(1),
        "old-hash",
    )
    .await;
    assert!(stale.is_err());
    let valid = account_repository::create_session(
        &pool,
        id,
        &format!("valid-{id}"),
        Utc::now() + Duration::hours(1),
        "new-hash",
    )
    .await;
    assert!(valid.is_ok());
}

#[tokio::test]
async fn session_creation_waiting_on_password_change_rejects_old_hash() {
    let Ok(url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let pool = connect_database(&url).await.unwrap();
    let (id, _) = fixture(&pool).await;
    let mut reset = pool.begin().await.unwrap();
    sqlx::query("UPDATE accounts SET password_hash = 'new-hash' WHERE id = $1")
        .bind(id)
        .execute(&mut *reset)
        .await
        .unwrap();
    let session_hash = format!("waiting-session-{id}");
    let mut login = Box::pin(account_repository::create_session(
        &pool,
        id,
        &session_hash,
        Utc::now() + Duration::hours(1),
        "old-hash",
    ));
    assert!(
        timeout(StdDuration::from_millis(100), &mut login)
            .await
            .is_err()
    );
    reset.commit().await.unwrap();
    assert!(login.await.is_err());
}
