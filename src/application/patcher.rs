//! Localiza parches en la memoria del juego y los aplica o revierte.

use super::error::TrainerError;
use super::ports::GameMemory;
use crate::domain::cheat::{Patch, PatchBody, cave_layout};
use crate::domain::pattern::{Captures, Signature, Symbols};
use crate::domain::x86;

/// Coincidencia de una firma en el proceso.
pub struct SignatureMatch {
    pub address: u64,
    pub bytes: Vec<u8>,
}

/// Busca `signature` en las regiones de código del juego.
pub fn scan(memory: &dyn GameMemory, signature: &Signature) -> Result<Option<SignatureMatch>, TrainerError> {
    for region in memory.code_regions()? {
        let mut buffer = vec![0u8; region.size];
        // Una región ilegible (p. ej. liberada entre la consulta y la lectura) no es un error.
        if memory.read(region.base, &mut buffer).is_err() {
            continue;
        }
        if let Some(start) = signature.find_in(&buffer) {
            return Ok(Some(SignatureMatch {
                address: region.base + start as u64,
                bytes: buffer[start..start + signature.len()].to_vec(),
            }));
        }
    }
    Ok(None)
}

/// Parche encontrado en memoria, listo para aplicarse o revertirse.
pub struct LocatedPatch {
    name: &'static str,
    address: u64,
    original: Vec<u8>,
    body: PatchBody,
    captures: Captures,
    cave_block: Option<u64>,
}

impl LocatedPatch {
    pub fn cave_block(&self) -> Option<u64> {
        self.cave_block
    }

    /// Prueba cada variante del parche hasta encontrar una.
    pub fn locate(memory: &dyn GameMemory, patch: &Patch) -> Result<Option<Self>, TrainerError> {
        for variant in &patch.variants {
            let Some(found) = scan(memory, &variant.signature)? else {
                continue;
            };
            let captures = variant.signature.read_captures(&found.bytes);
            let start = variant.offset;
            let original = found.bytes[start..start + variant.original.len()].to_vec();
            return Ok(Some(Self {
                name: patch.name,
                address: found.address + start as u64,
                original,
                body: variant.body.clone(),
                captures,
                cave_block: None,
            }));
        }
        Ok(None)
    }

    pub fn apply(&mut self, memory: &dyn GameMemory, symbols: &Symbols) -> Result<(), TrainerError> {
        match &self.body {
            PatchBody::Replace(replacement) => {
                let bytes = replacement.render(&self.captures, symbols)?;
                memory.write(self.address, &bytes)?;
            }
            PatchBody::CodeCave(code) => {
                let mut cave = code.render(&self.captures, symbols)?;
                cave.extend_from_slice(&x86::absolute_jump(self.return_address()));
                if cave.len() > cave_layout::CODE_CAPACITY {
                    return Err(TrainerError::CaveTooLarge(self.name));
                }
                let block = self.ensure_cave_block(memory)?;
                let jump = x86::relative_jump(self.address, block, self.original.len())
                    .ok_or(TrainerError::JumpOutOfRange)?;
                // Primero la cave y después el salto: el juego nunca ve una cave a medias.
                memory.write(block, &cave)?;
                memory.write(self.address, &jump)?;
            }
        }
        Ok(())
    }

    /// Restaura los bytes originales. La cave se conserva: algún hilo del juego
    /// podría estar ejecutándola todavía, y liberarla lo cerraría.
    pub fn revert(&self, memory: &dyn GameMemory) -> Result<(), TrainerError> {
        memory.write(self.address, &self.original)?;
        Ok(())
    }

    fn return_address(&self) -> u64 {
        self.address + self.original.len() as u64
    }

    fn ensure_cave_block(&mut self, memory: &dyn GameMemory) -> Result<u64, TrainerError> {
        if let Some(block) = self.cave_block {
            return Ok(block);
        }
        let block = memory.allocate_near(self.address, cave_layout::BLOCK_SIZE)?;
        self.cave_block = Some(block);
        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::fake_memory::FakeMemory;
    use crate::domain::cheat::PatchVariant;
    use crate::domain::pattern::Template;

    const CODE_BASE: u64 = 0x1_4000_1000;

    fn patch(body: PatchBody) -> Patch {
        Patch {
            name: "test",
            required: true,
            variants: vec![PatchVariant {
                signature: Signature::parse("11 22 66 83 B8 s1 00 00 14 77").unwrap(),
                offset: 2,
                original: Template::parse("66 83 B8 s1 00 00 14").unwrap(),
                body,
            }],
        }
    }

    fn memory() -> FakeMemory {
        FakeMemory::with_code(CODE_BASE, &[0x90, 0x11, 0x22, 0x66, 0x83, 0xB8, 0x34, 0x12, 0x00, 0x00, 0x14, 0x77])
    }

    #[test]
    fn replace_patch_writes_and_restores() {
        let memory = memory();
        let patch = patch(PatchBody::Replace(Template::parse("90 90 90 90 90 90 90 90").unwrap()));
        let mut located = LocatedPatch::locate(&memory, &patch).unwrap().unwrap();

        located.apply(&memory, &Symbols::new()).unwrap();
        assert_eq!(memory.bytes(CODE_BASE + 3, 8), vec![0x90; 8]);

        located.revert(&memory).unwrap();
        assert_eq!(memory.bytes(CODE_BASE + 3, 8), vec![0x66, 0x83, 0xB8, 0x34, 0x12, 0x00, 0x00, 0x14]);
    }

    #[test]
    fn code_cave_jumps_to_block_and_back() {
        let memory = memory();
        let patch = patch(PatchBody::CodeCave(Template::parse("66 C7 80 s1 00 00 A8 61").unwrap()));
        let mut located = LocatedPatch::locate(&memory, &patch).unwrap().unwrap();

        located.apply(&memory, &Symbols::new()).unwrap();

        let site = CODE_BASE + 3;
        let block = located.cave_block().unwrap();
        let jump = memory.bytes(site, 8);
        assert_eq!(jump[0], 0xE9);
        assert_eq!(x86::rip_relative_target(site, 5, i32::from_le_bytes(jump[1..5].try_into().unwrap())), block);
        assert_eq!(&jump[5..], &[0x90, 0x90, 0x90]);

        let cave = memory.bytes(block, 23);
        assert_eq!(&cave[..9], &[0x66, 0xC7, 0x80, 0x34, 0x12, 0x00, 0x00, 0xA8, 0x61]);
        assert_eq!(&cave[9..15], &[0xFF, 0x25, 0, 0, 0, 0]);
        assert_eq!(u64::from_le_bytes(cave[15..23].try_into().unwrap()), site + 8);
    }

    #[test]
    fn missing_signature_is_not_located() {
        let memory = FakeMemory::with_code(CODE_BASE, &[0x00; 32]);
        let patch = patch(PatchBody::Replace(Template::parse("90").unwrap()));
        assert!(LocatedPatch::locate(&memory, &patch).unwrap().is_none());
    }
}
