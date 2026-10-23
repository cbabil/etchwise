//! Writing an image: decompression, the write loop, verification and
//! customisation, ported from upstream `DownloadThread` and friends.

pub mod buffers;
pub mod cache;
pub mod decode;
pub mod device;
pub mod error;
mod feed;
pub mod finalize;
pub mod http;
pub mod job;
pub mod multifile;
pub mod progress;
pub mod source_size;
pub mod writer;
mod zipstream;

pub use device::Device;
pub use error::WriteError;
pub use feed::open_source;
pub use progress::{Progress, ProgressSnapshot, WriteState};
pub use writer::{WriteOptions, Written};
