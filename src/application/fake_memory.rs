//! Proceso simulado para probar la lógica sin Windows ni el juego.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use super::ports::{GameMemory, MemoryError, MemoryRegion};

const ALLOCATION_BASE: u64 = 0x1_3000_0000;

struct Segment {
    base: u64,
    bytes: Vec<u8>,
}

pub struct FakeMemory {
    code: MemoryRegion,
    segments: Mutex<Vec<Segment>>,
    running: AtomicBool,
}

impl FakeMemory {
    pub fn with_code(base: u64, code: &[u8]) -> Self {
        Self {
            code: MemoryRegion { base, size: code.len() },
            segments: Mutex::new(vec![Segment { base, bytes: code.to_vec() }]),
            running: AtomicBool::new(true),
        }
    }

    /// Añade datos fuera de la región de código (p. ej. el destino de un puntero).
    pub fn map(&self, base: u64, bytes: &[u8]) {
        self.segments.lock().unwrap().push(Segment { base, bytes: bytes.to_vec() });
    }

    pub fn bytes(&self, address: u64, len: usize) -> Vec<u8> {
        let mut buffer = vec![0; len];
        self.read(address, &mut buffer).unwrap();
        buffer
    }

    pub fn u32_at(&self, address: u64) -> u32 {
        u32::from_le_bytes(self.bytes(address, 4).try_into().unwrap())
    }

    pub fn exit(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn with_slice<R>(
        &self,
        operation: &'static str,
        address: u64,
        len: usize,
        action: impl FnOnce(&mut [u8]) -> R,
    ) -> Result<R, MemoryError> {
        let mut segments = self.segments.lock().unwrap();
        let segment = segments
            .iter_mut()
            .find(|segment| address >= segment.base && address + len as u64 <= segment.base + segment.bytes.len() as u64)
            .ok_or_else(|| MemoryError { operation, address, detail: "dirección no mapeada".into() })?;
        let start = (address - segment.base) as usize;
        Ok(action(&mut segment.bytes[start..start + len]))
    }
}

/// Permite que el test conserve una referencia mientras el trainer posee otra.
impl<T: GameMemory + Sync> GameMemory for std::sync::Arc<T> {
    fn read(&self, address: u64, buffer: &mut [u8]) -> Result<(), MemoryError> {
        (**self).read(address, buffer)
    }

    fn write(&self, address: u64, bytes: &[u8]) -> Result<(), MemoryError> {
        (**self).write(address, bytes)
    }

    fn allocate_near(&self, address: u64, size: usize) -> Result<u64, MemoryError> {
        (**self).allocate_near(address, size)
    }

    fn code_regions(&self) -> Result<Vec<MemoryRegion>, MemoryError> {
        (**self).code_regions()
    }

    fn is_running(&self) -> bool {
        (**self).is_running()
    }
}

impl GameMemory for FakeMemory {
    fn read(&self, address: u64, buffer: &mut [u8]) -> Result<(), MemoryError> {
        self.with_slice("leer", address, buffer.len(), |slice| buffer.copy_from_slice(slice))
    }

    fn write(&self, address: u64, bytes: &[u8]) -> Result<(), MemoryError> {
        self.with_slice("escribir", address, bytes.len(), |slice| slice.copy_from_slice(bytes))
    }

    fn allocate_near(&self, _address: u64, size: usize) -> Result<u64, MemoryError> {
        let mut segments = self.segments.lock().unwrap();
        let base = ALLOCATION_BASE + 0x1_0000 * segments.len() as u64;
        segments.push(Segment { base, bytes: vec![0; size] });
        Ok(base)
    }

    fn code_regions(&self) -> Result<Vec<MemoryRegion>, MemoryError> {
        Ok(vec![self.code])
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}
