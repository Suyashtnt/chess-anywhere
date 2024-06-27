use error_stack::{Result, ResultExt};
use std::{error::Error, fmt, future::Future};

pub mod auth;
pub mod chess;

#[derive(Debug)]
pub struct ServiceError;

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Service error")
    }
}

impl Error for ServiceError {}

pub trait Service {
    /// The name of the service used in logs
    const SERVICE_NAME: &'static str;

    /// Run the service
    ///
    /// # Returns
    /// Returns OK(()) If it runs successfully and it was intended to end
    /// Else returns an error if it fails (and consequently takes down the whole app)
    fn run(self) -> impl Future<Output = Result<(), ServiceError>> + Send + Sync;
}

#[derive(Debug, Clone)]
pub struct BackendService {
    pg_pool: sqlx::postgres::PgPool,
}

impl Service for BackendService {
    const SERVICE_NAME: &'static str = "Backend Scripts";

    /// Initializes the database and fills up various caches
    async fn run(self) -> Result<(), ServiceError> {
        // get current tokio runtime
        let runtime = tokio::runtime::Handle::current();

        runtime.block_on(async {
            sqlx::migrate!()
                .run(&self.pg_pool)
                .await
                .change_context(ServiceError)
        })?;

        Ok(())
    }
}
