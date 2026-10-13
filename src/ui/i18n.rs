//! Interface languages (English / Spanish) and every user-facing string.

use crate::application::error::TrainerError;
use crate::domain::cheat::LocalizedText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Spanish,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::English, Language::Spanish];

    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|language| code.trim().eq_ignore_ascii_case(language.code()))
    }

    /// Name of the language written in that same language, as selectors usually show it.
    pub fn native_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Spanish => "Español",
        }
    }

    /// First run: follow the system locale when it is Spanish, English otherwise.
    pub fn detect() -> Self {
        ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"]
            .iter()
            .filter_map(|key| std::env::var(key).ok())
            .find(|value| !value.is_empty())
            .filter(|value| value.to_ascii_lowercase().starts_with("es"))
            .map_or(Self::English, |_| Self::Spanish)
    }

    pub fn pick(self, text: LocalizedText) -> &'static str {
        match self {
            Self::English => text.en,
            Self::Spanish => text.es,
        }
    }

    pub fn strings(self) -> &'static Strings {
        match self {
            Self::English => &ENGLISH,
            Self::Spanish => &SPANISH,
        }
    }
}

/// A group of cheats shown as one card.
pub struct Section {
    pub title: LocalizedText,
    pub help: LocalizedText,
    pub cheat_ids: &'static [&'static str],
}

pub const SECTIONS: [Section; 4] = [
    Section {
        title: LocalizedText { es: "Combate", en: "Combat" },
        help: LocalizedText {
            es: "Trucos que afectan a la vida y a la barra de Break durante los combates. Solo funcionan con un combate en curso.",
            en: "Cheats that change health and the Break Gauge during fights. They only take effect while a match is running.",
        },
        cheat_ids: &["1", "2", "9", "0"],
    },
    Section {
        title: LocalizedText { es: "Tiempo y velocidad", en: "Time & Speed" },
        help: LocalizedText {
            es: "Controlan el reloj del round y la velocidad del juego. Supervelocidad y Cámara lenta no pueden estar activas a la vez.",
            en: "Control the round timer and game speed. Super Speed and Slow Motion cannot be on at the same time.",
        },
        cheat_ids: &["5", "7", "8"],
    },
    Section {
        title: LocalizedText { es: "Puntuación y desafíos", en: "Score & Challenges" },
        help: LocalizedText {
            es: "Ayudas para modos con puntuación, desafíos y misiones.",
            en: "Helpers for score-based modes, challenges and missions.",
        },
        cheat_ids: &["4", "6"],
    },
    Section {
        title: LocalizedText { es: "Desbloqueables y economía", en: "Unlockables & Economy" },
        help: LocalizedText {
            es: "Desbloqueos temporales en memoria: duran mientras el truco esté activo y se revierten al cerrar el trainer. Las compras hechas con Dinero infinito sí se guardan.",
            en: "Temporary in-memory unlocks: they last while the cheat is on and are reverted when the trainer closes. Purchases made with Infinite Money are saved for good.",
        },
        cheat_ids: &["C1", "3", "C2", "C3", "C4", "C5", "C6"],
    },
];

pub struct Strings {
    pub subtitle: &'static str,
    pub language_label: &'static str,
    pub language_help: &'static str,

    pub status_connected: &'static str,
    pub status_waiting: &'static str,
    pub status_help: &'static str,
    pub waiting_banner: &'static str,

    pub hotkeys_label: &'static str,
    pub hotkeys_help: &'static str,
    pub hotkeys_on: &'static str,
    pub hotkeys_off: &'static str,
    pub hotkeys_shortcut: &'static str,

    pub active_count: &'static str,
    pub disable_all: &'static str,
    pub disable_all_help: &'static str,

    pub switch_on: &'static str,
    pub switch_off: &'static str,
    pub apply: &'static str,
    pub applied: &'static str,
    pub hotkey_help: &'static str,
    pub requires_game: &'static str,
    pub help_prefix: &'static str,

    pub persistent_title: &'static str,
    pub persistent_help: &'static str,
    pub disk_chip: &'static str,
    pub dlc_name: &'static str,
    pub dlc_detail: &'static str,
    pub dlc_help: &'static str,
    pub dlc_not_found: &'static str,
    pub dlc_not_found_help: &'static str,
    pub money_tip: &'static str,

    pub dismiss: &'static str,
    pub footer_restore: &'static str,
    pub footer_credits: &'static str,
}

pub static ENGLISH: Strings = Strings {
    subtitle: "DOA6.exe v1.0–v1.20 · DOA6LR.exe (Last Round)",
    language_label: "Language",
    language_help: "Changes the trainer's language. Your choice is remembered for next time.",

    status_connected: "Connected",
    status_waiting: "Waiting for the game…",
    status_help: "The trainer looks for DOA6.exe or DOA6LR.exe every second. Once found, the options below are enabled. If the game is running but never connects, run the trainer as Administrator (Windows) or inside the same Wine prefix as the game (Linux).",
    waiting_banner: "Start DOA6.exe or DOA6LR.exe. Options unlock automatically once the game is detected.",

    hotkeys_label: "Global hotkeys",
    hotkeys_help: "When on, the numpad shortcuts shown next to each option work even while the game has focus. Press Ctrl+Shift+Home anywhere to turn them on or off so you don't trigger cheats by accident.",
    hotkeys_on: "On",
    hotkeys_off: "Paused",
    hotkeys_shortcut: "Toggle anywhere: Ctrl + Shift + Home",

    active_count: "active",
    disable_all: "Turn all off",
    disable_all_help: "Turns off every active cheat and restores the game's original code.",

    switch_on: "ON",
    switch_off: "OFF",
    apply: "Apply",
    applied: "Applied",
    hotkey_help: "Keyboard shortcut (numeric keypad). Works while global hotkeys are on.",
    requires_game: "Start the game first",
    help_prefix: "What does this do?",

    persistent_title: "Persistent options (disk)",
    persistent_help: "These options edit files on disk instead of game memory, so they stay in effect after closing the trainer and don't need the game to be running.",
    disk_chip: "DISK",
    dlc_name: "Unlock all DLC and characters",
    dlc_detail: "steam_emu.ini · Mai, Kula, Momiji, Rachel, costumes and modes",
    dlc_help: "Sets DLCUnlockall=1 in steam_emu.ini so the game treats every DLC as owned. It is permanent until you turn it off here. Restart the game to apply the change.",
    dlc_not_found: "steam_emu.ini not found",
    dlc_not_found_help: "Copy the trainer into the game's folder (next to steam_emu.ini) and open it again.",
    money_tip: "Tip: after applying Infinite Money (Ctrl+Num 1), anything you buy in the wardrobe is saved permanently in SYSTEMDATA.",

    dismiss: "Dismiss",
    footer_restore: "Every change in game memory is reverted automatically when you close the trainer.",
    footer_credits: "Clean Architecture · Written in Rust",
};

pub static SPANISH: Strings = Strings {
    subtitle: "DOA6.exe v1.0–v1.20 · DOA6LR.exe (Last Round)",
    language_label: "Idioma",
    language_help: "Cambia el idioma del trainer. Tu elección se recuerda para la próxima vez.",

    status_connected: "Conectado",
    status_waiting: "Esperando al juego…",
    status_help: "El trainer busca DOA6.exe o DOA6LR.exe cada segundo. Al encontrarlo se habilitan las opciones. Si el juego está abierto pero no conecta, ejecuta el trainer como Administrador (Windows) o dentro del mismo prefijo de Wine que el juego (Linux).",
    waiting_banner: "Inicia DOA6.exe o DOA6LR.exe. Las opciones se habilitan solas al detectar el juego.",

    hotkeys_label: "Atajos globales",
    hotkeys_help: "Si están activos, los atajos del teclado numérico que ves junto a cada opción funcionan aunque el juego tenga el foco. Pulsa Ctrl+Shift+Inicio en cualquier momento para activarlos o pausarlos y evitar activaciones accidentales.",
    hotkeys_on: "Activos",
    hotkeys_off: "En pausa",
    hotkeys_shortcut: "Alternar desde cualquier lugar: Ctrl + Shift + Inicio",

    active_count: "activos",
    disable_all: "Desactivar todo",
    disable_all_help: "Desactiva todos los trucos activos y restaura el código original del juego.",

    switch_on: "SÍ",
    switch_off: "NO",
    apply: "Aplicar",
    applied: "Aplicado",
    hotkey_help: "Atajo de teclado (teclado numérico). Funciona mientras los atajos globales estén activos.",
    requires_game: "Inicia el juego primero",
    help_prefix: "¿Qué hace esto?",

    persistent_title: "Opciones persistentes (disco)",
    persistent_help: "Estas opciones modifican archivos en disco en lugar de la memoria del juego: siguen activas tras cerrar el trainer y no necesitan el juego abierto.",
    disk_chip: "DISCO",
    dlc_name: "Desbloquear todos los DLC y personajes",
    dlc_detail: "steam_emu.ini · Mai, Kula, Momiji, Rachel, trajes y modos",
    dlc_help: "Escribe DLCUnlockall=1 en steam_emu.ini para que el juego considere todos los DLC como adquiridos. Es permanente hasta que lo desactives aquí. Reinicia el juego para aplicar el cambio.",
    dlc_not_found: "No se encontró steam_emu.ini",
    dlc_not_found_help: "Copia el trainer en la carpeta del juego (junto a steam_emu.ini) y vuelve a abrirlo.",
    money_tip: "Consejo: tras aplicar Dinero infinito (Ctrl+Num 1), lo que compres en el armario se guarda para siempre en SYSTEMDATA.",

    dismiss: "Cerrar",
    footer_restore: "Todos los cambios en la memoria del juego se revierten automáticamente al cerrar el trainer.",
    footer_credits: "Clean Architecture · Escrito en Rust",
};

/// Error message in the interface language.
pub fn error_message(language: Language, error: &TrainerError) -> String {
    let es = language == Language::Spanish;
    match error {
        TrainerError::GameNotRunning if es => "El juego no está en ejecución.".into(),
        TrainerError::GameNotRunning => "The game is not running.".into(),
        TrainerError::SignatureNotFound(name) if es => {
            format!("No se encontró la firma de «{name}»: esta opción no es compatible con tu versión del juego.")
        }
        TrainerError::SignatureNotFound(name) => {
            format!("Signature for “{name}” not found: this option is not supported by your game version.")
        }
        TrainerError::ValueNotReady if es => {
            "El valor aún no existe en el juego: carga tu partida o entra al menú principal y vuelve a intentarlo.".into()
        }
        TrainerError::ValueNotReady => {
            "The value doesn't exist in the game yet: load your save or reach the main menu, then try again.".into()
        }
        // Low-level technical errors are shown as they are
        other => format!("Error: {other}"),
    }
}

/// Error from the steam_emu.ini toggle.
pub fn dlc_error(language: Language, detail: &str) -> String {
    match language {
        Language::Spanish => format!("No se pudo modificar steam_emu.ini: {detail}"),
        Language::English => format!("Could not update steam_emu.ini: {detail}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::doa6;

    #[test]
    fn language_codes_round_trip() {
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(language));
        }
        assert_eq!(Language::from_code(" ES "), Some(Language::Spanish));
        assert_eq!(Language::from_code("fr"), None);
    }

    #[test]
    fn sections_cover_every_cheat_once() {
        let catalog = doa6::catalog();
        let mut ids: Vec<_> = SECTIONS.iter().flat_map(|section| section.cheat_ids.iter().copied()).collect();
        ids.sort();
        let mut expected: Vec<_> = catalog.cheats.iter().map(|cheat| cheat.id.0).collect();
        expected.sort();
        assert_eq!(ids, expected);
    }

    #[test]
    fn every_cheat_has_help_in_both_languages() {
        for cheat in doa6::catalog().cheats {
            assert!(!cheat.description.es.is_empty() && !cheat.description.en.is_empty(), "{}", cheat.id.0);
        }
    }
}
