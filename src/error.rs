
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::{error, event, Level, warn};
use strum_macros::AsRefStr;
use std::fmt;

use sqlx::Error as SqlxError;
use crate::{crypt, web};

use swiftide::integrations::qdrant::VectorConfigBuilderError;
use swiftide::integrations::ollama::OllamaBuilderError;
use swiftide::integrations::redis::RedisBuilderError;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    LoginFail,

    CtxExt(web::mw_auth::CtxExtError),

    TicketDeleteFailIdNotFound { id: u64 },
    TicketGetFailIdNotFound { id: u64 },
    TicketUpdateFailIdNotFound { id: u64 },

    DocumentUploadFail,
    DocumentUploadFailOllama,

    ServiceError(String),
    FailToCreatePool(String),
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),

    SwiftideError(String),
    QdrantError(String),
    OllamaError(String),
    RedisError(String),

    EntityNotFound { entity: &'static str, id: i64 },

    CryptError(String),
    SqlxError(String),

    KeyFailHmac,

    PwdNotMatching,
    UserNotFound,
    UserHasNoPwd { user_id: i64 },

    FailParseTime(String),

    TokenInvalidFormat,
    TokenCannotDecodeIdentifier,
    TokenCannotDecodeExpiration,
    TokenSignatureNotMatching,
    TokenExpirationNotIso,
    TokenExpired,

    QueryError(String),

}

impl From<SqlxError> for Error {
    fn from(e: SqlxError) -> Self {
        Self::SqlxError(e.to_string())
    }
}

impl From<VectorConfigBuilderError> for Error {
    fn from(err: VectorConfigBuilderError) -> Self {
        Error::QdrantError(err.to_string())
    }
}

impl From<OllamaBuilderError> for Error {
    fn from(err: OllamaBuilderError) -> Self {
        Error::OllamaError(err.to_string())
    }
}

impl From<RedisBuilderError> for Error {
    fn from(err: RedisBuilderError) -> Self {
        Error::RedisError(err.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(
        &self,
        fmt: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(fmt, "{}", serde_json::to_string(self).unwrap_or_else(|_| "Error".to_string()))
    }
}

impl std::error::Error for Error {}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the error into response extensions for later use
        response.extensions_mut().insert(self);

        response
    }
}

impl Error {
    #[tracing::instrument]
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        #[allow(unreachable_patterns)]
        match self {
            Self::LoginFail => {
                error!("Error in login: {:?}", self);
                (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
            }

            Self::CtxExt(_) => {
                error!("Error in CtxExt: {:?}", self);
                (StatusCode::FORBIDDEN, ClientError::NO_AUTH)
            }


            Self::TicketDeleteFailIdNotFound { .. }
            | Self::TicketGetFailIdNotFound { .. }
            | Self::TicketUpdateFailIdNotFound { .. } => {
                warn!("Error in ticket operation: {:?}", self);
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }


            Self::DocumentUploadFail => {
                error!("Error in document upload: {:?}", self);
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }

            Self::DocumentUploadFailOllama => {
                error!("Error in document upload Ollama: {:?}", self);
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }


            Self::ServiceError(_) => {
                error!("Service error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR)
            }

            Self::FailToCreatePool(_) => {
                error!("Failed to create pool: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::DATABASE_ERROR)
            }
            Self::ConfigMissingEnv(_) => {
                error!("Missing environment variable: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::CONFIG_ERROR)
            }
            Self::ConfigWrongFormat(_) => {
                error!("Configuration has wrong format: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::CONFIG_ERROR)
            }

            Self::SwiftideError(_) => {
                error!("SwiftideClient error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::CLIENT_ERROR)
            }
            Self::QdrantError(_) => {
                error!("QdrantClient error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::CLIENT_ERROR)
            }
            Self::OllamaError(_) => {
                error!("OllamaClient error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::CLIENT_ERROR)
            }


            Self::EntityNotFound { entity, id } => {
                warn!("Entity not found: {:?} with id: {:?}", entity, id);
                (StatusCode::NOT_FOUND, ClientError::DATABASE_ERROR)
            }


            Self::SqlxError(_) => {
                error!("Sqlx error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::DATABASE_ERROR)
            }

            Self::KeyFailHmac => {
                error!("Key failed HMAC: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::AUTH_FAIL)

            }

            Self::PwdNotMatching => {
                error!("Password not matching: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::AUTH_FAIL)
            }

            Self::CryptError(_) => {
                error!("Crypt error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::AUTH_FAIL)
            }

            Self::UserNotFound => {
                error!("User not found: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::AUTH_FAIL)
            }

            Self::UserHasNoPwd { user_id } => {
                error!("User has no password: {:?}", user_id);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::AUTH_FAIL)
            }

            Self::FailParseTime(time_str) => {
                error!("Failed to parse time: {:?}", time_str);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::TIME_ERROR)
            }

            Self::TokenInvalidFormat => {
                error!("Token has invalid format: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::TOKEN_ERROR)
            }

            Self::TokenCannotDecodeIdentifier => {
                error!("Token cannot decode identifier: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::TOKEN_ERROR)
            }

            Self::TokenCannotDecodeExpiration => {
                error!("Token cannot decode expiration: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::TOKEN_ERROR)
            }

            Self::TokenSignatureNotMatching => {
                error!("Token signature not matching: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::TOKEN_ERROR)
            }

            Self::TokenExpirationNotIso => {
                error!("Token expiration not ISO: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::TOKEN_ERROR)
            }

            Self::TokenExpired => {
                error!("Token expired: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::TOKEN_ERROR)
            }

            Self::QueryError(_) => {
                error!("Query error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::DATABASE_ERROR)
            }


            _ => {
                event!(
                    Level::ERROR,
                    error = ?self,
                    "Unknown error: {:?}",
                    self
                );
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR)
            }
        }
    }
}

#[derive(Debug, AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    AUTH_FAIL,
    INVALID_PARAMS,
    ENTITY_NOT_FOUND,
    DATABASE_ERROR,
    SERVICE_ERROR,
    CLIENT_ERROR,
    CONFIG_ERROR,
    TIME_ERROR,
    TOKEN_ERROR,
}
