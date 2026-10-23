//! Core rpiboot types and constants (upstream `rpiboot_types.h`).

/// Broadcom USB vendor id shared by all Pi silicon.
pub const BROADCOM_VID: u16 = 0x0a5c;
/// Largest bulk transfer chunk.
pub const BULK_CHUNK_SIZE: usize = 16 * 1024;
/// `bmRequestType` for vendor OUT.
pub const VENDOR_REQUEST_TYPE: u8 = 0x40;
/// `bRequest`.
pub const VENDOR_REQUEST: u8 = 0;
pub const DEFAULT_TIMEOUT_MS: u32 = 3000;
/// The Raspberry Pi fastboot gadget's ids after sideloading (Android's
/// standard fastboot PID is 0x4ee0; the Pi gadget uses 0x4e40).
pub const FASTBOOT_VID: u16 = 0x18d1;
pub const FASTBOOT_PID: u16 = 0x4e40;

/// Chip generations in USB boot mode, by product id. BCM2835 (0x2763) is
/// intentionally unsupported, as upstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChipGeneration {
    /// CM3/CM3+, Pi 2/3 (BCM2836 and BCM2837 share the PID).
    Bcm2836_7,
    /// CM4, Pi 4.
    Bcm2711,
    /// CM5, Pi 5.
    Bcm2712,
}

impl ChipGeneration {
    pub fn from_pid(pid: u16) -> Option<Self> {
        match pid {
            0x2764 => Some(Self::Bcm2836_7),
            0x2711 => Some(Self::Bcm2711),
            0x2712 => Some(Self::Bcm2712),
            _ => None,
        }
    }

    pub fn pid(self) -> u16 {
        match self {
            Self::Bcm2836_7 => 0x2764,
            Self::Bcm2711 => 0x2711,
            Self::Bcm2712 => 0x2712,
        }
    }

    /// `chipGenerationName`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Bcm2836_7 => "BCM2836/7",
            Self::Bcm2711 => "BCM2711",
            Self::Bcm2712 => "BCM2712",
        }
    }

    /// Directory prefix inside `bootfiles.bin` (`chipDirectoryPrefix`).
    pub fn directory_prefix(self) -> &'static str {
        match self {
            Self::Bcm2836_7 => "2836",
            Self::Bcm2711 => "2711",
            Self::Bcm2712 => "2712",
        }
    }

    /// Drive-list description (`deviceDescription`).
    pub fn description(self) -> &'static str {
        match self {
            Self::Bcm2836_7 => "Compute Module 3 (USB Boot)",
            Self::Bcm2711 => "Compute Module 4 (USB Boot)",
            Self::Bcm2712 => "Compute Module 5 (USB Boot)",
        }
    }
}

/// File-server commands sent by the device firmware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCommand {
    GetFileSize,
    ReadFile,
    Done,
}

/// A 260-byte `FileMessage`: a little-endian i32 command and a 256-byte
/// NUL-padded file name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMessage {
    pub command: Option<FileCommand>,
    pub raw_command: i32,
    pub name: String,
}

impl FileMessage {
    pub const SIZE: usize = 260;

    /// Parses the wire format; unknown commands keep their raw value.
    pub fn parse(bytes: &[u8; Self::SIZE]) -> Self {
        let raw_command = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let command = match raw_command {
            0 => Some(FileCommand::GetFileSize),
            1 => Some(FileCommand::ReadFile),
            2 => Some(FileCommand::Done),
            _ => None,
        };
        let field = &bytes[4..];
        let end = field.iter().position(|b| *b == 0).unwrap_or(field.len());
        Self {
            command,
            raw_command,
            name: String::from_utf8_lossy(&field[..end]).into_owned(),
        }
    }
}

/// The 24-byte boot message: payload length then a 20-byte signature
/// (zero when unsigned).
pub fn boot_message(length: i32, signature: [u8; 20]) -> [u8; 24] {
    let mut message = [0u8; 24];
    message[..4].copy_from_slice(&length.to_le_bytes());
    message[4..].copy_from_slice(&signature);
    message
}

/// What to sideload. Upstream's GUI only ever uses `Fastboot`; its
/// `SecureBootRecovery` (irreversible OTP key programming) is unreachable
/// there and is not ported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideloadMode {
    Fastboot,
}

/// Collected from star-prefixed file requests during the file-server phase.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeviceMetadata {
    pub serial_number: Option<String>,
    pub mac_address: Option<String>,
    pub otp_state: Option<String>,
    pub board_revision: Option<u32>,
}

/// `(current, total, status)`: percentages, step indices, or `total == 0`
/// for an indeterminate phase.
pub type ProgressCallback<'a> = &'a mut dyn FnMut(u64, u64, &str);

/// Parses leading digits like `std::stoul`: leading whitespace, then an
/// optional `0x` for base 16 (a bare `0x` reads as 0); `None` when no
/// digit is found.
pub(crate) fn parse_leading(text: &str, radix: u32) -> Option<u64> {
    let s = text.trim_start();
    let digits = if radix == 16 {
        s.strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .filter(|rest| rest.starts_with(|c: char| c.is_ascii_hexdigit()))
            .unwrap_or(s)
    } else {
        s
    };
    let end = digits
        .find(|c: char| !c.is_digit(radix))
        .unwrap_or(digits.len());
    u64::from_str_radix(&digits[..end], radix).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_product_ids_and_names() {
        for chip in [
            ChipGeneration::Bcm2836_7,
            ChipGeneration::Bcm2711,
            ChipGeneration::Bcm2712,
        ] {
            assert_eq!(ChipGeneration::from_pid(chip.pid()), Some(chip));
        }
        assert_eq!(ChipGeneration::from_pid(0x2763), None);
        assert_eq!(ChipGeneration::Bcm2712.directory_prefix(), "2712");
        assert_eq!(
            ChipGeneration::Bcm2711.description(),
            "Compute Module 4 (USB Boot)"
        );
        assert_eq!(ChipGeneration::Bcm2836_7.name(), "BCM2836/7");
    }

    #[test]
    fn parses_file_messages() {
        let mut raw = [0u8; FileMessage::SIZE];
        raw[..4].copy_from_slice(&1i32.to_le_bytes());
        raw[4..12].copy_from_slice(b"start.el");
        let m = FileMessage::parse(&raw);
        assert_eq!(
            (m.command, m.name.as_str()),
            (Some(FileCommand::ReadFile), "start.el")
        );
        raw[..4].copy_from_slice(&7i32.to_le_bytes());
        assert_eq!(FileMessage::parse(&raw).command, None);
        assert_eq!(boot_message(5, [0; 20])[..4], 5i32.to_le_bytes());
    }
}
