use std::{io, path::Path};

/// Reads the whole `path`, returning `None` if the file simply does not exist. Only used today
/// to migrate the legacy plain-text files (`identity.seed`/`peers.json`/
/// `trusted.txt`/`blocked.txt`) to `SecureBlobStore` — see `storage.rs`/`trust_store.rs`.
pub(crate) fn read_optional(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}
