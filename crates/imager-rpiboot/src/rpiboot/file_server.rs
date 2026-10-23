//! Reactive file server (upstream `file_server.cpp`): after the second
//! stage boots, the device asks for files with `FileMessage`s read via
//! vendor control IN transfers; this answers until the device is done.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::file_requests::{
    FileResolver, acknowledge, get_file_size, read_file, read_file_from_disk,
};
use super::file_session::{Read, Session, Step, is_garbage, read_message};
use crate::transport::UsbTransport;
use crate::types::{DeviceMetadata, FileCommand, FileMessage, ProgressCallback};

/// Consecutive failed reads tolerated before giving up.
pub const MAX_RETRIES: u32 = 20;

/// Serves files and collects device metadata.
#[derive(Debug, Clone)]
pub struct FileServer {
    /// Pause between failed reads (upstream 1 s).
    pub retry_delay: Duration,
    metadata: DeviceMetadata,
}

impl Default for FileServer {
    fn default() -> Self {
        Self {
            retry_delay: Duration::from_secs(1),
            metadata: DeviceMetadata::default(),
        }
    }
}

impl FileServer {
    #[cfg(test)]
    /// Metadata collected from star-prefixed requests.
    pub fn metadata(&self) -> &DeviceMetadata {
        &self.metadata
    }

    /// Runs until the device sends Done or an empty name, or disconnects
    /// after files were served. `resolver` defaults to `firmware_dir`.
    ///
    /// # Errors
    /// Upstream's `lastError()` text, or `Cancelled`.
    pub fn run(
        &mut self,
        transport: &mut dyn UsbTransport,
        firmware_dir: &Path,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
        resolver: Option<FileResolver<'_>>,
    ) -> Result<(), String> {
        let from_disk = |name: &str| read_file_from_disk(firmware_dir, name);
        let resolver = resolver.unwrap_or(&from_disk);
        let mut session = Session::default();
        progress(0, 0, "Waiting for device file requests...");
        while !cancel.load(Ordering::SeqCst) {
            let step = match read_message(transport) {
                Read::Failed { code, fatal } => self.failed(&mut session, code, fatal, progress),
                Read::Message(m) if is_garbage(&m) => self.garbage(&mut session, &m, progress),
                Read::Message(m) => {
                    self.request(transport, &mut session, m, resolver, progress, cancel)
                }
            };
            if let Some(result) = step {
                return result;
            }
        }
        Err("Cancelled".into())
    }

    fn failed(
        &self,
        s: &mut Session,
        code: i32,
        fatal: bool,
        progress: ProgressCallback<'_>,
    ) -> Step {
        s.errors += 1;
        if fatal {
            let error = format!(
                "Device disconnected (libusb error {code}){}",
                s.last_file(" after serving: ")
            );
            return Some(s.end(progress, error));
        }
        if s.errors > MAX_RETRIES {
            let error = format!(
                "Failed to read FileMessage from device (error {code} after {} retries{})",
                s.errors,
                s.last_file("; last file: ")
            );
            return Some(s.end(progress, error));
        }
        let status = format!(
            "Waiting for device (retry {}/{MAX_RETRIES}, error {code})...",
            s.errors
        );
        progress(s.served, 0, &status);
        std::thread::sleep(self.retry_delay);
        None
    }

    /// Garbage must not reset the failure count or reach the resolver.
    fn garbage(
        &self,
        s: &mut Session,
        message: &FileMessage,
        progress: ProgressCallback<'_>,
    ) -> Step {
        s.errors += 1;
        if s.errors > MAX_RETRIES {
            let error = format!("Unknown file command {}", message.raw_command);
            return Some(s.end(progress, error));
        }
        std::thread::sleep(self.retry_delay);
        None
    }

    fn request(
        &mut self,
        transport: &mut dyn UsbTransport,
        s: &mut Session,
        message: FileMessage,
        resolver: FileResolver<'_>,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
    ) -> Step {
        s.errors = 0;
        s.last.clone_from(&message.name);
        let name = message.name.as_str();
        if name.is_empty() {
            // rpiboot treats an empty name (any command) as "done".
            acknowledge(transport);
            progress(s.served, s.served, "Device boot complete");
            return Some(Ok(()));
        }
        let handled = match message.command {
            Some(FileCommand::GetFileSize) => {
                progress(s.served, 0, &format!("Querying: {name}"));
                get_file_size(transport, name, resolver)
            }
            Some(FileCommand::ReadFile) => {
                s.served += 1;
                progress(s.served, 0, &format!("Sending: {name}"));
                read_file(transport, name, resolver, &mut self.metadata, cancel)
            }
            Some(FileCommand::Done) | None => {
                progress(s.served, s.served, "Device boot complete");
                return Some(Ok(()));
            }
        };
        handled.err().map(Err)
    }
}
