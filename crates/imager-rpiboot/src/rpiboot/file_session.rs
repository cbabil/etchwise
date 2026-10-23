//! Reading and classifying the device's `FileMessage`s, and the file
//! server's loop state (upstream `FileServer::run` internals).

use crate::transport::UsbTransport;
use crate::types::{FileMessage, ProgressCallback, VENDOR_REQUEST, VENDOR_REQUEST_TYPE};

/// Timeout for one `FileMessage` read; upstream uses 3 s with retries
/// instead of rpiboot's single 20 s read so the UI stays responsive.
const EP_READ_TIMEOUT_MS: u32 = 3000;
const LIBUSB_ERROR_IO: i32 = -1;
const LIBUSB_ERROR_NO_DEVICE: i32 = -4;

/// What one read of the device produced.
pub enum Read {
    /// A message of at least 4 bytes.
    Message(FileMessage),
    /// Fewer than 4 bytes: upstream's `bytesRead` or libusb code, and
    /// whether it is a fatal `LIBUSB_ERROR_IO` / `NO_DEVICE`. A failure
    /// whose code the transport cannot tell is retried (shown as -1).
    Failed { code: i32, fatal: bool },
}

pub fn read_message(transport: &mut dyn UsbTransport) -> Read {
    let mut buf = [0u8; FileMessage::SIZE];
    let read = transport.control_in(
        VENDOR_REQUEST_TYPE | 0x80,
        VENDOR_REQUEST,
        FileMessage::SIZE as u16,
        0,
        &mut buf,
        EP_READ_TIMEOUT_MS,
    );
    match read {
        Some(n) if n >= 4 => Read::Message(FileMessage::parse(&buf)),
        Some(n) => Read::Failed {
            code: n as i32,
            fatal: false,
        },
        None => match transport.last_error_code() {
            Some(code) => Read::Failed {
                code,
                fatal: code == LIBUSB_ERROR_IO || code == LIBUSB_ERROR_NO_DEVICE,
            },
            None => Read::Failed {
                code: -1,
                fatal: false,
            },
        },
    }
}

/// An invalid command or a name that is not printable ASCII: reads after
/// the device reboots may return garbage.
pub fn is_garbage(message: &FileMessage) -> bool {
    message.command.is_none() || !message.name.chars().all(|c| (' '..='~').contains(&c))
}

/// Loop state: files served, consecutive failures, last file name.
#[derive(Default)]
pub struct Session {
    pub served: u64,
    pub errors: u32,
    pub last: String,
}

impl Session {
    pub fn last_file(&self, prefix: &str) -> String {
        if self.last.is_empty() {
            String::new()
        } else {
            format!("{prefix}{}", self.last)
        }
    }

    /// Success once files were served (the device rebooted into the next
    /// stage without sending Done), otherwise `error`.
    pub fn end(&self, progress: ProgressCallback<'_>, error: String) -> Result<(), String> {
        if self.served > 0 {
            progress(self.served, self.served, "Device rebooted into next stage");
            Ok(())
        } else {
            Err(error)
        }
    }
}

/// `Some(result)` ends the loop.
pub type Step = Option<Result<(), String>>;
