//! Interfaces que la capa de aplicación necesita del sistema operativo.
//! La infraestructura las implementa; los tests usan dobles en memoria.

use std::fmt;

use crate::domain::cheat::Hotkey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryError {
    pub operation: &'static str,
    pub address: u64,
    pub detail: String,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} en 0x{:X}: {}", self.operation, self.address, self.detail)
    }
}

impl std::error::Error for MemoryError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    pub base: u64,
    pub size: usize,
}

/// Acceso a la memoria del proceso del juego.
pub trait GameMemory: Send {
    fn read(&self, address: u64, buffer: &mut [u8]) -> Result<(), MemoryError>;
    /// Escribe aunque la página sea de solo lectura/ejecución.
    fn write(&self, address: u64, bytes: &[u8]) -> Result<(), MemoryError>;
    /// Reserva memoria ejecutable a menos de ±2 GiB de `address`.
    fn allocate_near(&self, address: u64, size: usize) -> Result<u64, MemoryError>;
    /// Regiones ejecutables del módulo principal, donde se buscan las firmas.
    fn code_regions(&self) -> Result<Vec<MemoryRegion>, MemoryError>;
    fn is_running(&self) -> bool;
}

pub struct GameProcess {
    pub pid: u32,
    pub memory: Box<dyn GameMemory>,
}

pub trait ProcessFinder: Send {
    /// `Ok(None)` si el juego no está abierto; `Err` si está abierto pero no se puede acceder.
    fn find(&self, process_name: &str) -> Result<Option<GameProcess>, MemoryError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Cheat(Hotkey),
    /// Ctrl+Shift+Inicio: activa o desactiva todos los atajos.
    ToggleHotkeys,
}

pub trait HotkeyListener: Send {
    /// Teclas pulsadas desde la última llamada.
    fn poll(&mut self) -> Vec<HotkeyEvent>;
}
