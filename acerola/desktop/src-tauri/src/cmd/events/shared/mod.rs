use serde::Serialize;

use crate::infra::error::ComicError;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub error_type: String,
    pub message: String,
}

impl From<&ComicError> for ErrorPayload {
    fn from(err: &ComicError) -> Self {
        // Bare variant name (without the `Debug` data), to match exactly the key used in
        // `COMIC_ERROR_MESSAGES` (errors.i18n.ts) — `format!("{:?}", err)` included the
        // internal data (e.g. `InvalidRequest("msg")`), which never matched the key.
        let error_type = match err {
            ComicError::AlreadyExists => "AlreadyExists",
            ComicError::SyncInProgress => "SyncInProgress",
            ComicError::NotFound => "NotFound",
            ComicError::ComicInfoNotFound => "ComicInfoNotFound",
            ComicError::InvalidRequest(_) => "InvalidRequest",
            ComicError::IntegrityViolation => "IntegrityViolation",
            ComicError::SystemFailure(_) => "SystemFailure",
            ComicError::Io(_) => "Io",
        };
        ErrorPayload { error_type: error_type.to_string(), message: err.to_string() }
    }
}
