//! LSP message transport framing and async stream parsing.

use std::fmt;

/// Framing and transport errors.
#[derive(Debug, thiserror::Error)]
pub enum LspFramingError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Missing Content-Length header")]
    MissingContentLength,

    #[error("UTF-8 decoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unexpected EOF while parsing message frame")]
    UnexpectedEof,

    #[error("Connection closed")]
    ConnectionClosed,
}

/// Encodes a raw text payload into an LSP framed message byte vector.
/// Format: `Content-Length: <len>\r\n\r\n<payload>`
pub fn encode_message(payload: &str) -> Vec<u8> {
    let header = format!("Content-Length: {}\r\n\r\n", payload.len());
    let mut bytes = Vec::with_capacity(header.len() + payload.len());
    bytes.extend_from_slice(header.as_bytes());
    bytes.extend_from_slice(payload.as_bytes());
    bytes
}

/// Encodes a JSON value into an LSP framed message byte vector.
pub fn encode_json_value(value: &serde_json::Value) -> Result<Vec<u8>, serde_json::Error> {
    let json_str = serde_json::to_string(value)?;
    Ok(encode_message(&json_str))
}

/// Finds the header separator in an LSP byte stream.
/// Returns (offset_to_separator_start, separator_length).
fn find_header_separator(buffer: &[u8]) -> Option<(usize, usize)> {
    if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
        return Some((pos, 4));
    }
    // Permissive fallback for non-standard servers using \n\n
    if let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
        return Some((pos, 2));
    }
    None
}

/// Attempts to parse a single complete LSP frame from `buffer`.
///
/// Returns `Ok(Some((consumed_bytes, payload_string)))` if a complete frame is parsed.
/// Returns `Ok(None)` if more data is needed.
/// Returns `Err(...)` if headers are malformed.
pub fn parse_frame(buffer: &[u8]) -> Result<Option<(usize, String)>, LspFramingError> {
    let (header_end, sep_len) = match find_header_separator(buffer) {
        Some(found) => found,
        None => return Ok(None),
    };

    let header_bytes = &buffer[..header_end];
    let header_str = match std::str::from_utf8(header_bytes) {
        Ok(s) => s,
        Err(_) => {
            return Err(LspFramingError::InvalidHeader(
                "Headers contain invalid UTF-8".to_string(),
            ))
        }
    };

    let mut content_length: Option<usize> = None;
    for line in header_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((key, val)) = line.split_once(':') {
            if key.trim().eq_ignore_ascii_case("Content-Length") {
                let len = val.trim().parse::<usize>().map_err(|e| {
                    LspFramingError::InvalidHeader(format!("Invalid Content-Length value: {e}"))
                })?;
                content_length = Some(len);
            }
            // Additional headers such as Content-Type are intentionally ignored.
        }
    }

    let length = match content_length {
        Some(len) => len,
        None => return Err(LspFramingError::MissingContentLength),
    };

    let payload_start = header_end + sep_len;
    let frame_len = payload_start + length;

    if buffer.len() < frame_len {
        return Ok(None);
    }

    let payload_slice = &buffer[payload_start..frame_len];
    let payload = String::from_utf8(payload_slice.to_vec())?;

    Ok(Some((frame_len, payload)))
}

/// Asynchronous framed message reader wrapping any `tokio::io::AsyncRead`.
pub struct LspFramedReader<R> {
    reader: R,
    buffer: Vec<u8>,
}

impl<R: fmt::Debug> fmt::Debug for LspFramedReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LspFramedReader")
            .field("reader", &self.reader)
            .field("buffered_bytes", &self.buffer.len())
            .finish()
    }
}

impl<R: tokio::io::AsyncRead + Unpin> LspFramedReader<R> {
    /// Creates a new framed reader with an initial 4KB buffer.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: Vec::with_capacity(4096),
        }
    }

    /// Reads the next complete LSP payload from the stream.
    ///
    /// Returns `Ok(Some(payload))` on success.
    /// Returns `Ok(None)` on clean EOF when buffer is empty.
    /// Returns `Err(...)` on malformed headers, invalid framing, or unexpected EOF.
    pub async fn read_message(&mut self) -> Result<Option<String>, LspFramingError> {
        use tokio::io::AsyncReadExt;

        loop {
            if let Some((consumed, payload)) = parse_frame(&self.buffer)? {
                self.buffer.drain(..consumed);
                return Ok(Some(payload));
            }

            let mut chunk = [0u8; 4096];
            let bytes_read = self.reader.read(&mut chunk).await?;
            if bytes_read == 0 {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(LspFramingError::UnexpectedEof);
                }
            }

            self.buffer.extend_from_slice(&chunk[..bytes_read]);
        }
    }

    /// Access the underlying reader.
    pub fn get_ref(&self) -> &R {
        &self.reader
    }

    /// Mutably access the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consume the wrapper and return the inner reader.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

/// Asynchronously writes a framed LSP message to an `AsyncWrite` sink.
pub async fn write_message<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &str,
) -> Result<(), LspFramingError> {
    use tokio::io::AsyncWriteExt;

    let encoded = encode_message(payload);
    writer.write_all(&encoded).await?;
    writer.flush().await?;
    Ok(())
}
