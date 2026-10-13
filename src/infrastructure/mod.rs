//! Adaptadores de infraestructura que conectan los puertos con el sistema operativo.

#[cfg(windows)]
mod windows;
#[cfg(not(windows))]
mod stub;

use crate::application::ports::{HotkeyListener, ProcessFinder};

#[cfg(windows)]
pub fn default_process_finder() -> Box<dyn ProcessFinder> {
    Box::new(windows::WindowsProcessFinder)
}

#[cfg(not(windows))]
pub fn default_process_finder() -> Box<dyn ProcessFinder> {
    Box::new(stub::StubProcessFinder)
}

#[cfg(windows)]
pub fn default_hotkey_listener() -> Box<dyn HotkeyListener> {
    Box::new(windows::WindowsHotkeyListener::new())
}

#[cfg(not(windows))]
pub fn default_hotkey_listener() -> Box<dyn HotkeyListener> {
    Box::new(stub::StubHotkeyListener::new())
}
