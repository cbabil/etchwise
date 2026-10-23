//! Fastboot flashing (upstream `src/fastboot/` and
//! `fastbootflashthread.cpp`): once rpiboot has sideloaded the fastboot
//! gadget, the image is streamed to the device as self-positioning
//! Android sparse segments, verified by hash, customised through the
//! gadget's file protocol and the device rebooted.
//!
//! - [`protocol`]: the fastboot wire protocol over [`crate::transport::UsbTransport`]
//! - [`sparse`] / [`SparseEncoder`]: the sparse format and streaming encoder
//! - [`bmap`]: `.bmap` block maps for DONT_CARE skipping
//! - [`flash_over_fastboot`]: the full flash flow, over the transport
//!   opened by [`crate::rpiboot::LibusbContext::open_at`]

pub mod bmap;
mod commands;
pub mod customise;
mod encoder;
pub mod flash;
pub mod protocol;
pub mod sparse;
mod stream;
mod transfer;

pub use bmap::{BlockMap, BlockRange, BmapError};
pub use encoder::{MIN_SEGMENT_SIZE, SparseEncoder};
pub use flash::{FlashError, FlashOptions, flash_over_fastboot};
pub use protocol::{FastbootError, Response, ResponseType};
pub use sparse::{Segment, SegmentStats};

#[cfg(test)]
mod tests_bmap;
#[cfg(test)]
mod tests_customise;
#[cfg(test)]
mod tests_flash;
#[cfg(test)]
mod tests_flash_errors;
#[cfg(test)]
mod tests_protocol;
#[cfg(test)]
mod tests_protocol_data;
#[cfg(test)]
mod tests_protocol_files;
#[cfg(test)]
mod tests_sparse;
#[cfg(test)]
mod tests_sparse_split;
#[cfg(test)]
mod tests_support;
