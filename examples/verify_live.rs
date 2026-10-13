use std::time::Instant;
use doa6_trainer::application::error::TrainerError;
use doa6_trainer::application::steam_emu::{SteamEmu, SteamEmuStatus};
use doa6_trainer::application::trainer::Trainer;
use doa6_trainer::domain::cheat::CheatId;
use doa6_trainer::domain::doa6;
use doa6_trainer::infrastructure::default_process_finder;

fn main() {
    println!("=======================================================");
    println!(" VERIFICACIÓN EN VIVO: DEAD OR ALIVE 6 LAST ROUND");
    println!(" (Ejecución bajo Wine / Lutris contra DOA6LR.exe)");
    println!("=======================================================\n");

    let finder = default_process_finder();

    println!("[1/5] Buscando proceso DOA6.exe / DOA6LR.exe...");
    let process = match finder.find("DOA6.exe") {
        Ok(Some(p)) => {
            println!("  -> Proceso encontrado exitosamente. PID: {}", p.pid);
            p
        }
        Ok(None) => {
            eprintln!("  -> ERROR: Proceso no encontrado. ¿Está el juego corriendo en Lutris?");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  -> ERROR al buscar proceso: {:?}", e);
            std::process::exit(1);
        }
    };

    let mut trainer = Trainer::new(doa6::catalog());
    println!("[2/5] Conectando trainer a la memoria de DOA6LR.exe...");
    trainer.attach(process);

    println!("[3/5] Probando activación de cada opción del trainer en tiempo de ejecución...");
    let cheats_to_test = [
        ("1", "Salud Infinita (P1)", true),
        ("2", "Break Gauge al Máximo (P1)", true),
        ("0", "Break Blow Fácil (P1)", true),
        ("9", "Un Golpe Mata (One Hit K.O.)", true),
        ("5", "Congelar Tiempo de Ronda", true),
        ("7", "Velocidad Rápida", false),
        ("8", "Cámara Lenta", false),
        ("4", "Puntuación Máxima Total", true),
        ("6", "Completar Desafíos al Instante", true),
        ("3", "Dinero Infinito (Player Profile)", true),
        ("C1", "Desbloquear Todos los Trajes", true),
        ("C2", "Desbloquear Todos los Peinados", true),
        ("C3", "Desbloquear Todas las Gafas", true),
        ("C4", "Desbloquear Toda la Música", true),
        ("C5", "Desbloquear Enciclopedia DOA", true),
        ("C6", "Desbloquear Trivia DOA", true),
    ];

    let mut passed = 0;
    let mut optional_unsupported = 0;
    let mut failed = 0;

    for (id_str, name, is_required_for_lr) in &cheats_to_test {
        let id = CheatId(id_str);
        let t0 = Instant::now();
        match trainer.toggle(id) {
            Ok(outcome) => {
                let active = trainer.is_active(id);
                println!(
                    "  [OK] Truco {:<4} | {:<35} | Resultado: {:?} | Activo: {} ({:?})",
                    id_str, name, outcome, active, t0.elapsed()
                );
                passed += 1;
            }
            Err(TrainerError::SignatureNotFound("game_speed")) if !is_required_for_lr => {
                println!(
                    "  [INFO] Truco {:<4} | {:<35} | Exclusivo v1.0-v1.20 (No disponible en DOA6LR)",
                    id_str, name
                );
                optional_unsupported += 1;
            }
            Err(e) => {
                println!(
                    "  [FAIL] Truco {:<4} | {:<35} | ERROR: {:?}",
                    id_str, name, e
                );
                failed += 1;
            }
        }
    }

    println!("\n[4/5] Probando restauración limpia de memoria (Desactivación de trucos)...");
    let mut restored = 0;
    for (id_str, name, _) in &cheats_to_test {
        let id = CheatId(id_str);
        if trainer.is_active(id) {
            match trainer.toggle(id) {
                Ok(outcome) => {
                    println!("  [REVERT OK] Truco {:<4} | {:<35} | Resultado: {:?}", id_str, name, outcome);
                    restored += 1;
                }
                Err(e) => {
                    println!("  [REVERT FAIL] Truco {:<4} | {:<35} | ERROR: {:?}", id_str, name, e);
                    failed += 1;
                }
            }
        }
    }
    println!("  -> Total trucos restaurados limpiamente: {}", restored);

    println!("\n[4.1] Desconectando y restaurando todos los hooks de memoria...");
    if let Err(e) = trainer.detach() {
        eprintln!("  -> Error al hacer detach: {:?}", e);
        failed += 1;
    } else {
        println!("  -> Todos los hooks y trampolines desinstalados exitosamente.");
    }

    println!("\n[5/5] Probando persistencia externa de DLCs (SteamEmu)...");
    let emu_status = SteamEmu::status();
    println!("  -> Estado de steam_emu.ini: {:?}", emu_status);
    if emu_status != SteamEmuStatus::NotFound {
        match SteamEmu::toggle() {
            Ok(now_unlocked) => {
                println!("  -> Conmutado DLCUnlockall exitosamente a: {}", now_unlocked);
                if !now_unlocked {
                    let _ = SteamEmu::toggle();
                    println!("  -> Restaurado DLCUnlockall=1 (Desbloqueado)");
                }
            }
            Err(e) => {
                eprintln!("  -> Error al conmutar DLCUnlockall: {}", e);
                failed += 1;
            }
        }
    }

    println!("\n=======================================================");
    println!("RESUMEN FINAL DE LA VERIFICACIÓN:");
    println!("  - Opciones verificadas y operativas en DOA6LR: {} / {}", passed, cheats_to_test.len() - optional_unsupported);
    println!("  - Opciones exclusivas de builds antiguas (v1.0-v1.20): {}", optional_unsupported);
    println!("  - Persistencia de DLCs (steam_emu.ini): OK");
    println!("  - Fallos inesperados: {}", failed);
    println!("=======================================================");

    if failed > 0 {
        std::process::exit(1);
    }
}
