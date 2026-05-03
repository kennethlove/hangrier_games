//! Tiny shim around reqwest builder so cookies are attached on the WASM
//! target (`fetch_credentials_include` only exists in reqwest's WASM
//! backend) while host-target builds (cargo tests) compile cleanly.

use reqwest::RequestBuilder;

pub trait WithCredentials {
    /// Ensure the browser sends cookies with this cross-origin request.
    /// No-op when compiled for non-WASM targets (used by host-side tests).
    fn with_credentials(self) -> Self;
}

impl WithCredentials for RequestBuilder {
    #[cfg(target_arch = "wasm32")]
    fn with_credentials(self) -> Self {
        self.fetch_credentials_include()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn with_credentials(self) -> Self {
        self
    }
}
