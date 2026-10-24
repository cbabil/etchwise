//! HTTP access with upstream's curl settings (`curlfetcher.cpp`,
//! `downloadthread.cpp`): user agent `Mozilla/5.0 Etchwise/<version>`,
//! 30 s connect timeout, 60 s stall timeout (reqwest's blocking timeout
//! applies to each send and each body read), up to 10 redirects, system
//! proxy, and resumable downloads via `Range` after transient failures.

use std::io::{self, Read};
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::StatusCode;
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::header::{CONTENT_RANGE, RANGE};

use super::progress::Progress;
use crate::{APP_NAME, IMAGER_VERSION};

/// Retries after a failed transfer before giving up.
pub const MAX_RESUMES: u32 = 10;

/// `Mozilla/5.0 Etchwise/v1.0.0`.
pub fn user_agent() -> String {
    format!("Mozilla/5.0 {APP_NAME}/{IMAGER_VERSION}")
}

/// A builder with upstream's curl settings; reqwest's default redirect
/// policy already follows up to 10 redirects, as upstream does.
pub fn builder() -> ClientBuilder {
    Client::builder()
        .user_agent(user_agent())
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(60))
        .tcp_keepalive(Duration::from_secs(30))
}

/// A client configured like upstream's curl handles.
///
/// # Errors
/// Fails when the TLS backend cannot be initialised.
pub fn client() -> reqwest::Result<Client> {
    builder().build()
}

/// GETs a whole document (OS lists, sublists); returns the body and the
/// final URL after redirects.
///
/// # Errors
/// Fails on network errors or a non-success status.
pub fn fetch(client: &Client, url: &str) -> io::Result<(Vec<u8>, String)> {
    let response = get(client, url)?;
    let final_url = response.url().to_string();
    let body = response.bytes().map_err(io::Error::other)?;
    Ok((Vec::from(body), final_url))
}

fn get(client: &Client, url: &str) -> io::Result<Response> {
    client
        .get(url)
        .send()
        .and_then(Response::error_for_status)
        .map_err(io::Error::other)
}

/// A download that reconnects with `Range: bytes=<pos>-` when the
/// transfer breaks, and counts bytes into the shared progress.
pub struct ResumableDownload {
    client: Client,
    url: String,
    response: Option<Response>,
    position: u64,
    resumes: u32,
    last_failure: Option<Instant>,
    backoff: Duration,
    progress: Arc<Progress>,
}

impl ResumableDownload {
    /// Starts the request and records the total size when known.
    ///
    /// # Errors
    /// Fails when the server cannot be reached or refuses the request.
    pub fn open(client: Client, url: &str, progress: Arc<Progress>) -> io::Result<Self> {
        let response = get(&client, url)?;
        if let Some(total) = response.content_length() {
            Progress::set(&progress.download_total, total);
        }
        Ok(Self {
            client,
            url: url.to_owned(),
            response: Some(response),
            position: 0,
            resumes: 0,
            last_failure: None,
            backoff: Duration::from_secs(5),
            progress,
        })
    }

    /// Overrides the 5 s pause between quick consecutive failures.
    #[cfg(test)]
    pub fn with_backoff(mut self, backoff: Duration) -> Self {
        self.backoff = backoff;
        self
    }

    fn resume(&mut self, cause: io::Error) -> io::Result<()> {
        if self.resumes >= MAX_RESUMES || self.progress.is_cancelled() {
            return Err(cause);
        }
        if self
            .last_failure
            .is_some_and(|at| at.elapsed() < self.backoff)
        {
            std::thread::sleep(self.backoff);
        }
        self.last_failure = Some(Instant::now());
        self.resumes += 1;
        let response = self
            .client
            .get(&self.url)
            .header(RANGE, format!("bytes={}-", self.position))
            .send()
            .map_err(io::Error::other)?;
        let resumed_at = response
            .headers()
            .get(CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(range_start);
        if response.status() != StatusCode::PARTIAL_CONTENT || resumed_at != Some(self.position) {
            return Err(io::Error::other(format!(
                "server cannot resume the download ({})",
                response.status()
            )));
        }
        self.response = Some(response);
        Ok(())
    }
}

impl Read for ResumableDownload {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let Some(response) = self.response.as_mut() else {
                return Ok(0);
            };
            match response.read(buf) {
                Ok(n) => {
                    self.position += n as u64;
                    Progress::add(&self.progress.download_now, n as u64);
                    return Ok(n);
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => self.resume(e)?,
            }
        }
    }
}

/// Start offset of `bytes <start>-<end>/<total>`.
fn range_start(content_range: &str) -> Option<u64> {
    content_range
        .strip_prefix("bytes ")?
        .split('-')
        .next()?
        .trim()
        .parse()
        .ok()
}

#[cfg(test)]
mod tests;
