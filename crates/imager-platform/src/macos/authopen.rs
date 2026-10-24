//! Raw disk access through `/usr/libexec/authopen` (upstream
//! `MacFile::authOpen`): an Authorization right for
//! `sys.openfile.readwrite.<path>` with a custom prompt, its external form
//! sent to authopen, and the opened descriptor received over a Unix socket
//! (`SCM_RIGHTS`). No helper tool or entitlement is needed.

use std::fs::File;
use std::io::{self, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};

use security_framework::authorization::{Authorization, AuthorizationItemSetBuilder, Flags};

/// Upstream's dialog text (`MacFile` translation context).
pub const PROMPT: &str = "Etchwise needs to access the disk to write the image.";

/// Why the device could not be opened.
#[derive(Debug, thiserror::Error)]
pub enum AuthOpenError {
    #[error("authorization was cancelled")]
    Cancelled,
    #[error("authopen failed: {0}")]
    Failed(String),
}

/// Opens `path` read-write after the user authorizes it.
///
/// # Errors
/// `Cancelled` when the user declines, `Failed` otherwise.
pub fn auth_open(path: &str, prompt: &str) -> Result<File, AuthOpenError> {
    let rights = AuthorizationItemSetBuilder::new()
        .add_right(format!("sys.openfile.readwrite.{path}"))
        .map_err(failed)?
        .build();
    let env = AuthorizationItemSetBuilder::new()
        .add_string("prompt", prompt)
        .map_err(failed)?
        .build();
    let flags = Flags::INTERACTION_ALLOWED | Flags::EXTEND_RIGHTS | Flags::PREAUTHORIZE;
    let auth =
        Authorization::new(Some(rights), Some(env), flags).map_err(|_| AuthOpenError::Cancelled)?;
    let external = auth.make_external_form().map_err(failed)?;
    let (ours, theirs) = UnixStream::pair().map_err(failed)?;
    let mut child = Command::new("/usr/libexec/authopen")
        .args([
            "-stdoutpipe",
            "-extauth",
            "-o",
            &libc::O_RDWR.to_string(),
            path,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::from(OwnedFd::from(theirs)))
        .spawn()
        .map_err(failed)?;
    let bytes: Vec<u8> = external.bytes.iter().map(|b| *b as u8).collect();
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&bytes).map_err(failed)?;
    }
    let fd = receive_fd(&ours);
    let status = child.wait().map_err(failed)?;
    match (fd, status.success()) {
        (Ok(fd), true) => Ok(File::from(fd)),
        (Err(e), _) => Err(failed(e)),
        (_, false) => Err(AuthOpenError::Failed(format!(
            "authopen exited with {status}"
        ))),
    }
}

fn failed(e: impl std::fmt::Display) -> AuthOpenError {
    AuthOpenError::Failed(e.to_string())
}

/// Receives one descriptor sent with `SCM_RIGHTS`.
fn receive_fd(socket: &UnixStream) -> io::Result<OwnedFd> {
    let mut data = [0u8; 64];
    let mut iov = libc::iovec {
        iov_base: data.as_mut_ptr().cast(),
        iov_len: data.len(),
    };
    // SAFETY: CMSG_SPACE is a pure size computation.
    let space = unsafe { libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) } as usize;
    let mut control = vec![0u8; space];
    // SAFETY: zeroed msghdr is a valid initial value.
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = control.as_mut_ptr().cast();
    msg.msg_controllen = space as libc::socklen_t;
    let received = loop {
        // SAFETY: msg points at live buffers for the duration of the call.
        let n = unsafe { libc::recvmsg(socket.as_raw_fd(), &mut msg, 0) };
        if n >= 0 || io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            break n;
        }
    };
    if received <= 0 {
        return Err(io::Error::other("authopen sent no descriptor"));
    }
    // SAFETY: msg was filled by recvmsg; CMSG_FIRSTHDR handles a short buffer.
    let header = unsafe { libc::CMSG_FIRSTHDR(&msg) };
    // SAFETY: header (when non-null) points into `control`.
    if header.is_null() || unsafe { (*header).cmsg_type } != libc::SCM_RIGHTS {
        return Err(io::Error::other("authopen reply had no SCM_RIGHTS"));
    }
    // SAFETY: an SCM_RIGHTS message carries a c_int descriptor we now own.
    Ok(unsafe {
        OwnedFd::from_raw_fd(std::ptr::read_unaligned(
            libc::CMSG_DATA(header).cast::<libc::c_int>(),
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receives_a_descriptor_over_a_socket() {
        let (a, b) = UnixStream::pair().unwrap();
        let file = tempfile::tempfile().unwrap();
        let fd = file.as_raw_fd();
        let mut payload = [0u8; 1];
        let mut iov = libc::iovec {
            iov_base: payload.as_mut_ptr().cast(),
            iov_len: 1,
        };
        // SAFETY: test-only sendmsg mirroring authopen's reply.
        unsafe {
            let space = libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) as usize;
            let mut control = vec![0u8; space];
            let mut msg: libc::msghdr = std::mem::zeroed();
            msg.msg_iov = &mut iov;
            msg.msg_iovlen = 1;
            msg.msg_control = control.as_mut_ptr().cast();
            msg.msg_controllen = space as libc::socklen_t;
            let header = libc::CMSG_FIRSTHDR(&msg);
            (*header).cmsg_level = libc::SOL_SOCKET;
            (*header).cmsg_type = libc::SCM_RIGHTS;
            (*header).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<libc::c_int>() as u32) as _;
            std::ptr::write_unaligned(libc::CMSG_DATA(header).cast::<libc::c_int>(), fd);
            assert!(libc::sendmsg(a.as_raw_fd(), &msg, 0) > 0);
        }
        let received = receive_fd(&b).unwrap();
        assert!(received.as_raw_fd() >= 0);
        drop(a);
        assert!(receive_fd(&b).is_err());
    }
}
