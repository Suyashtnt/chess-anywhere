use std::ops::{Deref, DerefMut};

use axum::response::{IntoResponse, Response};
use error_stack::{Context, Report};
pub struct AxumReport<C: Context>(Report<C>);

impl<C: Context> IntoResponse for AxumReport<C> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

impl<C> From<Report<C>> for AxumReport<C>
where
    C: Context,
{
    fn from(report: Report<C>) -> Self {
        Self(report)
    }
}

impl<C> Deref for AxumReport<C>
where
    C: Context,
{
    type Target = Report<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for AxumReport<C>
where
    C: Context,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
