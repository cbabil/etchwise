//! DiskArbitration, IOKit and IOPM declarations the platform layer uses.
#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    clippy::upper_case_acronyms
)]

use std::ffi::{c_char, c_void};

use core_foundation_sys::base::{CFAllocatorRef, CFTypeRef};
use core_foundation_sys::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};
use core_foundation_sys::runloop::CFRunLoopRef;
use core_foundation_sys::string::CFStringRef;
use core_foundation_sys::url::CFURLRef;

pub type DASessionRef = *const c_void;
pub type DADiskRef = *const c_void;
pub type DADissenterRef = *const c_void;
pub type DAReturn = i32;
pub type DADiskCallback = extern "C" fn(DADiskRef, DADissenterRef, *mut c_void);

pub const kDADiskUnmountOptionForce: u32 = 0x0008_0000;
pub const kDADiskUnmountOptionWhole: u32 = 0x0000_0001;
pub const kDADiskEjectOptionDefault: u32 = 0;

#[link(name = "DiskArbitration", kind = "framework")]
unsafe extern "C" {
    pub static kDADiskDescriptionDeviceProtocolKey: CFStringRef;
    pub static kDADiskDescriptionMediaBlockSizeKey: CFStringRef;
    pub static kDADiskDescriptionDeviceInternalKey: CFStringRef;
    pub static kDADiskDescriptionMediaRemovableKey: CFStringRef;
    pub static kDADiskDescriptionMediaEjectableKey: CFStringRef;
    pub static kDADiskDescriptionMediaWritableKey: CFStringRef;
    pub static kDADiskDescriptionMediaNameKey: CFStringRef;
    pub static kDADiskDescriptionMediaSizeKey: CFStringRef;
    pub static kDADiskDescriptionMediaIconKey: CFStringRef;
    pub static kDADiskDescriptionVolumeNameKey: CFStringRef;

    pub fn DASessionCreate(allocator: CFAllocatorRef) -> DASessionRef;
    pub fn DASessionScheduleWithRunLoop(
        session: DASessionRef,
        run_loop: CFRunLoopRef,
        mode: CFStringRef,
    );
    pub fn DASessionUnscheduleFromRunLoop(
        session: DASessionRef,
        run_loop: CFRunLoopRef,
        mode: CFStringRef,
    );
    pub fn DADiskCreateFromBSDName(
        allocator: CFAllocatorRef,
        session: DASessionRef,
        name: *const c_char,
    ) -> DADiskRef;
    pub fn DADiskCreateFromVolumePath(
        allocator: CFAllocatorRef,
        session: DASessionRef,
        path: CFURLRef,
    ) -> DADiskRef;
    pub fn DADiskCopyDescription(disk: DADiskRef) -> CFDictionaryRef;
    pub fn DADiskGetBSDName(disk: DADiskRef) -> *const c_char;
    pub fn DADiskUnmount(
        disk: DADiskRef,
        options: u32,
        callback: Option<DADiskCallback>,
        context: *mut c_void,
    );
    pub fn DADiskEject(
        disk: DADiskRef,
        options: u32,
        callback: Option<DADiskCallback>,
        context: *mut c_void,
    );
    pub fn DADissenterGetStatus(dissenter: DADissenterRef) -> DAReturn;
}

pub type io_object_t = u32;
pub type io_service_t = io_object_t;
pub type io_iterator_t = io_object_t;
pub type kern_return_t = i32;
pub type IOPMAssertionID = u32;
pub const kIOMainPortDefault: u32 = 0;
pub const kIOPMAssertionLevelOn: u32 = 255;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    pub fn IOBSDNameMatching(
        main_port: u32,
        options: u32,
        bsd_name: *const c_char,
    ) -> CFMutableDictionaryRef;
    pub fn IOServiceGetMatchingService(main_port: u32, matching: CFDictionaryRef) -> io_service_t;
    pub fn IORegistryEntryGetParentEntry(
        entry: io_object_t,
        plane: *const c_char,
        parent: *mut io_object_t,
    ) -> kern_return_t;
    pub fn IORegistryEntryGetParentIterator(
        entry: io_object_t,
        plane: *const c_char,
        iterator: *mut io_iterator_t,
    ) -> kern_return_t;
    pub fn IORegistryEntryCreateCFProperty(
        entry: io_object_t,
        key: CFStringRef,
        allocator: CFAllocatorRef,
        options: u32,
    ) -> CFTypeRef;
    pub fn IOObjectConformsTo(object: io_object_t, class_name: *const c_char) -> u32;
    pub fn IOIteratorNext(iterator: io_iterator_t) -> io_object_t;
    pub fn IOObjectRelease(object: io_object_t) -> kern_return_t;
    pub fn IOPMAssertionCreateWithName(
        kind: CFStringRef,
        level: u32,
        name: CFStringRef,
        id: *mut IOPMAssertionID,
    ) -> kern_return_t;
    pub fn IOPMAssertionRelease(id: IOPMAssertionID) -> kern_return_t;
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {
    pub fn NSBeep();
}
