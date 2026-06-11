use actix_web::{
    body::MessageBody,
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    Error, HttpResponse, ResponseError,
};
use futures_util::future::{ok, FutureExt, LocalBoxFuture, Ready};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::panic::AssertUnwindSafe;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

impl ApiError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<Value>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details,
        }
    }

    pub fn bad_request(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("BAD_REQUEST", message, details)
    }

    pub fn unauthorized(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("UNAUTHORIZED", message, details)
    }

    pub fn forbidden(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("FORBIDDEN", message, details)
    }

    pub fn not_found(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("NOT_FOUND", message, details)
    }

    pub fn conflict(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("CONFLICT", message, details)
    }

    pub fn invalid_token(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("INVALID_TOKEN", message, details)
    }

    pub fn internal_error(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("INTERNAL_SERVER_ERROR", message, details)
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self.code.as_str() {
            "BAD_REQUEST" => StatusCode::BAD_REQUEST,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "CONFLICT" => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(self)
    }
}

pub struct ErrorResponseMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ErrorResponseMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = ErrorResponseMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ErrorResponseMiddlewareService { service })
    }
}

pub struct ErrorResponseMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ErrorResponseMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request = req.request().clone();
        let fut = AssertUnwindSafe(self.service.call(req)).catch_unwind();

        Box::pin(async move {
            match fut.await {
                Ok(Ok(res)) => Ok(res.map_into_boxed_body()),
                Ok(Err(err)) => Err(err),
                Err(payload) => {
                    let details = Some(json!({ "panic": format!("{:?}", payload) }));
                    let error = ApiError::internal_error("Internal server error", details);
                    let response = HttpResponse::InternalServerError()
                        .json(error)
                        .map_into_boxed_body();
                    Ok(ServiceResponse::new(request, response))
                }
            }
        })
    }
}
