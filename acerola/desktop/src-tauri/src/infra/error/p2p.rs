use super::{ComicError, RpcError};

/// Bridge between this crate's internal errors and the `ConnectionError` that
/// `acerola_p2p::api::protocol::Handler::handle` requires as a return type. There is no
/// specific variant for an "application protocol error" in the lib, so we use
/// the generic `StreamFailed` bucket — the same one the lib's internal RPC already uses.
impl From<RpcError> for acerola_p2p::api::error::P2pError {
    fn from(err: RpcError) -> Self {
        acerola_p2p::api::error::P2pError::StreamFailed(err.to_string())
    }
}

impl From<ComicError> for acerola_p2p::api::error::P2pError {
    fn from(err: ComicError) -> Self {
        acerola_p2p::api::error::P2pError::StreamFailed(err.to_string())
    }
}
