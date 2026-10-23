//! A DiskArbitration session and per-disk lookups.

use std::ffi::{CStr, CString};
use std::io;

use core_foundation::base::TCFType;
use core_foundation::url::CFURL;
use core_foundation_sys::base::CFRelease;

use super::cf::Description;
use super::ffi::*;

/// Owns a `DASessionRef`.
pub struct Session(DASessionRef);

impl Session {
    /// # Errors
    /// Fails when DiskArbitration refuses a session.
    pub fn new() -> io::Result<Self> {
        // SAFETY: a null allocator selects the default allocator.
        let session = unsafe { DASessionCreate(std::ptr::null()) };
        if session.is_null() {
            Err(io::Error::other(
                "Failed to enumerate drives: DiskArbitration session creation failed",
            ))
        } else {
            Ok(Self(session))
        }
    }

    pub fn raw(&self) -> DASessionRef {
        self.0
    }

    /// A disk object for `diskN` / `diskNsM`.
    pub fn disk(&self, bsd_name: &str) -> Option<Disk> {
        let name = CString::new(bsd_name).ok()?;
        // SAFETY: valid session and NUL-terminated name; we own the result.
        let disk = unsafe { DADiskCreateFromBSDName(std::ptr::null(), self.0, name.as_ptr()) };
        (!disk.is_null()).then_some(Disk(disk))
    }

    /// The disk (partition) mounted at `path`.
    pub fn disk_for_volume(&self, path: &str) -> Option<Disk> {
        let url = CFURL::from_path(path, true)?;
        // SAFETY: valid session and URL; we own the result.
        let disk = unsafe {
            DADiskCreateFromVolumePath(std::ptr::null(), self.0, url.as_concrete_TypeRef())
        };
        (!disk.is_null()).then_some(Disk(disk))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // SAFETY: we own the session reference.
        unsafe { CFRelease(self.0) };
    }
}

/// Owns a `DADiskRef`, released on drop.
pub struct Disk(DADiskRef);

impl Disk {
    pub fn as_raw(&self) -> DADiskRef {
        self.0
    }

    /// The disk's description dictionary.
    pub fn description(&self) -> Option<Description> {
        // SAFETY: valid disk; the copy follows the create rule.
        unsafe { Description::from_owned(DADiskCopyDescription(self.0)) }
    }

    /// `diskNsM`.
    pub fn bsd_name(&self) -> Option<String> {
        // SAFETY: valid disk; the name lives as long as the disk object.
        let name = unsafe { DADiskGetBSDName(self.0) };
        // SAFETY: non-null, NUL-terminated.
        (!name.is_null()).then(|| {
            unsafe { CStr::from_ptr(name) }
                .to_string_lossy()
                .into_owned()
        })
    }
}

impl Drop for Disk {
    fn drop(&mut self) {
        // SAFETY: we own the disk reference.
        unsafe { CFRelease(self.0) };
    }
}
