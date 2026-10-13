//! Caso de uso principal: activar y desactivar trucos sobre el proceso del juego.

use std::collections::{HashMap, HashSet};

use super::error::TrainerError;
use super::patcher::{LocatedPatch, scan};
use super::ports::{GameMemory, GameProcess};
use crate::domain::cheat::{CheatId, Effect, GameCatalog, HookId, Patch, RipRelativePointer};
use crate::domain::pattern::Symbols;
use crate::domain::x86;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheatOutcome {
    Enabled,
    Disabled,
    /// Trucos de un solo uso (p. ej. fijar el dinero).
    Applied,
}

pub struct Trainer {
    catalog: GameCatalog,
    session: Option<Session>,
}

impl Trainer {
    pub fn new(catalog: GameCatalog) -> Self {
        Self { catalog, session: None }
    }

    pub fn catalog(&self) -> &GameCatalog {
        &self.catalog
    }

    pub fn game_pid(&self) -> Option<u32> {
        self.session.as_ref().map(|session| session.pid)
    }

    pub fn is_active(&self, id: CheatId) -> bool {
        self.session.as_ref().is_some_and(|session| session.active.contains(&id))
    }

    pub fn attach(&mut self, process: GameProcess) {
        // Una sesión anterior pertenece a otro proceso: no hay nada que restaurar en este.
        self.session = Some(Session::new(process));
    }

    /// Revierte todos los cambios en el juego y suelta el proceso.
    pub fn detach(&mut self) -> Result<(), TrainerError> {
        match self.session.take() {
            Some(session) if session.memory.is_running() => session.restore(),
            _ => Ok(()),
        }
    }

    /// Suelta la sesión si el juego se ha cerrado. Devuelve si sigue conectado.
    pub fn refresh(&mut self) -> bool {
        if self.session.as_ref().is_some_and(|session| !session.memory.is_running()) {
            self.session = None;
        }
        self.session.is_some()
    }

    pub fn toggle(&mut self, id: CheatId) -> Result<CheatOutcome, TrainerError> {
        let catalog = &self.catalog;
        let session = self.session.as_mut().ok_or(TrainerError::GameNotRunning)?;
        let cheat = catalog.cheat(id).ok_or(TrainerError::UnknownCheat(id.0))?;
        match &cheat.effect {
            Effect::TogglePatches(patches) => session.toggle_patches(catalog, id, patches),
            Effect::HookFlag { hook, block_offset, on_value, exclusive_with } => {
                session.toggle_flag(catalog, id, *hook, *block_offset, *on_value, *exclusive_with)
            }
            Effect::SetValue { pointers, value } => session.set_value(id, pointers, *value),
        }
    }
}

struct Session {
    pid: u32,
    memory: Box<dyn GameMemory>,
    /// En orden de instalación, para desinstalar en orden inverso.
    hooks: Vec<(HookId, LocatedPatch)>,
    patches: HashMap<CheatId, Vec<LocatedPatch>>,
    pointers: HashMap<CheatId, (u64, u64)>,
    active: HashSet<CheatId>,
}

impl Session {
    fn new(process: GameProcess) -> Self {
        Self {
            pid: process.pid,
            memory: process.memory,
            hooks: Vec::new(),
            patches: HashMap::new(),
            pointers: HashMap::new(),
            active: HashSet::new(),
        }
    }

    fn toggle_patches(
        &mut self,
        catalog: &GameCatalog,
        id: CheatId,
        patches: &[Patch],
    ) -> Result<CheatOutcome, TrainerError> {
        let memory = self.memory.as_ref();
        if self.active.remove(&id) {
            for located in self.patches[&id].iter().rev() {
                located.revert(memory)?;
            }
            return Ok(CheatOutcome::Disabled);
        }

        if !self.patches.contains_key(&id) {
            let located = locate_all(memory, patches)?;
            self.patches.insert(id, located);
        }
        let symbols = symbols(catalog, &self.hooks);
        let located = self.patches.get_mut(&id).expect("insertado arriba");
        for index in 0..located.len() {
            if let Err(error) = located[index].apply(memory, &symbols) {
                for applied in located[..index].iter().rev() {
                    let _ = applied.revert(memory);
                }
                return Err(error);
            }
        }
        self.active.insert(id);
        Ok(CheatOutcome::Enabled)
    }

    fn toggle_flag(
        &mut self,
        catalog: &GameCatalog,
        id: CheatId,
        hook: HookId,
        block_offset: u64,
        on_value: u32,
        exclusive_with: Option<CheatId>,
    ) -> Result<CheatOutcome, TrainerError> {
        let block = self.ensure_hook(catalog, hook)?;
        let enable = !self.active.contains(&id);
        let value = if enable { on_value } else { 0 };
        self.memory.write(block + block_offset, &value.to_le_bytes())?;

        if !enable {
            self.active.remove(&id);
            return Ok(CheatOutcome::Disabled);
        }
        // El otro truco comparte la variable: al escribir el nuevo valor queda desactivado.
        if let Some(other) = exclusive_with {
            self.active.remove(&other);
        }
        self.active.insert(id);
        Ok(CheatOutcome::Enabled)
    }

    fn set_value(
        &mut self,
        id: CheatId,
        pointers: &[RipRelativePointer],
        value: u32,
    ) -> Result<CheatOutcome, TrainerError> {
        let (static_address, target_offset) = match self.pointers.get(&id) {
            Some((address, offset)) => (*address, *offset),
            None => {
                let mut found = None;
                for ptr in pointers {
                    if let Ok(address) = resolve_static_pointer(self.memory.as_ref(), id, ptr) {
                        found = Some((address, ptr.target_offset));
                        break;
                    }
                }
                let (address, offset) = found.ok_or(TrainerError::SignatureNotFound(id.0))?;
                self.pointers.insert(id, (address, offset));
                (address, offset)
            }
        };
        let target = read_u64(self.memory.as_ref(), static_address)?;
        if target == 0 {
            return Err(TrainerError::ValueNotReady);
        }
        self.memory.write(target + target_offset, &value.to_le_bytes())?;
        Ok(CheatOutcome::Applied)
    }

    /// Instala el hook (y los que necesita) si no lo estaba. Devuelve su bloque de datos.
    fn ensure_hook(&mut self, catalog: &GameCatalog, id: HookId) -> Result<u64, TrainerError> {
        if let Some((_, located)) = self.hooks.iter().find(|(installed, _)| *installed == id) {
            return Ok(cave_block_of(id, located));
        }
        let hook = catalog.hook(id).ok_or(TrainerError::UnknownHook(id.0))?;
        for dependency in &hook.requires {
            self.ensure_hook(catalog, *dependency)?;
        }
        let memory = self.memory.as_ref();
        let mut located =
            LocatedPatch::locate(memory, &hook.patch)?.ok_or(TrainerError::SignatureNotFound(hook.patch.name))?;
        located.apply(memory, &symbols(catalog, &self.hooks))?;
        let block = cave_block_of(id, &located);
        self.hooks.push((id, located));
        Ok(block)
    }

    fn restore(self) -> Result<(), TrainerError> {
        let memory = self.memory.as_ref();
        let active_patches = self
            .patches
            .iter()
            .filter(|(id, _)| self.active.contains(id))
            .flat_map(|(_, located)| located.iter().rev());
        let hooks = self.hooks.iter().rev().map(|(_, located)| located);

        // Se intenta restaurar todo aunque algo falle; se informa del primer error.
        let mut first_error = None;
        for located in active_patches.chain(hooks) {
            if let Err(error) = located.revert(memory) {
                first_error.get_or_insert(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

fn locate_all(memory: &dyn GameMemory, patches: &[Patch]) -> Result<Vec<LocatedPatch>, TrainerError> {
    let mut located = Vec::with_capacity(patches.len());
    for patch in patches {
        match LocatedPatch::locate(memory, patch)? {
            Some(found) => located.push(found),
            None if patch.required => return Err(TrainerError::SignatureNotFound(patch.name)),
            None => {}
        }
    }
    Ok(located)
}

fn cave_block_of(id: HookId, located: &LocatedPatch) -> u64 {
    located
        .cave_block()
        .unwrap_or_else(|| panic!("el hook {} debe ser una code cave ya aplicada", id.0))
}

fn symbols(catalog: &GameCatalog, hooks: &[(HookId, LocatedPatch)]) -> Symbols {
    hooks
        .iter()
        .filter_map(|(id, located)| Some((catalog.hook(*id)?, located.cave_block()?)))
        .flat_map(|(hook, block)| {
            hook.exports.iter().map(move |export| (export.symbol.to_owned(), block + export.block_offset))
        })
        .collect()
}

fn resolve_static_pointer(
    memory: &dyn GameMemory,
    id: CheatId,
    pointer: &RipRelativePointer,
) -> Result<u64, TrainerError> {
    let found = scan(memory, &pointer.signature)?.ok_or(TrainerError::SignatureNotFound(id.0))?;
    let start = pointer.displacement_offset;
    let displacement = i32::from_le_bytes(found.bytes[start..start + 4].try_into().expect("4 bytes"));
    Ok(x86::rip_relative_target(found.address, pointer.instruction_len, displacement))
}

fn read_u64(memory: &dyn GameMemory, address: u64) -> Result<u64, TrainerError> {
    let mut buffer = [0u8; 8];
    memory.read(address, &mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::application::fake_memory::FakeMemory;
    use crate::domain::cheat::{cave_layout, Cheat, Hook, Hotkey, LocalizedText, PatchBody, PatchVariant, SymbolExport};
    use crate::domain::pattern::{Signature, Template};

    const CODE: u64 = 0x1_4000_0000;
    const PLAYER: HookId = HookId("player");
    const KO: HookId = HookId("ko");

    // Código simulado: [sitio del hook del jugador][sitio del hook de K.O.][sitio de parche][instrucción con puntero]
    fn game_code() -> Vec<u8> {
        let mut code = vec![0xCC; 16];
        code.extend([0x66, 0x83, 0xB8, 0x34, 0x12, 0x00, 0x00, 0x14, 0x77]);
        code.extend([0x66, 0x2B, 0xCA, 0x66, 0x3B, 0xC1, 0xEE]);
        code.extend([0xF6, 0x00, 0x0A, 0xDD]);
        code.extend([0x48, 0x8B, 0x15, 0x10, 0x00, 0x00, 0x00, 0xAB]);
        code.extend([0xCC; 32]);
        code
    }

    fn cave(signature: &str, original: &str, code: &str) -> Patch {
        Patch {
            name: "test",
            required: true,
            variants: vec![PatchVariant {
                signature: Signature::parse(signature).unwrap(),
                offset: 0,
                original: Template::parse(original).unwrap(),
                body: PatchBody::CodeCave(Template::parse(code).unwrap()),
            }],
        }
    }

    fn cheat(id: &'static str, effect: Effect) -> Cheat {
        let text = LocalizedText { es: id, en: id };
        Cheat { id: CheatId(id), hotkey: Hotkey::numpad(0), name: text, description: text, effect }
    }

    fn catalog() -> GameCatalog {
        GameCatalog {
            title: "test",
            supported_versions: "test",
            process_name: "test.exe",
            hooks: vec![
                Hook {
                    id: PLAYER,
                    requires: vec![],
                    patch: cave("66 83 B8 s1 00 00 14 77", "66 83 B8 s1 00 00 14", "66 83 B8 s1 00 00 14"),
                    exports: vec![SymbolExport { symbol: "player", block_offset: cave_layout::DATA }],
                },
                Hook {
                    id: KO,
                    requires: vec![PLAYER],
                    patch: cave("66 2B CA 66 3B C1 EE", "66 2B CA 66 3B C1", "48 A1 $player 66 2B CA 66 3B C1"),
                    exports: vec![],
                },
            ],
            cheats: vec![
                cheat(
                    "fast",
                    Effect::HookFlag { hook: PLAYER, block_offset: 0x200, on_value: 7, exclusive_with: Some(CheatId("slow")) },
                ),
                cheat(
                    "slow",
                    Effect::HookFlag { hook: PLAYER, block_offset: 0x200, on_value: 3, exclusive_with: Some(CheatId("fast")) },
                ),
                cheat("ko", Effect::HookFlag { hook: KO, block_offset: 0x200, on_value: 1, exclusive_with: None }),
                cheat(
                    "unlock",
                    Effect::TogglePatches(vec![Patch {
                        name: "unlock",
                        required: true,
                        variants: vec![PatchVariant {
                            signature: Signature::parse("F6 00 0A DD").unwrap(),
                            offset: 0,
                            original: Template::parse("F6 00 0A").unwrap(),
                            body: PatchBody::Replace(Template::parse("48 85 C0").unwrap()),
                        }],
                    }]),
                ),
                cheat(
                    "money",
                    Effect::SetValue {
                        pointers: vec![RipRelativePointer {
                            signature: Signature::parse("48 8B 15 * * * * AB").unwrap(),
                            displacement_offset: 3,
                            instruction_len: 7,
                            target_offset: 0,
                        }],
                        value: 99_999_999,
                    },
                ),
            ],
        }
    }

    fn attached() -> (Trainer, Arc<FakeMemory>) {
        let memory = Arc::new(FakeMemory::with_code(CODE, &game_code()));
        let mut trainer = Trainer::new(catalog());
        trainer.attach(GameProcess { pid: 42, memory: Box::new(memory.clone()) });
        (trainer, memory)
    }

    #[test]
    fn toggle_without_game_fails() {
        let mut trainer = Trainer::new(catalog());
        assert_eq!(trainer.toggle(CheatId("fast")), Err(TrainerError::GameNotRunning));
    }

    #[test]
    fn flag_installs_hook_once_and_toggles_value() {
        let (mut trainer, memory) = attached();

        assert_eq!(trainer.toggle(CheatId("fast")), Ok(CheatOutcome::Enabled));
        let block = memory_hook_block(&memory, CODE + 16);
        assert_eq!(memory.u32_at(block + 0x200), 7);

        assert_eq!(trainer.toggle(CheatId("fast")), Ok(CheatOutcome::Disabled));
        assert_eq!(memory.u32_at(block + 0x200), 0);
        assert_eq!(memory_hook_block(&memory, CODE + 16), block);
    }

    #[test]
    fn exclusive_flags_replace_each_other() {
        let (mut trainer, memory) = attached();

        trainer.toggle(CheatId("fast")).unwrap();
        trainer.toggle(CheatId("slow")).unwrap();

        let block = memory_hook_block(&memory, CODE + 16);
        assert_eq!(memory.u32_at(block + 0x200), 3);
        assert!(trainer.is_active(CheatId("slow")));
        assert!(!trainer.is_active(CheatId("fast")));
    }

    #[test]
    fn dependent_hook_receives_exported_symbol() {
        let (mut trainer, memory) = attached();

        trainer.toggle(CheatId("ko")).unwrap();

        let player_block = memory_hook_block(&memory, CODE + 16);
        let ko_block = memory_hook_block(&memory, CODE + 25);
        assert_eq!(memory.bytes(ko_block, 2), vec![0x48, 0xA1]);
        let player_slot = u64::from_le_bytes(memory.bytes(ko_block + 2, 8).try_into().unwrap());
        assert_eq!(player_slot, player_block + cave_layout::DATA);
    }

    #[test]
    fn toggled_patch_is_applied_and_reverted() {
        let (mut trainer, memory) = attached();
        let site = CODE + 32;

        assert_eq!(trainer.toggle(CheatId("unlock")), Ok(CheatOutcome::Enabled));
        assert_eq!(memory.bytes(site, 3), vec![0x48, 0x85, 0xC0]);

        assert_eq!(trainer.toggle(CheatId("unlock")), Ok(CheatOutcome::Disabled));
        assert_eq!(memory.bytes(site, 3), vec![0xF6, 0x00, 0x0A]);
    }

    #[test]
    fn set_value_writes_through_static_pointer() {
        let (mut trainer, memory) = attached();
        let instruction = CODE + 36;
        let static_address = instruction + 7 + 0x10;
        let money = 0x2_0000_0000;
        memory.map(money, &[0; 4]);
        memory.write(static_address, &money.to_le_bytes()).unwrap();

        assert_eq!(trainer.toggle(CheatId("money")), Ok(CheatOutcome::Applied));
        assert_eq!(memory.u32_at(money), 99_999_999);
        assert!(!trainer.is_active(CheatId("money")));
    }

    #[test]
    fn set_value_with_null_pointer_reports_not_ready() {
        let (mut trainer, memory) = attached();
        let static_address = CODE + 36 + 7 + 0x10;
        memory.write(static_address, &0u64.to_le_bytes()).unwrap();

        assert_eq!(trainer.toggle(CheatId("money")), Err(TrainerError::ValueNotReady));
    }

    #[test]
    fn detach_restores_original_code() {
        let (mut trainer, memory) = attached();
        trainer.toggle(CheatId("ko")).unwrap();
        trainer.toggle(CheatId("unlock")).unwrap();

        trainer.detach().unwrap();

        assert_eq!(memory.bytes(CODE, game_code().len() as usize), game_code());
        assert_eq!(trainer.game_pid(), None);
    }

    #[test]
    fn refresh_drops_session_when_game_exits() {
        let (mut trainer, memory) = attached();
        trainer.toggle(CheatId("unlock")).unwrap();

        memory.exit();

        assert!(!trainer.refresh());
        assert!(!trainer.is_active(CheatId("unlock")));
    }

    /// Sigue el `jmp rel32` escrito en `site` hasta su code cave.
    fn memory_hook_block(memory: &FakeMemory, site: u64) -> u64 {
        let jump = memory.bytes(site, 5);
        assert_eq!(jump[0], 0xE9, "no hay salto en 0x{site:X}");
        x86::rip_relative_target(site, 5, i32::from_le_bytes(jump[1..5].try_into().unwrap()))
    }
}
