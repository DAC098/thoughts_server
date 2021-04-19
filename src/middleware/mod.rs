
use actix_web::dev::{ServiceRequest, ServiceResponse, MessageBody};
use actix_web::scope::ScopeService;
use actix_web::error::Error;
use core::future::Future;

pub struct CheckLoggedIn {}

// impl<S, B> Transform<S> for CheckLoggedIn
// where
//     S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//     S::Future: 'static,
// {
//     type Request = ServiceRequest;
//     type Response = ServiceResponse<B>;
//     type Error = Error;
// }

pub struct CheckLoggedInMiddleware<S> {
    service: S
}

