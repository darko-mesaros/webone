use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

use tracing::error;

use crate::templates::Error5xxTemplate;

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // Returning a HTML page for an error
        let template = Error5xxTemplate {
            error: self.0.to_string(),
        };
        match template.render() {
            Ok(html) => {
                error!("Internal Application Error: {}", self.0.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
            }
            // This has failed catastrophically - just return some string
            Err(_) => {
                error!("Internal Server Error: {}", self.0.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
