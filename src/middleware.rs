//! Custom middleware components used across the application.

use actix_web::{
    Error,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorInternalServerError, ErrorUnauthorized},
    web,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use pushkind_common::models::auth::AuthenticatedUser;
use std::rc::Rc;

use crate::repository::DieselRepository;
use crate::repository::UserReader;

/// Middleware ensuring that the authenticated user referenced in the request
/// actually exists in the database.
pub struct RequireUserExists;

impl<S, B> Transform<S, ServiceRequest> for RequireUserExists
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireUserExistsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequireUserExistsMiddleware {
            service: Rc::new(service),
        })
    }
}

/// Service wrapper produced by [`RequireUserExists`].
pub struct RequireUserExistsMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireUserExistsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);
        let user = req.extract::<AuthenticatedUser>();
        let repo = req.app_data::<web::Data<DieselRepository>>().cloned();

        Box::pin(async move {
            let claims = match user.await {
                Ok(claims) => claims,
                Err(_) => return srv.call(req).await,
            };

            let uid: i32 = claims
                .sub
                .parse()
                .map_err(|_| ErrorUnauthorized("Invalid user"))?;

            let repo = repo.ok_or_else(|| ErrorInternalServerError("DB repo not found"))?;

            match repo.get_user_by_id(uid) {
                Ok(Some(_)) => srv.call(req).await,
                _ => Err(ErrorUnauthorized("User not found")),
            }
        })
    }
}
