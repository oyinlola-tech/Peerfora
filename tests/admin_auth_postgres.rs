#[path = "admin_auth_postgres/suite.rs"]
mod suite;

#[tokio::test]
async fn administrator_login_mfa_session_and_audit_use_postgresql() {
    suite::run_administrator_postgresql_regression_suite().await;
}
