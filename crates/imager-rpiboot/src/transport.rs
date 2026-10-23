//! The USB operations the protocols need (upstream `IUsbTransport`); the
//! real implementation wraps libusb, tests use [`MockTransport`].

/// One opened USB device.
pub trait UsbTransport: Send {
    /// Vendor control transfer OUT; true on success.
    fn control_out(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
        timeout_ms: u32,
    ) -> bool;
    /// Vendor control transfer IN; bytes read or `None` on error.
    fn control_in(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        buffer: &mut [u8],
        timeout_ms: u32,
    ) -> Option<usize>;
    /// Bulk OUT; bytes written or `None` on error.
    fn bulk_write(&mut self, endpoint: u8, data: &[u8], timeout_ms: u32) -> Option<usize>;
    /// Bulk IN; bytes read or `None` on error.
    fn bulk_read(&mut self, endpoint: u8, buffer: &mut [u8], timeout_ms: u32) -> Option<usize>;
    /// The device handle is still usable.
    fn is_open(&self) -> bool;
    /// Bulk OUT endpoint (from the configuration descriptor, default 0x01).
    fn out_endpoint(&self) -> u8 {
        0x01
    }
    /// libusb error code of the most recent failed transfer (e.g. -4 for
    /// `LIBUSB_ERROR_NO_DEVICE`), when the backend knows it. Upstream's
    /// transfers return these codes directly; `None` means unknown.
    fn last_error_code(&self) -> Option<i32> {
        None
    }
}

/// One recorded USB operation, for tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsbCall {
    ControlOut {
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: Vec<u8>,
    },
    ControlIn {
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        len: usize,
    },
    BulkWrite {
        endpoint: u8,
        data: Vec<u8>,
    },
    BulkRead {
        endpoint: u8,
        len: usize,
    },
}

/// Scripted transport: records every call and answers IN transfers from
/// queues (`None` entries simulate errors). Upstream's tests use the same
/// pattern with a mock `IUsbTransport`.
#[derive(Debug, Default)]
pub struct MockTransport {
    pub calls: Vec<UsbCall>,
    pub control_in_replies: std::collections::VecDeque<Option<Vec<u8>>>,
    pub bulk_read_replies: std::collections::VecDeque<Option<Vec<u8>>>,
    /// Bulk writes fail once this many have succeeded (`None`: never).
    pub fail_bulk_write_after: Option<usize>,
    pub control_out_ok: bool,
    pub closed: bool,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            control_out_ok: true,
            ..Self::default()
        }
    }

    fn take(
        queue: &mut std::collections::VecDeque<Option<Vec<u8>>>,
        buffer: &mut [u8],
    ) -> Option<usize> {
        let reply = queue.pop_front().flatten()?;
        let n = reply.len().min(buffer.len());
        buffer[..n].copy_from_slice(&reply[..n]);
        Some(n)
    }
}

impl UsbTransport for MockTransport {
    fn control_out(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
        _timeout_ms: u32,
    ) -> bool {
        self.calls.push(UsbCall::ControlOut {
            request_type,
            request,
            value,
            index,
            data: data.to_vec(),
        });
        self.control_out_ok
    }

    fn control_in(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        buffer: &mut [u8],
        _timeout_ms: u32,
    ) -> Option<usize> {
        self.calls.push(UsbCall::ControlIn {
            request_type,
            request,
            value,
            index,
            len: buffer.len(),
        });
        Self::take(&mut self.control_in_replies, buffer)
    }

    fn bulk_write(&mut self, endpoint: u8, data: &[u8], _timeout_ms: u32) -> Option<usize> {
        let done = self
            .calls
            .iter()
            .filter(|c| matches!(c, UsbCall::BulkWrite { .. }))
            .count();
        self.calls.push(UsbCall::BulkWrite {
            endpoint,
            data: data.to_vec(),
        });
        if self
            .fail_bulk_write_after
            .is_some_and(|limit| done >= limit)
        {
            None
        } else {
            Some(data.len())
        }
    }

    fn bulk_read(&mut self, endpoint: u8, buffer: &mut [u8], _timeout_ms: u32) -> Option<usize> {
        self.calls.push(UsbCall::BulkRead {
            endpoint,
            len: buffer.len(),
        });
        Self::take(&mut self.bulk_read_replies, buffer)
    }

    fn is_open(&self) -> bool {
        !self.closed
    }
}
