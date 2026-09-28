use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::de::DeserializeOwned;

use crate::domain::error::AppError;

pub struct ApiQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(q)) => Ok(ApiQuery(q)),
            Err(rejection) => Err(AppError::InvalidQuery {
                detail: rejection.body_text(),
            }),
        }
    }
}
