use std::{path::Path, time::Duration};

use futures::SinkExt;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::infra::error::RpcError;

/// Maximum size of a frame (JSON messages and file chunks), in bytes.
pub const MAX_FRAME_LEN: usize = 1024 * 1024;
/// Size of each chunk when streaming a file — keeps memory usage constant
/// regardless of the size of the cbz/cbr being transferred.
pub const FILE_CHUNK_SIZE: usize = 256 * 1024;

const FRAME_TIMEOUT: Duration = Duration::from_secs(30);

pub type FramedReader = FramedRead<Box<dyn AsyncRead + Send + Unpin>, LengthDelimitedCodec>;
pub type FramedWriter = FramedWrite<Box<dyn AsyncWrite + Send + Unpin>, LengthDelimitedCodec>;

fn codec() -> LengthDelimitedCodec {
    LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec()
}

pub fn framed_reader(recv: Box<dyn AsyncRead + Send + Unpin>) -> FramedReader {
    FramedRead::new(recv, codec())
}

pub fn framed_writer(send: Box<dyn AsyncWrite + Send + Unpin>) -> FramedWriter {
    FramedWrite::new(send, codec())
}

pub async fn write_json<T: Serialize>(
    writer: &mut FramedWriter, value: &T,
) -> Result<(), RpcError> {
    let bytes = serde_json::to_vec(value)?;
    writer.send(bytes.into()).await?;
    Ok(())
}

pub async fn read_json<T: DeserializeOwned>(reader: &mut FramedReader) -> Result<T, RpcError> {
    let bytes = tokio::time::timeout(FRAME_TIMEOUT, reader.next())
        .await
        .map_err(|_| RpcError::Stream("timeout waiting for frame".into()))?
        .ok_or_else(|| RpcError::Stream("stream closed".into()))??;

    Ok(serde_json::from_slice(&bytes)?)
}

pub async fn write_bytes(writer: &mut FramedWriter, bytes: &[u8]) -> Result<(), RpcError> {
    writer.send(bytes.to_vec().into()).await?;
    Ok(())
}

pub async fn read_bytes(reader: &mut FramedReader) -> Result<Vec<u8>, RpcError> {
    let bytes = tokio::time::timeout(FRAME_TIMEOUT, reader.next())
        .await
        .map_err(|_| RpcError::Stream("timeout waiting for frame".into()))?
        .ok_or_else(|| RpcError::Stream("stream closed".into()))??;

    Ok(bytes.to_vec())
}

/// Sends the content of `path` in [`FILE_CHUNK_SIZE`] chunks, without loading the whole
/// file into memory — needed for cbz/cbr files that can be hundreds of MB.
pub async fn send_file_bytes(
    writer: &mut FramedWriter, path: &Path, expected_size: u64,
) -> Result<(), RpcError> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut buffer = vec![0u8; FILE_CHUNK_SIZE];
    let mut sent: u64 = 0;

    while sent < expected_size {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }

        write_bytes(writer, &buffer[..read]).await?;
        sent += read as u64;
    }

    Ok(())
}

/// Receives a file streaming it straight to disk (`dest`), computing the SHA-256 along
/// the way so the file does not have to be re-read later just to verify integrity. Same
/// algorithm and format as Android (lowercase hex, no separator), since the result is
/// compared string-for-string against the checksum announced in the header by the peer.
/// Returns the computed checksum, which the caller compares against the one announced in the manifest.
pub async fn receive_file_to_disk(
    reader: &mut FramedReader, dest: &Path, expected_size: u64,
) -> Result<String, RpcError> {
    use sha2::{Digest, Sha256};

    let mut file = tokio::fs::File::create(dest).await?;
    let mut hasher = Sha256::new();
    let mut received: u64 = 0;

    while received < expected_size {
        let chunk = read_bytes(reader).await?;
        if chunk.is_empty() {
            break;
        }

        file.write_all(&chunk).await?;
        hasher.update(&chunk);
        received += chunk.len() as u64;
    }

    file.flush().await?;
    Ok(format!("{:x}", hasher.finalize()))
}
