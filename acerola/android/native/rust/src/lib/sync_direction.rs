use crate::protocol::files::SyncDirection;

/// Explicit direction of a `P2PNode::sync_comic`, exposed to the FFI boundary (Kotlin) — same
/// pattern as `FfiNetworkMode`/`NetworkMode` in `lib/mode.rs`. `SyncDirection` (the protocol's
/// internal type, in `protocol::files::model`) cannot be exported via UniFFI directly because it lives
/// in a `pub(crate)` module, so this mirrored enum is what Kotlin actually sees.
#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfiSyncDirection {
    Push,
    Pull,
}

impl From<FfiSyncDirection> for SyncDirection {
    fn from(value: FfiSyncDirection) -> Self {
        match value {
            FfiSyncDirection::Push => SyncDirection::Push,
            FfiSyncDirection::Pull => SyncDirection::Pull,
        }
    }
}
