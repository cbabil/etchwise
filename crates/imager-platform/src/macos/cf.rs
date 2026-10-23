//! Typed reads from DiskArbitration description dictionaries.

use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_foundation_sys::dictionary::CFDictionaryRef;
use core_foundation_sys::string::CFStringRef;

/// Wraps a framework key constant (`kDADiskDescription…Key`).
///
/// # Safety
/// `key` must be a valid `CFStringRef`, such as an extern framework static.
pub unsafe fn key(key: CFStringRef) -> CFString {
    // SAFETY: guaranteed by the caller; the get rule retains it.
    unsafe { CFString::wrap_under_get_rule(key) }
}

/// An owned description dictionary.
pub struct Description(CFDictionary<CFString, CFType>);

impl Description {
    /// Takes ownership of a dictionary returned by a `Copy`/`Create` call.
    ///
    /// # Safety
    /// `dict` must be a valid, owned `CFDictionaryRef` or null.
    pub unsafe fn from_owned(dict: CFDictionaryRef) -> Option<Self> {
        // SAFETY: caller passes an owned dictionary (create rule).
        (!dict.is_null()).then(|| Self(unsafe { CFDictionary::wrap_under_create_rule(dict) }))
    }

    fn get(&self, key: &CFString) -> Option<CFType> {
        self.0.find(key).map(|v| v.clone())
    }

    pub fn string(&self, key: &CFString) -> Option<String> {
        self.get(key)?.downcast::<CFString>().map(|s| s.to_string())
    }

    pub fn u64(&self, key: &CFString) -> Option<u64> {
        self.get(key)?
            .downcast::<CFNumber>()?
            .to_i64()
            .and_then(|n| u64::try_from(n).ok())
    }

    pub fn bool(&self, key: &CFString) -> bool {
        self.get(key)
            .and_then(|v| v.downcast::<CFBoolean>())
            .is_some_and(bool::from)
    }

    /// `IOBundleResourceFile` inside the media icon dictionary.
    pub fn icon_file(&self, icon_key: &CFString) -> Option<String> {
        let icon = self.get(icon_key)?.downcast::<CFDictionary>()?;
        let name = CFString::from_static_string("IOBundleResourceFile");
        let value = icon.find(name.as_concrete_TypeRef().cast())?;
        // SAFETY: values of an icon dictionary are CF objects (get rule).
        let value = unsafe { CFType::wrap_under_get_rule(*value) };
        value.downcast::<CFString>().map(|s| s.to_string())
    }
}
