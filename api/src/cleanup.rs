use crate::{AppError, AppState};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

/// Delete expired and revoked refresh tokens from the database
pub async fn cleanup_refresh_tokens(state: &AppState) -> Result<usize, AppError> {
    let result = state
        .db
        .query("DELETE FROM refresh_token WHERE revoked = true OR expires_at < time::now()")
        .await
        .map_err(|e| AppError::DbError(format!("Failed to cleanup refresh tokens: {}", e)))?;

    // Extract the number of deleted tokens
    let deleted: Vec<serde_json::Value> = result
        .check()
        .map_err(|e| AppError::DbError(format!("Failed to check cleanup result: {}", e)))?;

    // Count the number of deleted records
    let count = deleted.len();

    info!(
        "Cleanup job: deleted {} expired/revoked refresh tokens",
        count
    );
    Ok(count)
}

/// Initialize the cleanup job scheduler
pub async fn start_cleanup_scheduler(state: AppState) -> Result<JobScheduler, AppError> {
    let scheduler = JobScheduler::new()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create scheduler: {}", e)))?;

    // Schedule cleanup job to run daily at 3 AM UTC
    // Cron format: "sec min hour day_of_month month day_of_week year"
    let state_clone = state.clone();
    let job = Job::new_async("0 0 3 * * *", move |_uuid, _lock| {
        let state = state_clone.clone();
        Box::pin(async move {
            match cleanup_refresh_tokens(&state).await {
                Ok(count) => {
                    info!("Scheduled cleanup completed: {} tokens deleted", count);
                }
                Err(e) => {
                    error!("Scheduled cleanup failed: {}", e);
                }
            }
        })
    })
    .map_err(|e| AppError::InternalServerError(format!("Failed to create cleanup job: {}", e)))?;

    scheduler.add(job).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to add job to scheduler: {}", e))
    })?;

    scheduler
        .start()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to start scheduler: {}", e)))?;

    info!("Cleanup scheduler started: daily at 3 AM UTC");

    Ok(scheduler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use surrealdb::Surreal;
    use surrealdb::engine::any::Any;

    #[tokio::test]
    async fn test_cleanup_scheduler_creation() {
        // This test verifies the scheduler can be created
        // Note: We can't easily test the actual cleanup without a real database
        let result = JobScheduler::new().await;
        assert!(result.is_ok());
    }
}
