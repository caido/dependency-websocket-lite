use flate2::Compression;

// Parameters `server_max_window_bits` and `client_max_window_bits` are not supported for now
// because custom window size requires `flate2/zlib` feature.
/// Configurations for `permessage-deflate` Per-Message Compression Extension.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeflateConfig {
    /// Compression level.
    pub compression: Compression,
    /// Request the peer server not to use context takeover.
    pub server_no_context_takeover: bool,
    /// Hint that context takeover is not used.
    pub client_no_context_takeover: bool,
}
