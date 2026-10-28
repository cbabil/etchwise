//! `DeviceIoControl` on files opened through std, and the handles the
//! disk code needs.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;

use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};
use windows_sys::Win32::System::IO::DeviceIoControl;

/// Opens a device for IOCTL queries only (no administrator needed).
pub fn query_handle(path: &str) -> io::Result<File> {
    OpenOptions::new()
        .access_mode(0)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(path)
}

/// Opens a device read-write, shared, as upstream does.
pub fn rw_handle(path: &str) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(path)
}

/// Sends `code`; returns how many bytes of `output` were filled.
pub fn ioctl(file: &File, code: u32, input: &[u8], output: &mut [u8]) -> io::Result<usize> {
    let mut returned = 0u32;
    let len = |n: usize| u32::try_from(n).map_err(io::Error::other);
    let (in_len, out_len) = (len(input.len())?, len(output.len())?);
    // SAFETY: the buffers are live for the call and their lengths are
    // passed with them; no OVERLAPPED, so the call is synchronous.
    let ok = unsafe {
        DeviceIoControl(
            file.as_raw_handle(),
            code,
            if input.is_empty() {
                std::ptr::null()
            } else {
                input.as_ptr().cast()
            },
            in_len,
            if output.is_empty() {
                std::ptr::null_mut()
            } else {
                output.as_mut_ptr().cast()
            },
            out_len,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(returned as usize)
}

/// An IOCTL without input or output (lock, dismount, eject...).
pub fn command(file: &File, code: u32) -> io::Result<()> {
    ioctl(file, code, &[], &mut []).map(|_| ())
}

/// Views plain-data `value` as bytes, for IOCTL input.
pub fn bytes_of<T: Copy>(value: &T) -> &[u8] {
    // SAFETY: T is plain old data (windows-sys repr(C) structs); the slice
    // covers exactly its bytes and borrows it.
    unsafe {
        std::slice::from_raw_parts(std::ptr::from_ref(value).cast(), std::mem::size_of::<T>())
    }
}

/// Reads one plain-data struct an IOCTL returns.
pub fn read_struct<T: Copy>(file: &File, code: u32) -> io::Result<T> {
    let mut out = vec![0u8; std::mem::size_of::<T>()];
    ioctl(file, code, &[], &mut out)?;
    // SAFETY: `out` holds size_of::<T>() bytes; T is plain data, valid for
    // any bit pattern; read_unaligned copes with the Vec's alignment.
    Ok(unsafe { std::ptr::read_unaligned(out.as_ptr().cast::<T>()) })
}
