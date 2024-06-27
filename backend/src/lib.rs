use std::{fmt, future::Future};

pub mod auth;
pub mod chess;

#[derive(Debug)]
struct ServiceError;

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Service error")
    }
}

pub trait Service {
    const SERVICE_NAME: &'static str;

    fn run(self) -> impl Future<Output = Result<(), ServiceError>> + Send + Sync;
}
