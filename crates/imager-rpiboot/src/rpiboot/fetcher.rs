//! Network access for firmware downloads, injectable so tests run offline.
//! The real fetcher uses imager-core's blocking HTTP client (upstream's
//! curl settings and user agent).

use std::io;

use imager_core::pipeline::http;

/// Downloads a whole document.
pub trait Fetcher: Send + Sync {
    /// GETs `url` and returns its body.
    ///
    /// # Errors
    /// Network errors and non-success statuses.
    fn fetch(&self, url: &str) -> io::Result<Vec<u8>>;
}

type Get = dyn Fn(&str) -> io::Result<Vec<u8>> + Send + Sync;

/// Fetches over HTTPS with `imager_core::pipeline::http`.
pub struct HttpFetcher {
    get: Box<Get>,
}

impl HttpFetcher {
    /// Builds the HTTP client.
    ///
    /// # Errors
    /// Fails when the TLS backend cannot be initialised.
    pub fn new() -> io::Result<Self> {
        let client = http::client().map_err(io::Error::other)?;
        Ok(Self {
            get: Box::new(move |url| http::fetch(&client, url).map(|(body, _)| body)),
        })
    }
}

impl Fetcher for HttpFetcher {
    fn fetch(&self, url: &str) -> io::Result<Vec<u8>> {
        (self.get)(url)
    }
}
