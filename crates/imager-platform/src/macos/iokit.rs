//! IOKit registry walk from a synthesized APFS container to the physical
//! disk that holds it (upstream `findAPFSParent`).

use std::ffi::{CString, c_char};

use core_foundation::base::{CFType, TCFType};
use core_foundation::string::CFString;

use super::ffi::*;

const SERVICE_PLANE: &[u8] = b"IOService\0";
const IO_MEDIA: &[u8] = b"IOMedia\0";

/// Owns an IOKit object reference.
struct Object(io_object_t);

impl Drop for Object {
    fn drop(&mut self) {
        if self.0 != 0 {
            // SAFETY: we own this reference.
            unsafe { IOObjectRelease(self.0) };
        }
    }
}

/// Three levels up the service plane, then the first `IOMedia` parent's
/// BSD name.
pub fn apfs_parent(bsd_name: &str) -> Option<String> {
    let name = CString::new(bsd_name).ok()?;
    // SAFETY: IOServiceGetMatchingService consumes the matching dictionary.
    let mut current = Object(unsafe {
        IOServiceGetMatchingService(
            kIOMainPortDefault,
            IOBSDNameMatching(kIOMainPortDefault, 0, name.as_ptr()).cast(),
        )
    });
    if current.0 == 0 {
        return None;
    }
    for _ in 0..3 {
        let mut parent = 0;
        // SAFETY: valid entry; parent is an out-parameter we then own.
        if unsafe { IORegistryEntryGetParentEntry(current.0, plane(), &mut parent) } != 0 {
            return None;
        }
        current = Object(parent);
    }
    let mut iterator = 0;
    // SAFETY: valid entry; iterator is an out-parameter we then own.
    if unsafe { IORegistryEntryGetParentIterator(current.0, plane(), &mut iterator) } != 0 {
        return None;
    }
    let iterator = Object(iterator);
    loop {
        // SAFETY: valid iterator; each result is owned.
        let parent = Object(unsafe { IOIteratorNext(iterator.0) });
        if parent.0 == 0 {
            return None;
        }
        // SAFETY: valid object and NUL-terminated class name.
        if unsafe { IOObjectConformsTo(parent.0, IO_MEDIA.as_ptr().cast()) } != 0
            && let Some(name) = bsd_name_property(parent.0)
        {
            return Some(name);
        }
    }
}

fn plane() -> *const c_char {
    SERVICE_PLANE.as_ptr().cast()
}

fn bsd_name_property(object: io_object_t) -> Option<String> {
    let key = CFString::from_static_string("BSD Name");
    // SAFETY: valid object and key; the property follows the create rule.
    let value = unsafe {
        IORegistryEntryCreateCFProperty(object, key.as_concrete_TypeRef(), std::ptr::null(), 0)
    };
    if value.is_null() {
        return None;
    }
    // SAFETY: owned CF object from the call above.
    let value = unsafe { CFType::wrap_under_create_rule(value) };
    value.downcast::<CFString>().map(|s| s.to_string())
}
