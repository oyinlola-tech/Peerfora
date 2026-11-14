use std::{future::Future, time::Duration};

use axum::{extract::Request, middleware::Next, response::Response};

use crate::{errors::app_error::AppError, models::app_state::AppState};

pub async fn enforce_request_timeout(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let timeout = Duration::from_secs(state.request_timeout_seconds);
    run_request_with_timeout(next.run(request), timeout).await
}

async fn run_request_with_timeout<F>(
    request_future: F,
    timeout: Duration,
) -> Result<Response, AppError>
where
    F: Future<Output = Response>,
{
    tokio::time::timeout(timeout, request_future)
        .await
        .map_err(|_| AppError::RequestTimedOut)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::{body::Body, response::Response};

    use super::run_request_with_timeout;
    use crate::errors::app_error::AppError;

    #[tokio::test]
    async fn returns_response_when_request_finishes_before_timeout() {
        let result = run_request_with_timeout(
            async { Response::new(Body::empty()) },
            Duration::from_secs(1),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn returns_timeout_error_when_request_exceeds_timeout() {
        let result = run_request_with_timeout(
            async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Response::new(Body::empty())
            },
            Duration::from_millis(1),
        )
        .await;

        assert!(matches!(result, Err(AppError::RequestTimedOut)));
    }
}
