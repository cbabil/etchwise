//! Location permission, which macOS requires before CoreWLAN reveals the
//! SSID (upstream `mac/location_helper.mm`). The request must come from the
//! main thread; the manager is kept alive so the prompt can be answered.

use std::cell::RefCell;

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_core_location::{CLAuthorizationStatus, CLLocationManager};

thread_local! {
    static MANAGER: RefCell<Option<Retained<CLLocationManager>>> = const { RefCell::new(None) };
}

/// Location access state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationAccess {
    Granted,
    Denied,
    NotDetermined,
}

fn manager(_main: MainThreadMarker) -> Retained<CLLocationManager> {
    MANAGER.with(|slot| {
        // SAFETY: CLLocationManager is created and used on the main thread.
        slot.borrow_mut()
            .get_or_insert_with(|| unsafe { CLLocationManager::new() })
            .clone()
    })
}

/// Current authorization.
pub fn location_access(main: MainThreadMarker) -> LocationAccess {
    // SAFETY: main-thread manager.
    match unsafe { manager(main).authorizationStatus() } {
        CLAuthorizationStatus::AuthorizedAlways | CLAuthorizationStatus::AuthorizedWhenInUse => {
            LocationAccess::Granted
        }
        CLAuthorizationStatus::NotDetermined => LocationAccess::NotDetermined,
        _ => LocationAccess::Denied,
    }
}

/// Shows the system prompt when the user has not decided yet.
pub fn request_location_access(main: MainThreadMarker) {
    if location_access(main) == LocationAccess::NotDetermined {
        // SAFETY: main-thread manager.
        unsafe { manager(main).requestWhenInUseAuthorization() };
    }
}
