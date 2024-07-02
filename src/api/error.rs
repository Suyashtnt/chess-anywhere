use std::ops::{Deref, DerefMut};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use error_stack::{Context, Report};
pub struct AxumReport<C: Context>(StatusCode, Report<C>);

impl<C: Context> AxumReport<C> {
    pub fn new(status: StatusCode, report: Report<C>) -> Self {
        Self(status, report)
    }
}

impl<C: Context> IntoResponse for AxumReport<C> {
    fn into_response(self) -> Response {
        (self.0, axum::Json(self.1)).into_response()
    }
}

impl<C: Context> From<Report<C>> for AxumReport<C> {
    fn from(report: Report<C>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, report)
    }
}

impl<C: Context> Deref for AxumReport<C> {
    type Target = Report<C>;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<C: Context> DerefMut for AxumReport<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}
