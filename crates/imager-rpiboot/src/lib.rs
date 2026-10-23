//! USB boot of Raspberry Pi Compute Modules and boards in rpiboot mode
//! (upstream `src/rpiboot/`, `src/fastboot/`, `rpibootthread.cpp`,
//! `fastbootflashthread.cpp`): load the fastboot gadget over the Broadcom
//! USB boot protocol, then flash the image as Android sparse chunks.

pub mod fastboot;
pub mod rpiboot;
pub mod transport;
pub mod types;
