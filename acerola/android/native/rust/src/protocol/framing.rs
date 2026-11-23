//! Length-delimited (JSON) frame reading/writing shared by all P2P protocols
//! (files/sync-comic, history, library-browse, cover-browse). Before this consolidation, each one
//! reimplemented the SAME logic by hand — four nearly identical copies, each with its
//! own error text and (for no documented reason) its own timeout value — and one
//! of them classified the read timeout as a generic `P2pError::StreamFailed` instead of
//! `P2pError::Timeout`, making the whole session fail with NO retry attempt at all, even
//! when the problem was a passing hiccup (found live in production). A single
//! implementation here guarantees that the right classification (`Timeout`, recognized by
//! `classify_sync_error` and fully retried by `handle_connect_command` in `acerola-p2p`)
//! applies equally to every protocol, without having to remember to replicate the fix in N places.

use std::time::Duration;

use acerola_p2p::api::error::P2pError;
use futures::SinkExt;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub(crate) type Recv = FramedRead<Box<dyn AsyncRead + Send + Unpin>, LengthDelimitedCodec>;
pub(crate) type Writer = FramedWrite<Box<dyn AsyncWrite + Send + Unpin>, LengthDelimitedCodec>;

/// Reads a raw frame (bytes), with timeout — each layer of the nested type that
/// `tokio::time::timeout(_, recv.next())` returns (`Result<Option<Result<BytesMut, io::Error>>,
/// Elapsed>`) becomes a DIFFERENT cause: time ran out, the stream closed before anything
/// arrived, or reading/decoding the frame itself failed. `P2pError::Timeout` (not
/// `StreamFailed`) on purpose only for the first case — it is the variant `classify_sync_error`
/// already recognizes and that `handle_connect_command` (lib/p2p) already treats as transient enough
/// to retry the whole session (new dial + handler again); the other two stay
/// `StreamFailed`, which is not retried today.
pub(crate) async fn read_bytes(recv: &mut Recv, timeout: Duration) -> Result<Vec<u8>, P2pError> {
    let frame = tokio::time::timeout(timeout, recv.next())
        .await
        .map_err(|_| P2pError::Timeout)?
        .ok_or_else(|| P2pError::StreamFailed("stream closed before frame".into()))?
        .map_err(|err| P2pError::StreamFailed(format!("failed to read frame: {err}")))?;

    Ok(frame.to_vec())
}

/// Reads and decodes a JSON frame, with timeout — same error classification as [`read_bytes`].
pub(crate) async fn read_json<T: DeserializeOwned>(
    recv: &mut Recv, timeout: Duration,
) -> Result<T, P2pError> {
    let bytes = read_bytes(recv, timeout).await?;
    serde_json::from_slice(&bytes)
        .map_err(|err| P2pError::StreamFailed(format!("failed to decode frame: {err}")))
}

/// Writes a value as a standalone JSON frame (no enum tag) — same format Desktop uses
/// in `infra/sync/framing.rs::write_json`.
pub(crate) async fn write_json<T: Serialize>(
    writer: &mut Writer, value: &T,
) -> Result<(), P2pError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|err| P2pError::StreamFailed(format!("failed to encode frame: {err}")))?;
    writer
        .send(bytes.into())
        .await
        .map_err(|err| P2pError::StreamFailed(format!("failed to write frame: {err}")))
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use tokio_util::codec::{FramedRead, FramedWrite};

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Ping {
        n: u32,
    }

    fn framed_pair() -> (Writer, Recv) {
        let (client, server) = tokio::io::duplex(1024);
        let writer: Writer = FramedWrite::new(Box::new(client), LengthDelimitedCodec::new());
        let reader: Recv = FramedRead::new(Box::new(server), LengthDelimitedCodec::new());
        (writer, reader)
    }

    #[tokio::test]
    async fn write_json_then_read_json_roundtrips() {
        let (mut writer, mut reader) = framed_pair();

        write_json(&mut writer, &Ping { n: 42 }).await.unwrap();
        let received: Ping = read_json(&mut reader, Duration::from_secs(1)).await.unwrap();

        assert_eq!(received, Ping { n: 42 });
    }

    /// Core regression: a read timeout must become `P2pError::Timeout` — not
    /// `StreamFailed` — so `handle_connect_command` (lib/p2p) knows it is worth retrying the
    /// whole session.
    #[tokio::test]
    async fn read_json_times_out_as_a_typed_timeout_not_a_generic_stream_failure() {
        let (_writer, mut reader) = framed_pair();

        let result: Result<Ping, P2pError> =
            read_json(&mut reader, Duration::from_millis(20)).await;

        assert!(
            matches!(result, Err(P2pError::Timeout)),
            "expected P2pError::Timeout, got {result:?}"
        );
    }

    /// Unhappy path different from the timeout: the other side closed the stream on purpose (or
    /// dropped) BEFORE sending any frame — that is `StreamFailed`, not `Timeout`, because
    /// retrying immediately changes nothing (the connection no longer exists).
    #[tokio::test]
    async fn read_json_reports_a_clean_stream_close_as_stream_failed_not_timeout() {
        let (writer, mut reader) = framed_pair();
        drop(writer);

        let result: Result<Ping, P2pError> =
            read_json(&mut reader, Duration::from_secs(1)).await;

        assert!(
            matches!(result, Err(P2pError::StreamFailed(_))),
            "expected StreamFailed (stream closed), got {result:?}"
        );
    }

    #[tokio::test]
    async fn read_json_reports_malformed_json_as_stream_failed() {
        let (mut writer, mut reader) = framed_pair();

        writer.send(b"not valid json".to_vec().into()).await.unwrap();

        let result: Result<Ping, P2pError> =
            read_json(&mut reader, Duration::from_secs(1)).await;

        assert!(
            matches!(result, Err(P2pError::StreamFailed(_))),
            "expected StreamFailed (malformed JSON), got {result:?}"
        );
    }
}
