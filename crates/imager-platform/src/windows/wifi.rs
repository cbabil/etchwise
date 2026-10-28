//! Wi-Fi prefill through the WLAN API (upstream `winwlancredentials.cpp`):
//! the SSID of the connected interface and, from its profile with the
//! plaintext-key flag (administrator only), the saved key.

use std::ffi::c_void;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::NetworkManagement::WiFi::{
    WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST,
    WLAN_PROFILE_GET_PLAINTEXT_KEY, WlanCloseHandle, WlanEnumInterfaces, WlanFreeMemory,
    WlanGetProfile, WlanOpenHandle, WlanQueryInterface, wlan_interface_state_connected,
    wlan_intf_opcode_current_connection,
};
use windows_sys::core::GUID;

use super::wlan_xml::key_material;

/// SSID of the Wi-Fi network this computer is on, if any.
pub fn current_ssid() -> Option<String> {
    Client::open()?.current().map(|c| c.ssid)
}

/// The saved key of `ssid` when it is the connected network.
pub fn saved_password(ssid: &str) -> Option<String> {
    let client = Client::open()?;
    let current = client.current().filter(|c| c.ssid == ssid)?;
    client.profile_key(&current)
}

struct Connected {
    interface: GUID,
    profile: Vec<u16>,
    ssid: String,
}

/// A WLAN client handle, closed when dropped.
struct Client(HANDLE);

impl Client {
    fn open() -> Option<Self> {
        let (mut version, mut handle) = (0u32, std::ptr::null_mut());
        // SAFETY: out-parameters; client version 2 is Vista and later.
        (unsafe { WlanOpenHandle(2, std::ptr::null(), &mut version, &mut handle) } == 0)
            .then_some(Self(handle))
    }

    fn current(&self) -> Option<Connected> {
        let mut list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        // SAFETY: valid handle; the list is freed below.
        if unsafe { WlanEnumInterfaces(self.0, std::ptr::null(), &mut list) } != 0 {
            return None;
        }
        let _free = Freed(list.cast());
        // SAFETY: WlanEnumInterfaces filled a list of dwNumberOfItems
        // entries. They run past the declared one-element array, so the
        // pointer comes from the list allocation, not from that array.
        let interfaces: &[WLAN_INTERFACE_INFO] = unsafe {
            std::slice::from_raw_parts(
                (&raw const (*list).InterfaceInfo).cast::<WLAN_INTERFACE_INFO>(),
                (*list).dwNumberOfItems as usize,
            )
        };
        interfaces
            .iter()
            .filter(|i| i.isState == wlan_interface_state_connected)
            .find_map(|i| self.connection(&i.InterfaceGuid))
    }

    fn connection(&self, interface: &GUID) -> Option<Connected> {
        let (mut size, mut data) = (0u32, std::ptr::null_mut::<c_void>());
        // SAFETY: valid handle and GUID; data is freed below.
        let status = unsafe {
            WlanQueryInterface(
                self.0,
                interface,
                wlan_intf_opcode_current_connection,
                std::ptr::null(),
                &mut size,
                &mut data,
                std::ptr::null_mut(),
            )
        };
        if status != 0 || (size as usize) < std::mem::size_of::<WLAN_CONNECTION_ATTRIBUTES>() {
            return None;
        }
        let _free = Freed(data);
        // SAFETY: the opcode returns a WLAN_CONNECTION_ATTRIBUTES of `size` bytes.
        let attributes = unsafe { &*data.cast::<WLAN_CONNECTION_ATTRIBUTES>() };
        let ssid = &attributes.wlanAssociationAttributes.dot11Ssid;
        let len = (ssid.uSSIDLength as usize).min(ssid.ucSSID.len());
        let name = std::str::from_utf8(&ssid.ucSSID[..len])
            .ok()
            .filter(|s| !s.is_empty())?
            .to_owned();
        let profile_len = attributes.strProfileName.iter().position(|c| *c == 0)?;
        Some(Connected {
            interface: *interface,
            profile: attributes.strProfileName[..=profile_len].to_vec(),
            ssid: name,
        })
    }

    fn profile_key(&self, connected: &Connected) -> Option<String> {
        let mut xml: *mut u16 = std::ptr::null_mut();
        let mut flags = WLAN_PROFILE_GET_PLAINTEXT_KEY;
        // SAFETY: NUL-terminated profile name; xml is freed below.
        let status = unsafe {
            WlanGetProfile(
                self.0,
                &connected.interface,
                connected.profile.as_ptr(),
                std::ptr::null(),
                &mut xml,
                &mut flags,
                std::ptr::null_mut(),
            )
        };
        if status != 0 || xml.is_null() {
            return None;
        }
        let _free = Freed(xml.cast());
        // SAFETY: WlanGetProfile returns a NUL-terminated UTF-16 string.
        let text = unsafe {
            let len = (0..).take_while(|&i| *xml.add(i) != 0).count();
            String::from_utf16_lossy(std::slice::from_raw_parts(xml, len))
        };
        key_material(&text)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // SAFETY: the handle from WlanOpenHandle.
        unsafe { WlanCloseHandle(self.0, std::ptr::null()) };
    }
}

/// Memory the WLAN API allocated, freed when dropped.
struct Freed(*mut c_void);

impl Drop for Freed {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: allocated by the WLAN API.
            unsafe { WlanFreeMemory(self.0) };
        }
    }
}
