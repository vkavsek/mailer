use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;
use strum_macros::AsRefStr;
// use tracing::debug;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From, AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    UuidNotInHeader,
    HeaderToStrFail,

    #[from]
    WebStructParsing(super::structs::WebStructParsingError),

    #[from]
    TokioJoin(#[serde_as(as = "DisplayFromStr")] tokio::task::JoinError),
    #[from]
    Io(#[serde_as(as = "DisplayFromStr")] std::io::Error),
    #[from]
    SqlxCore(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}

// Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn status_code_and_client_error(&self) -> (StatusCode, ClientError) {
        use sqlx::{error::DatabaseError, postgres::PgDatabaseError};
        use ClientError::*;

        // If we get an error for unique violation from our DB, we want to let the user know that they need to
        // input a different email / they are already subscribed.
        #[allow(clippy::borrowed_box)]
        let is_unique_violation_err = |er: &Box<dyn DatabaseError>| {
            if let Some(er) = er.try_downcast_ref::<PgDatabaseError>() {
                er.code() == "23505"
            } else {
                false
            }
        };

        match self {
            Error::WebStructParsing(wsp_er) => {
                (StatusCode::BAD_REQUEST, InvalidInput(wsp_er.to_string()))
            }
            Error::SqlxCore(sqlx::Error::Database(er)) if is_unique_violation_err(er) => (
                StatusCode::NOT_ACCEPTABLE,
                InvalidInput("The email you provided is already used.".to_string()),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ServiceError),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::debug!("{:<12} - into_response(Error: {self:?})", "INTO_RESP");

        // Construct a response
        let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into response so that it can be retrieved later.
        res.extensions_mut().insert(Arc::new(self));

        res
    }
}

#[derive(Debug, Serialize, AsRefStr)]
#[serde(tag = "message", content = "detail")]
pub enum ClientError {
    InvalidInput(String),
    ServiceError,
}
