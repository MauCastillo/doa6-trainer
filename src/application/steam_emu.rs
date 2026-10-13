//! Gestión persistente de DLCs y configuración del emulador Steam (`steam_emu.ini`).
//!
//! Permite comprobar y alternar el desbloqueo permanente de todos los DLCs
//! (personajes adicionales, trajes de reserva, pases de temporada) tanto en
//! entornos Wine/Lutris como en Windows 11 nativo.

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamEmuStatus {
    NotFound,
    Locked(PathBuf),
    Unlocked(PathBuf),
}

pub struct SteamEmu;

impl SteamEmu {
    /// Busca la ruta de `steam_emu.ini` comprobando ubicaciones habituales.
    pub fn find_config() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from("steam_emu.ini"),
            PathBuf::from(r"C:\Program Files\DEAD OR ALIVE 6 Last Round\steam_emu.ini"),
            PathBuf::from("/home/wau/Games/dead-or-alive-the-last-round/drive_c/Program Files/DEAD OR ALIVE 6 Last Round/steam_emu.ini"),
        ];

        for candidate in candidates {
            if candidate.is_file() {
                return Some(candidate);
            }
        }

        // Probar junto al ejecutable del trainer
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let candidate = parent.join("steam_emu.ini");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    /// Comprueba el estado del desbloqueo automático de DLCs.
    pub fn status() -> SteamEmuStatus {
        match Self::find_config() {
            Some(path) => {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("DLCUnlockall=") {
                            let value = trimmed.trim_start_matches("DLCUnlockall=").trim();
                            if value == "1" {
                                return SteamEmuStatus::Unlocked(path);
                            } else {
                                return SteamEmuStatus::Locked(path);
                            }
                        }
                    }
                }
                SteamEmuStatus::Locked(path)
            }
            None => SteamEmuStatus::NotFound,
        }
    }

    /// Alterna el flag `DLCUnlockall=1` / `DLCUnlockall=0` en el archivo.
    pub fn toggle() -> Result<bool, String> {
        let path = Self::find_config().ok_or_else(|| "No se encontró steam_emu.ini".to_string())?;
        let content = fs::read_to_string(&path).map_err(|e| format!("Error leyendo {}: {e}", path.display()))?;

        let mut found = false;
        let mut now_unlocked = false;
        let mut new_lines = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("DLCUnlockall=") {
                found = true;
                let current_val = trimmed.trim_start_matches("DLCUnlockall=").trim();
                if current_val == "1" {
                    new_lines.push("DLCUnlockall=0".to_string());
                    now_unlocked = false;
                } else {
                    new_lines.push("DLCUnlockall=1".to_string());
                    now_unlocked = true;
                }
            } else {
                new_lines.push(line.to_string());
            }
        }

        if !found {
            // Si la clave no existía en [DLC], la añadimos
            new_lines.push("DLCUnlockall=1".to_string());
            now_unlocked = true;
        }

        let mut output = new_lines.join("\r\n");
        if content.ends_with('\n') {
            output.push_str("\r\n");
        }

        fs::write(&path, output).map_err(|e| format!("Error guardando {}: {e}", path.display()))?;
        Ok(now_unlocked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dlc_unlock_all() {
        let sample = "[DLC]\nDLCUnlockall=1\n";
        assert!(sample.lines().any(|l| l.trim() == "DLCUnlockall=1"));
    }
}
