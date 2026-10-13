//! Modelo de los trucos: qué hacen y cómo se activan, sin depender del sistema operativo.

use super::pattern::{Signature, Template};

/// Distribución del bloque de memoria reservado para cada code cave. El código de
/// las caves direcciona sus variables con desplazamientos relativos a RIP que
/// asumen que el código empieza en el offset 0 del bloque.
pub mod cave_layout {
    pub const BLOCK_SIZE: usize = 0x800;
    /// Espacio para código (incluido el salto de vuelta) antes de las variables.
    pub const CODE_CAPACITY: usize = 0x200;
    pub const FLAGS: u64 = 0x200;
    pub const DATA: u64 = 0x400;
}

/// Identificador estable de un truco (coincide con el ID del trainer original: "1", "C1"...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CheatId(pub &'static str);

/// Identificador de un hook persistente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HookId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hotkey {
    pub ctrl: bool,
    pub numpad: u8,
}

impl Hotkey {
    pub const fn numpad(digit: u8) -> Self {
        Self { ctrl: false, numpad: digit }
    }

    pub const fn ctrl_numpad(digit: u8) -> Self {
        Self { ctrl: true, numpad: digit }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalizedText {
    pub es: &'static str,
    pub en: &'static str,
}

/// Qué se escribe en el punto de parcheo.
#[derive(Debug, Clone)]
pub enum PatchBody {
    /// Sustituye los bytes originales por otros del mismo tamaño.
    Replace(Template),
    /// Salta a una "code cave" con este código; el salto de vuelta se añade solo.
    CodeCave(Template),
}

/// Una forma concreta de localizar y parchear un fragmento de código del juego.
#[derive(Debug, Clone)]
pub struct PatchVariant {
    pub signature: Signature,
    /// Distancia desde el inicio de la firma hasta el punto de parcheo.
    pub offset: usize,
    /// Bytes originales en el punto de parcheo (se restauran al desactivar).
    pub original: Template,
    pub body: PatchBody,
}

/// Parche de código con variantes alternativas para distintas versiones del juego.
#[derive(Debug, Clone)]
pub struct Patch {
    pub name: &'static str,
    pub variants: Vec<PatchVariant>,
    /// Si es opcional y no se encuentra, el truco se activa igualmente.
    pub required: bool,
}

/// Dirección exportada por un hook para que otros la usen como `$símbolo`.
#[derive(Debug, Clone, Copy)]
pub struct SymbolExport {
    pub symbol: &'static str,
    pub block_offset: u64,
}

/// Code cave que se instala una vez y queda activa; su comportamiento se controla
/// con valores en su bloque de datos.
#[derive(Debug, Clone)]
pub struct Hook {
    pub id: HookId,
    pub requires: Vec<HookId>,
    pub patch: Patch,
    pub exports: Vec<SymbolExport>,
}

/// Puntero estático leído de una instrucción con direccionamiento relativo a RIP
/// (p. ej. `mov rdx, [rip+X]`).
#[derive(Debug, Clone)]
pub struct RipRelativePointer {
    pub signature: Signature,
    pub displacement_offset: usize,
    pub instruction_len: usize,
    pub target_offset: u64,
}

#[derive(Debug, Clone)]
pub enum Effect {
    /// Parches que se aplican al activar y se revierten al desactivar.
    TogglePatches(Vec<Patch>),
    /// Valor escrito en el bloque de datos de un hook mientras el truco está activo.
    HookFlag {
        hook: HookId,
        block_offset: u64,
        on_value: u32,
        /// Truco que comparte la misma variable y se desactiva al activar este.
        exclusive_with: Option<CheatId>,
    },
    /// Escritura puntual a través de un puntero estático (soporta variantes de versión).
    SetValue { pointers: Vec<RipRelativePointer>, value: u32 },
}

impl Effect {
    pub fn is_one_shot(&self) -> bool {
        matches!(self, Self::SetValue { .. })
    }
}

#[derive(Debug, Clone)]
pub struct Cheat {
    pub id: CheatId,
    pub hotkey: Hotkey,
    pub name: LocalizedText,
    pub effect: Effect,
}

/// Todo lo que el trainer sabe de un juego concreto.
#[derive(Debug, Clone)]
pub struct GameCatalog {
    pub title: &'static str,
    pub supported_versions: &'static str,
    pub process_name: &'static str,
    pub hooks: Vec<Hook>,
    pub cheats: Vec<Cheat>,
}

impl GameCatalog {
    pub fn cheat(&self, id: CheatId) -> Option<&Cheat> {
        self.cheats.iter().find(|cheat| cheat.id == id)
    }

    pub fn cheat_for_hotkey(&self, hotkey: Hotkey) -> Option<&Cheat> {
        self.cheats.iter().find(|cheat| cheat.hotkey == hotkey)
    }

    pub fn hook(&self, id: HookId) -> Option<&Hook> {
        self.hooks.iter().find(|hook| hook.id == id)
    }
}
