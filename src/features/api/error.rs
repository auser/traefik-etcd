use axum::http::header::WWW_AUTHENTICATE;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::error;

pub type TraefikApiResult<T = (), E = TraefikApiError> = Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum TraefikApiError {
    /// Return `401 Unauthorized`
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Return `401 Unauthorized`
    #[error("Unauthorized")]
    Unauthorized,
    /// Return `422 Unprocessable Entity`
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("resource not found")]
    NotFound(String),

    /// Automatically return `500 Internal Server Error` on a `sqlx::Error`.
    #[error("an error occurred with the database: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Return `500 Internal Server Error`
    #[error("an error occurred")]
    InternalServerError,

    /// Serialize error
    #[error("error serializing yaml data")]
    Serialize(#[from] serde_yaml::Error),

    /// Serialize error
    #[error("error serializing json data")]
    SerializeJson(#[from] serde_json::Error),

    /// Return `409 Conflict`
    #[error("resource already exists")]
    Conflict,
}

impl TraefikApiError {
    /// Convinient constructor for `Error::UnprocessableEntity`.
    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        Self::UnprocessableEntity {
            errors: errors
                .into_iter()
                .map(|(k, v)| (k.into(), vec![v.into()]))
                .collect(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Serialize(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SerializeJson(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Conflict => StatusCode::CONFLICT,
        }
    }
}

/// Axum allows you to return `Result` from handler functions, but the error type
/// also must be some sort of response type.
impl IntoResponse for TraefikApiError {
    fn into_response(self) -> Response {
        match self {
            Self::UnprocessableEntity { errors } => {
                #[derive(serde::Serialize)]
                struct Errors {
                    errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
                }

                return (StatusCode::UNPROCESSABLE_ENTITY, Json(Errors { errors })).into_response();
            }
            Self::Unauthorized => {
                return (
                    self.status_code(),
                    [(WWW_AUTHENTICATE, HeaderValue::from_static("Token"))]
                        .into_iter()
                        .collect::<HeaderMap>(),
                    self.to_string(),
                )
                    .into_response();
            }

            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

/// A little helper trait for more easily converting database constraint errors into API errors.
///
/// ```rust,ignore
/// let user_id = sqlx::query_scalar!(
///         r#"insert into "user" (username, email, password_hash) values ($1, $2, $3) returning user_id"#,
///         username,
///         email,
///         password_hash,
///     )
///     .fetch_one(&ctx.db)
///     .await
///     .on_constraint("user_username_key", |_| Error::unprocessable_entity([("username", "already taken")]))?;
/// ```
pub trait ResultExt<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn sqlx::error::DatabaseError>) -> TraefikApiError,
    ) -> Result<T, TraefikApiError>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<TraefikApiError>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn sqlx::error::DatabaseError>) -> TraefikApiError,
    ) -> Result<T, TraefikApiError> {
        self.map_err(|e| match e.into() {
            TraefikApiError::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            e => e,
        })
    }
}

impl From<walkdir::Error> for TraefikApiError {
    fn from(e: walkdir::Error) -> Self {
        error!("Error walking directory: {:?}", e);
        Self::InternalServerError
    }
}

impl From<std::time::SystemTimeError> for TraefikApiError {
    fn from(e: std::time::SystemTimeError) -> Self {
        error!("Error getting system time: {:?}", e);
        Self::InternalServerError
    }
}
