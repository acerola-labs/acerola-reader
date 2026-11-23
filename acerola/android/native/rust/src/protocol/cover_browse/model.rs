use serde::{Deserialize, Serialize};

/// The only frame written by the outbound side — the connection itself is not the request (unlike
/// `browse-library`), because here the outbound must say WHICH comic and which version it already
/// has cached, so the inbound can decide between `not_modified` and publishing a new blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CoverRequest {
    pub comic_name: String,
    /// `None` = "never fetched this cover before" — any version the inbound has counts
    /// as "changed".
    pub known_version: Option<i64>,
}

/// Discriminator of `CoverResponse.status` — no enum tag on the wire (same convention as the rest
/// of the protocol), just a `String` compared by value.
pub(crate) const STATUS_NOT_MODIFIED: &str = "not_modified";
pub(crate) const STATUS_CHANGED: &str = "changed";
pub(crate) const STATUS_UNAVAILABLE: &str = "unavailable";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CoverResponse {
    pub status: String,
    pub cover_version: Option<i64>,
    /// Only present when `status == "changed"` — blob hash (BLAKE3, hex) the outbound uses
    /// in `ChapterTransfer::fetch_reader` to pull the actual bytes.
    pub cover_hash: Option<String>,
}
