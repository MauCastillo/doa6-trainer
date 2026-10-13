//! Implementación de pruebas/simulación para sistemas no Windows (Linux/macOS).

use crate::application::ports::{
    GameMemory, GameProcess, HotkeyEvent, HotkeyListener, MemoryError, MemoryRegion, ProcessFinder,
};

pub struct StubProcessFinder;

impl ProcessFinder for StubProcessFinder {
    fn find(&self, _process_name: &str) -> Result<Option<GameProcess>, MemoryError> {
        // En Linux fuera de Wine no hay proceso nativo Win32 por defecto
        Ok(None)
    }
}

pub struct StubHotkeyListener;

impl StubHotkeyListener {
    pub fn new() -> Self {
        Self
    }
}

impl HotkeyListener for StubHotkeyListener {
    fn poll(&mut self) -> Vec<HotkeyEvent> {
        Vec::new()
    }
}

#[allow(dead_code)]
pub struct StubGameMemory;

impl GameMemory for StubGameMemory {
    fn read(&self, _address: u64, _buffer: &mut [u8]) -> Result<(), MemoryError> {
        Ok(())
    }

    fn write(&self, _address: u64, _bytes: &[u8]) -> Result<(), MemoryError> {
        Ok(())
    }

    fn allocate_near(&self, _address: u64, _size: usize) -> Result<u64, MemoryError> {
        Ok(0)
    }

    fn code_regions(&self) -> Result<Vec<MemoryRegion>, MemoryError> {
        Ok(Vec::new())
    }

    fn is_running(&self) -> bool {
        false
    }
}
