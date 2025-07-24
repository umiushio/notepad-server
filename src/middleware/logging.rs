use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use std::{future::{ready, Ready}, pin::Pin, time::Instant};
use tracing::info_span;

use crate::log_request;

pub struct EnhancedLogging;

impl<S, B> Transform<S, ServiceRequest> for EnhancedLogging
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = EnhancedLoggingMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(EnhancedLoggingMiddleware { service }))
    }
}

pub struct EnhancedLoggingMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for EnhancedLoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;
    
    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let path = req.path().to_string();
        let method = req.method().to_string();
        let ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();

        // 为每个请求创建span
        let span = info_span!(
            "request",
            method = %method,
            path = %path,
            ip = %ip,
            user_agent = ?req.headers().get("user-agent")
        );

        // 进入span
        let _enter = span.enter();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let status = res.status().as_u16();
            let duration = start.elapsed();

            // 记录请求完成
            log_request!(method, path, status, duration, ip);

            Ok(res)
        })
    }
}