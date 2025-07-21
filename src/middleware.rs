use actix_web::{
    Error,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorInternalServerError, ErrorUnauthorized},
    web,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use pushkind_common::db::DbPool;
use pushkind_common::models::auth::AuthenticatedUser;
use std::rc::Rc;

use crate::repository::UserReader;
use crate::repository::user::DieselUserRepository;

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
        let pool = req.app_data::<web::Data<DbPool>>().cloned();

        Box::pin(async move {
            let claims = match user.await {
                Ok(claims) => claims,
                Err(_) => return srv.call(req).await,
            };

            let uid: i32 = claims
                .sub
                .parse()
                .map_err(|_| ErrorUnauthorized("Invalid user"))?;

            let pool = pool.ok_or_else(|| ErrorInternalServerError("DB pool not found"))?;
            let repo = DieselUserRepository::new(&pool);

            match repo.get_by_id(uid) {
                Ok(Some(_)) => srv.call(req).await,
                _ => Err(ErrorUnauthorized("User not found")),
            }
        })
    }
}
