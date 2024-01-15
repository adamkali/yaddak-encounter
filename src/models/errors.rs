use std::env::VarError;

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Clone, ToSchema)]
pub struct YaddakError {
    pub kind: YaddakErrorKind,
    message: String,
}

#[derive(Serialize, Clone, ToSchema)]
pub enum YaddakErrorKind {
    EnvError,
    InternalError,
    AuthError,
    DBError,
}


impl std::fmt::Display for YaddakError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            YaddakErrorKind::EnvError => {
                write!(f, "[E001] Environment Variable Error: {}", self.message) 
            },
            YaddakErrorKind::InternalError => {
                write!(f, "[E002] Internal Error: {}", self.message) 
            },
            YaddakErrorKind::AuthError => {
                write!(f, "[E003] Authentication Error: {}", self.message) 
            },
            YaddakErrorKind::DBError => {
                write!(f, "[E004] Database Error:  {}", self.message)
            }
        }
    }
}

impl From<VarError> for YaddakError {
    fn from(value: VarError) -> Self {
        return YaddakError {
            kind: YaddakErrorKind::EnvError,
            message: value.to_string()
        }
    }
}

impl From<argon2::Error> for YaddakError {
    fn from(value: argon2::Error) -> Self {
        return YaddakError {
            kind: YaddakErrorKind::InternalError,
            message: value.to_string()
        }
    }
}

impl From<sqlx::Error> for YaddakError {
    fn from(value: sqlx::Error) -> Self {
        return YaddakError {
            kind: YaddakErrorKind::DBError,
            message: value.to_string()
        };
    }
}

impl From<sea_query::error::Error> for YaddakError {
    fn from(value: sea_query::error::Error) -> Self {
        return YaddakError {
            kind: YaddakErrorKind::DBError,
            message: value.to_string()
        };
    }
}

impl YaddakError {
    pub fn authorize_error(message: String)
    -> Self {
        Self {
            kind: YaddakErrorKind::AuthError,
            message,
        }
    }
    pub fn database_error(message: String)
    -> Self {
        Self {
            kind: YaddakErrorKind::DBError,
            message,
        }
    }
}

pub type SResult<T> = Result<T, YaddakError>;
