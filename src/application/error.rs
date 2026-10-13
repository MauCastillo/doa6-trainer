use std::fmt;

use super::ports::MemoryError;
use crate::domain::pattern::PatternError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrainerError {
    GameNotRunning,
    UnknownCheat(&'static str),
    UnknownHook(&'static str),
    /// La firma no aparece: probablemente la versión del juego no es compatible.
    SignatureNotFound(&'static str),
    /// El juego aún no ha creado el dato (p. ej. la partida no está cargada).
    ValueNotReady,
    JumpOutOfRange,
    CaveTooLarge(&'static str),
    Pattern(PatternError),
    Memory(MemoryError),
}

impl fmt::Display for TrainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameNotRunning => write!(f, "el juego no está en ejecución"),
            Self::UnknownCheat(id) => write!(f, "truco desconocido: {id}"),
            Self::UnknownHook(id) => write!(f, "hook desconocido: {id}"),
            Self::SignatureNotFound(name) => write!(f, "no se encontró la firma de «{name}»"),
            Self::ValueNotReady => write!(f, "el valor aún no está disponible en el juego"),
            Self::JumpOutOfRange => write!(f, "la code cave quedó fuera del alcance de un salto rel32"),
            Self::CaveTooLarge(name) => write!(f, "la code cave de «{name}» no cabe en su bloque"),
            Self::Pattern(error) => write!(f, "{error}"),
            Self::Memory(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for TrainerError {}

impl From<MemoryError> for TrainerError {
    fn from(error: MemoryError) -> Self {
        Self::Memory(error)
    }
}

impl From<PatternError> for TrainerError {
    fn from(error: PatternError) -> Self {
        Self::Pattern(error)
    }
}
