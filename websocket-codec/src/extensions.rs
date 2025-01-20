/// Container for configured extensions.
#[derive(Debug, Default)]
#[allow(missing_copy_implementations)]
pub struct Extensions {
    // Per-Message Compression. Only `permessage-deflate` is supported.
    #[cfg(feature = "deflate")]
    pub(crate) compression: Option<DeflateContext>,
}
