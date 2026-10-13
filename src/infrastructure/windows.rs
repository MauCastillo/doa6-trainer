//! Implementación real sobre la API Win32 de Windows.

use std::collections::HashSet;
use std::ffi::c_void;

use windows::Win32::Foundation::{CloseHandle, HANDLE, STILL_ACTIVE};
use windows::Win32::System::Diagnostics::Debug::{FlushInstructionCache, ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Process32FirstW, Process32NextW, MODULEENTRY32W, PROCESSENTRY32W,
    TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualProtectEx, VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_FREE,
    MEM_RESERVE, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY,
    PAGE_GUARD, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
use windows::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_ALL_ACCESS};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VIRTUAL_KEY, VK_CONTROL, VK_HOME, VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3,
    VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9, VK_SHIFT,
};

use crate::application::ports::{
    GameMemory, GameProcess, HotkeyEvent, HotkeyListener, MemoryError, MemoryRegion, ProcessFinder,
};
use crate::domain::cheat::Hotkey;

pub struct WindowsProcessFinder;

impl ProcessFinder for WindowsProcessFinder {
    fn find(&self, process_name: &str) -> Result<Option<GameProcess>, MemoryError> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|e| MemoryError {
                operation: "CreateToolhelp32Snapshot",
                address: 0,
                detail: e.to_string(),
            })?;

            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                    let name = String::from_utf16_lossy(&entry.szExeFile[..len]);

                    let matches = name.eq_ignore_ascii_case(process_name)
                        || (process_name.eq_ignore_ascii_case("DOA6.exe")
                            && name.eq_ignore_ascii_case("DOA6LR.exe"));
                    if matches {
                        let _ = CloseHandle(snapshot);
                        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, entry.th32ProcessID).map_err(|e| {
                            MemoryError {
                                operation: "OpenProcess",
                                address: 0,
                                detail: format!("PID {}: {}", entry.th32ProcessID, e),
                            }
                        })?;

                        return Ok(Some(GameProcess {
                            pid: entry.th32ProcessID,
                            memory: Box::new(WindowsGameMemory::new(handle, entry.th32ProcessID)?),
                        }));
                    }

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
            Ok(None)
        }
    }
}

pub struct WindowsGameMemory {
    handle: HANDLE,
    /// Imagen del ejecutable del juego: las firmas solo se buscan aquí, como el original.
    main_module: MemoryRegion,
}

impl WindowsGameMemory {
    pub fn new(handle: HANDLE, pid: u32) -> Result<Self, MemoryError> {
        match main_module(pid) {
            Some(main_module) => Ok(Self { handle, main_module }),
            None => {
                unsafe {
                    let _ = CloseHandle(handle);
                }
                Err(MemoryError {
                    operation: "Module32FirstW",
                    address: 0,
                    detail: format!("no se pudo leer el módulo principal del PID {pid}"),
                })
            }
        }
    }
}

fn main_module(pid: u32) -> Option<MemoryRegion> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid).ok()?;
        let mut entry = MODULEENTRY32W { dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32, ..Default::default() };
        let found = Module32FirstW(snapshot, &mut entry).is_ok();
        let _ = CloseHandle(snapshot);
        found.then(|| MemoryRegion { base: entry.modBaseAddr as u64, size: entry.modBaseSize as usize })
    }
}

impl Drop for WindowsGameMemory {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

unsafe impl Send for WindowsGameMemory {}
unsafe impl Sync for WindowsGameMemory {}

impl GameMemory for WindowsGameMemory {
    fn read(&self, address: u64, buffer: &mut [u8]) -> Result<(), MemoryError> {
        unsafe {
            let mut bytes_read = 0;
            ReadProcessMemory(
                self.handle,
                address as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
                Some(&mut bytes_read),
            )
            .map_err(|e| MemoryError {
                operation: "ReadProcessMemory",
                address,
                detail: e.to_string(),
            })?;
            Ok(())
        }
    }

    fn write(&self, address: u64, bytes: &[u8]) -> Result<(), MemoryError> {
        unsafe {
            let mut old_protect = PAGE_PROTECTION_FLAGS(0);
            VirtualProtectEx(
                self.handle,
                address as *const c_void,
                bytes.len(),
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
            .map_err(|e| MemoryError {
                operation: "VirtualProtectEx (desbloquear)",
                address,
                detail: e.to_string(),
            })?;

            let mut bytes_written = 0;
            let write_result = WriteProcessMemory(
                self.handle,
                address as *const c_void,
                bytes.as_ptr() as *const c_void,
                bytes.len(),
                Some(&mut bytes_written),
            );

            let mut temp = PAGE_PROTECTION_FLAGS(0);
            let _ = VirtualProtectEx(
                self.handle,
                address as *const c_void,
                bytes.len(),
                old_protect,
                &mut temp,
            );

            let _ = FlushInstructionCache(self.handle, Some(address as *const c_void), bytes.len());

            write_result.map_err(|e| MemoryError {
                operation: "WriteProcessMemory",
                address,
                detail: e.to_string(),
            })?;

            Ok(())
        }
    }

    fn allocate_near(&self, address: u64, size: usize) -> Result<u64, MemoryError> {
        unsafe {
            let mut sys_info = SYSTEM_INFO::default();
            GetSystemInfo(&mut sys_info);
            // Las reservas empiezan en múltiplos de la granularidad (64 KiB), no de página:
            // una dirección desalineada se redondea hacia abajo y choca con reservas previas.
            let granularity = u64::from(sys_info.dwAllocationGranularity);

            // Busca memoria libre en un rango de +/- 2 GB
            let min_addr = address.saturating_sub(0x7000_0000).max(sys_info.lpMinimumApplicationAddress as u64);
            let max_addr = (address + 0x7000_0000).min(sys_info.lpMaximumApplicationAddress as u64);

            let mut current = (address.saturating_sub(0x1000_0000)).max(min_addr);
            let mut mbi = MEMORY_BASIC_INFORMATION::default();

            while current < max_addr {
                if VirtualQueryEx(
                    self.handle,
                    Some(current as *const c_void),
                    &mut mbi,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                ) == 0
                {
                    break;
                }

                let region_end = mbi.BaseAddress as u64 + mbi.RegionSize as u64;
                let candidate = current.next_multiple_of(granularity);
                if mbi.State == MEM_FREE && candidate + size as u64 <= region_end {
                    let allocated = VirtualAllocEx(
                        self.handle,
                        Some(candidate as *const c_void),
                        size,
                        MEM_COMMIT | MEM_RESERVE,
                        PAGE_EXECUTE_READWRITE,
                    );
                    if !allocated.is_null() {
                        return Ok(allocated as u64);
                    }
                }

                current = region_end.max(current + granularity);
            }

            Err(MemoryError {
                operation: "VirtualAllocEx",
                address,
                detail: "no se encontró un bloque libre dentro del rango rel32 (+/- 2 GiB)".into(),
            })
        }
    }

    fn code_regions(&self) -> Result<Vec<MemoryRegion>, MemoryError> {
        let module_end = self.main_module.base + self.main_module.size as u64;
        let mut current = self.main_module.base;
        let mut regions = Vec::new();
        let mut mbi = MEMORY_BASIC_INFORMATION::default();

        while current < module_end {
            let queried = unsafe {
                VirtualQueryEx(
                    self.handle,
                    Some(current as *const c_void),
                    &mut mbi,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            if queried == 0 {
                break;
            }

            let region_end = (mbi.BaseAddress as u64 + mbi.RegionSize as u64).min(module_end);
            let executable = PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;
            if mbi.State == MEM_COMMIT && (mbi.Protect & executable).0 != 0 && (mbi.Protect & PAGE_GUARD).0 == 0 {
                regions.push(MemoryRegion { base: current, size: (region_end - current) as usize });
            }
            current = region_end;
        }

        Ok(regions)
    }

    fn is_running(&self) -> bool {
        unsafe {
            let mut exit_code = 0;
            if GetExitCodeProcess(self.handle, &mut exit_code).is_ok() {
                exit_code == STILL_ACTIVE.0 as u32
            } else {
                false
            }
        }
    }
}

pub struct WindowsHotkeyListener {
    down_keys: HashSet<u16>,
}

impl WindowsHotkeyListener {
    pub fn new() -> Self {
        Self {
            down_keys: HashSet::new(),
        }
    }

    fn is_key_down(&self, vk: VIRTUAL_KEY) -> bool {
        unsafe { (GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000) != 0 }
    }

    fn check_pressed(&mut self, vk: VIRTUAL_KEY) -> bool {
        let is_down = self.is_key_down(vk);
        let was_down = self.down_keys.contains(&vk.0);

        if is_down && !was_down {
            self.down_keys.insert(vk.0);
            true
        } else {
            if !is_down && was_down {
                self.down_keys.remove(&vk.0);
            }
            false
        }
    }
}

impl HotkeyListener for WindowsHotkeyListener {
    fn poll(&mut self) -> Vec<HotkeyEvent> {
        let mut events = Vec::new();
        let ctrl_down = self.is_key_down(VK_CONTROL);
        let shift_down = self.is_key_down(VK_SHIFT);

        // Ctrl + Shift + Inicio: Alternar atajos
        if ctrl_down && shift_down && self.check_pressed(VK_HOME) {
            events.push(HotkeyEvent::ToggleHotkeys);
            return events;
        }

        let numpad_keys = [
            (VK_NUMPAD0, 0),
            (VK_NUMPAD1, 1),
            (VK_NUMPAD2, 2),
            (VK_NUMPAD3, 3),
            (VK_NUMPAD4, 4),
            (VK_NUMPAD5, 5),
            (VK_NUMPAD6, 6),
            (VK_NUMPAD7, 7),
            (VK_NUMPAD8, 8),
            (VK_NUMPAD9, 9),
        ];

        for (vk, digit) in numpad_keys {
            if self.check_pressed(vk) {
                events.push(HotkeyEvent::Cheat(Hotkey {
                    ctrl: ctrl_down,
                    numpad: digit,
                }));
            }
        }

        events
    }
}
