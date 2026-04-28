/// Unified tracing middleware that injects an `x-request-id` into every log
/// entry for a request, enabling end-to-end correlation across log lines.
///
/// Usage (actix-web):
/// ```rust
/// use petchain_2fa::tracing_middleware::RequestIdMiddleware;
/// App::new().wrap(RequestIdMiddleware)
/// ```
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::task::{Context, Poll};
use tracing::Instrument;
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Actix-web middleware factory.
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestIdMiddlewareService { service })
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Reuse incoming request-id or generate a new one.
        let request_id = req
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Store on request extensions so handlers can read it.
        req.extensions_mut().insert(RequestId(request_id.clone()));

        let span = tracing::info_span!(
            "http_request",
            request_id = %request_id,
            method = %req.method(),
            path = %req.path(),
        );

        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.instrument(span).await?;
            // Echo the request-id back in the response headers.
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static(REQUEST_ID_HEADER),
                actix_web::http::header::HeaderValue::from_str(&request_id)
                    .unwrap_or_else(|_| actix_web::http::header::HeaderValue::from_static("")),
            );
            Ok(res)
        })
    }
}

/// Newtype wrapper stored in request extensions.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Initialise a `tracing-subscriber` that emits structured JSON logs.
/// Call once at application startup before building the Actix app.
pub fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let _ = fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}
