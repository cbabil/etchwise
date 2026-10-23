//! Write failures, with upstream's English messages (the UI translates).

/// Why a write stopped.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("Cancelled")]
    Cancelled,
    #[error(
        "Error writing to storage device during {action}.\n\nThis could be caused by:\n\u{2022} Device disconnected or became unresponsive\n\u{2022} Device is full or write-protected\n\u{2022} Hardware failure or bad sectors\n\nPlease check the device and try again."
    )]
    Io {
        action: &'static str,
        source: std::io::Error,
    },
    #[error(
        "Write error while trying to zero out last part of card.\n\nThis could indicate the card is advertising wrong capacity (possible counterfeit).\n\nPlease try a different storage device."
    )]
    EndOfDevice,
    #[error(
        "Download appears to be corrupt. SHA256 hash does not match.<br>Expected: {expected}<br>Actual: {actual}<br>Please check your network connection and try again."
    )]
    HashMismatch { expected: String, actual: String },
    #[error("Error reading from storage.<br>SD card may be broken.")]
    VerifyRead,
    #[error(
        "Verifying write failed. Contents of SD card is different from what was written to it."
    )]
    VerifyMismatch,
    #[error("Error customising OS: {0}")]
    Customise(std::io::Error),
    #[error(
        "Input file is not a valid disk image.\n\nFile size {size} bytes is not a multiple of 512 bytes."
    )]
    NotSectorMultiple { size: u64 },
    #[error("Storage capacity is not large enough.\n\nThe image requires at least {} of storage.", crate::format::format_size(*needed, 1))]
    TooSmall { needed: u64 },
    #[error("Error extracting archive: {0}")]
    Decompress(String),
    /// Formatting before a multi-file extraction failed.
    #[error(transparent)]
    Format(#[from] crate::erase::EraseError),
}

impl WriteError {
    /// Wraps an I/O error with what was being done.
    pub fn io(action: &'static str) -> impl FnOnce(std::io::Error) -> Self {
        move |source| Self::Io { action, source }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_match_upstream() {
        let io = WriteError::io("final sync")(std::io::Error::other("x"));
        assert!(
            io.to_string()
                .starts_with("Error writing to storage device during final sync.\n\n")
        );
        assert_eq!(WriteError::Cancelled.to_string(), "Cancelled");
        assert!(
            WriteError::EndOfDevice
                .to_string()
                .contains("possible counterfeit")
        );
        assert!(
            WriteError::VerifyRead
                .to_string()
                .contains("SD card may be broken")
        );
        assert!(
            WriteError::VerifyMismatch
                .to_string()
                .starts_with("Verifying write failed.")
        );
        assert!(
            WriteError::Customise(std::io::Error::other("no fat"))
                .to_string()
                .ends_with("no fat")
        );
        assert_eq!(
            WriteError::TooSmall {
                needed: 6 * 1024 * 1024 * 1024
            }
            .to_string(),
            "Storage capacity is not large enough.\n\nThe image requires at least 6 GB of storage."
        );
        assert!(
            WriteError::NotSectorMultiple { size: 3 }
                .to_string()
                .contains("File size 3 bytes")
        );
    }
}
