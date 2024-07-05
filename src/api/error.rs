use std::ops::{Deref, DerefMut};

use aide::{
    gen::GenContext,
    openapi::{self, MediaType, Operation, SchemaObject},
    OperationOutput,
};
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use error_stack::{Context, Report};
use indexmap::IndexMap;
use schemars::JsonSchema;

#[derive(JsonSchema)]
#[serde(remote = "Report")]
#[allow(dead_code)]
pub struct ReportRef<C> {
    context: C,
    attachments: Vec<String>,
    sources: Vec<ReportRef<serde_json::Value>>,
}
pub struct AxumReport<C: Context>(StatusCode, Report<C>);

impl<C: Context + JsonSchema> OperationOutput for AxumReport<C> {
    type Inner = ReportRef<C>;

    fn operation_response(
        ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Option<openapi::Response> {
        let mut schema = ctx.schema.subschema_for::<ReportRef<C>>().into_object();

        Some(openapi::Response {
            description: schema.metadata().description.clone().unwrap_or_default(),
            content: IndexMap::from_iter([(
                "application/json".into(),
                MediaType {
                    schema: Some(SchemaObject {
                        json_schema: schema.into(),
                        example: None,
                        external_docs: None,
                    }),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        })
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, openapi::Response)> {
        if let Some(res) = Self::operation_response(ctx, operation) {
            let success_response = [(Some(200), res)];

            [
                &success_response,
                JsonRejection::inferred_responses(ctx, operation).as_slice(),
            ]
            .concat()
        } else {
            Vec::new()
        }
    }
}

impl<C: Context + JsonSchema> JsonSchema for AxumReport<C> {
    fn schema_name() -> String {
        format!("AxumReport_for_{}", C::schema_name())
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Owned(format!(
            std::concat!(std::module_path!(), "::", "AxumReport_for_{}"),
            C::schema_id()
        ))
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        ReportRef::<C>::json_schema(gen)
    }
}

impl<C: Context> From<AxumReport<C>> for Report<C> {
    fn from(report: AxumReport<C>) -> Self {
        report.1
    }
}

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
