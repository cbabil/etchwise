//! Runs `lsblk` (5 s limit, as upstream) and parses its JSON.

use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use imager_core::drives::DeviceDescriptor;

use super::lsblk;

const TIMEOUT: Duration = Duration::from_secs(5);

/// Every whole disk with its mount points.
///
/// # Errors
/// Fails when `lsblk` cannot run, times out or writes something unreadable.
pub fn list_drives() -> io::Result<Vec<DeviceDescriptor>> {
    lsblk::parse(&run_lsblk()?, mmc_type)
}

fn run_lsblk() -> io::Result<String> {
    let mut child = Command::new("lsblk")
        .args(lsblk::LSBLK_ARGS)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("no lsblk output"))?;
    let reader = std::thread::spawn(move || {
        let mut out = String::new();
        stdout.read_to_string(&mut out).map(|_| out)
    });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(io::ErrorKind::TimedOut, "lsblk timed out"));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let out = reader
        .join()
        .map_err(|_| io::Error::other("lsblk reader failed"))??;
    if !status.success() {
        return Err(io::Error::other(format!("lsblk failed ({status})")));
    }
    Ok(out)
}

/// The kernel's card type for an MMC disk (`SD`, `MMC`), or empty.
fn mmc_type(kname: &str) -> String {
    std::fs::read_to_string(format!("/sys/block/{kname}/device/type"))
        .map(|t| t.trim().to_owned())
        .unwrap_or_default()
}
