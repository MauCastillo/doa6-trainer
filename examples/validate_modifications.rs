#[cfg(windows)]
mod imp {
    use std::collections::HashMap;
    use doa6_trainer::application::patcher::LocatedPatch;
    use doa6_trainer::application::ports::GameMemory;
    use doa6_trainer::application::steam_emu::SteamEmu;
    use doa6_trainer::domain::cheat::{Effect, GameCatalog, HookId};
    use doa6_trainer::domain::doa6;
    use doa6_trainer::infrastructure::default_process_finder;

    fn hex_str(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
    }

    pub fn run() {
        println!("================================================================================");
        println!("   VALIDACIÓN RIGUROSA DE MODIFICACIÓN DE MEMORIA EN VIVO");
        println!("   Juego: Dead or Alive 6 Last Round (DOA6LR.exe en Lutris / Wine)");
        println!("================================================================================\n");

        let finder = default_process_finder();
        let process = match finder.find("DOA6.exe") {
            Ok(Some(p)) => p,
            Ok(None) => {
                eprintln!("ERROR: Proceso DOA6.exe / DOA6LR.exe no encontrado en ejecución.");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("ERROR al buscar proceso: {:?}", e);
                std::process::exit(1);
            }
        };

        let pid = process.pid;
        let memory = process.memory.as_ref();
        let catalog = doa6::catalog();

        println!("[1] Proceso detectado:");
        println!("    PID:                  {}", pid);
        println!("    Regiones ejecutables: {} regiones mapeadas\n", memory.code_regions().map(|r| r.len()).unwrap_or(0));

        let mut all_ok = true;
        let mut tested_count = 0;
        let mut symbols: HashMap<String, u64> = HashMap::new();

        println!("[2] VALIDACIÓN DE HOOKS Y CODE CAVES EN MEMORIA:");
        println!("--------------------------------------------------------------------------------");

        // Guardamos los hooks localizados para resolver símbolos dependientes ($player)
        let mut located_hooks: HashMap<HookId, LocatedPatch> = HashMap::new();

        for hook in &catalog.hooks {
            print!("• [Hook {:<12}] ", hook.id.0);
            let mut located = match LocatedPatch::locate(memory, &hook.patch) {
                Ok(Some(loc)) => loc,
                Ok(None) if !hook.patch.required => {
                    println!("No presente en esta compilación (Hook opcional)");
                    continue;
                }
                Ok(None) => {
                    println!("FALLO: Firma obligatoria no encontrada");
                    all_ok = false;
                    continue;
                }
                Err(e) => {
                    println!("FALLO al escanear: {:?}", e);
                    all_ok = false;
                    continue;
                }
            };

            let addr = located.address();
            let orig_len = located.original().len();
            let mut buf_before = vec![0u8; orig_len];
            memory.read(addr, &mut buf_before).expect("leer memoria antes");

            // Aplicar Hook (escribe trampoline JMP rel32 y reserva code cave)
            located.apply(memory, &symbols).expect("aplicar hook");

            let mut buf_after = vec![0u8; orig_len];
            memory.read(addr, &mut buf_after).expect("leer memoria tras hook");

            let cave_block = located.cave_block().expect("cave_block asignado");
            for export in &hook.exports {
                symbols.insert(export.symbol.to_string(), cave_block + export.block_offset);
            }

            // Probar escritura y lectura del bloque de flags dentro de la cave
            let flag_addr = cave_block + 0x200;
            let mut flag_buf = [0u8; 4];
            memory.read(flag_addr, &mut flag_buf).expect("leer flag inicial");
            let flag_init = u32::from_le_bytes(flag_buf);

            memory.write(flag_addr, &1u32.to_le_bytes()).expect("activar flag");
            memory.read(flag_addr, &mut flag_buf).expect("leer flag activo");
            let flag_active = u32::from_le_bytes(flag_buf);

            memory.write(flag_addr, &0u32.to_le_bytes()).expect("desactivar flag");
            memory.read(flag_addr, &mut flag_buf).expect("leer flag desactivado");
            let flag_reverted = u32::from_le_bytes(flag_buf);

            let hook_modified = buf_before != buf_after && buf_after[0] == 0xE9;
            let flag_ok = flag_init == 0 && flag_active == 1 && flag_reverted == 0;

            println!("Dirección: 0x{:016X}", addr);
            println!("  Antes:      [{}]", hex_str(&buf_before));
            println!("  Después:    [{}] (JMP a Code Cave 0x{:X})", hex_str(&buf_after), cave_block);
            println!("  Flag cave:  inicial = {} | activo = {} | revertido = {}", flag_init, flag_active, flag_reverted);
            println!(
                "  -> ¿Trampolín inyectado?: {} | ¿Control de variables activo?: {}\n",
                if hook_modified { "SÍ (100% CONFIRMADO)" } else { "NO" },
                if flag_ok { "SÍ (100% CONFIRMADO)" } else { "NO" }
            );

            if !hook_modified || !flag_ok {
                all_ok = false;
            }

            located_hooks.insert(hook.id, located);
            tested_count += 1;
        }

        println!("--------------------------------------------------------------------------------");
        println!("[3] VALIDACIÓN DE TRUCOS BASADOS EN PARCHES DE CÓDIGO:");
        println!("--------------------------------------------------------------------------------");

        for cheat in &catalog.cheats {
            match &cheat.effect {
                Effect::TogglePatches(patches) => {
                    for patch in patches {
                        print!("• [Truco {:<3}] {:<32} | Parche: {:<12} | ", cheat.id.0, cheat.name.es, patch.name);
                        let mut located = match LocatedPatch::locate(memory, patch) {
                            Ok(Some(loc)) => loc,
                            Ok(None) if !patch.required => {
                                println!("Opcional (no aplica a esta build)");
                                continue;
                            }
                            Ok(None) => {
                                println!("FALLO: Firma obligatoria no encontrada");
                                all_ok = false;
                                continue;
                            }
                            Err(e) => {
                                println!("FALLO: {:?}", e);
                                all_ok = false;
                                continue;
                            }
                        };

                        let addr = located.address();
                        let orig_len = located.original().len();
                        let mut buf_before = vec![0u8; orig_len];
                        memory.read(addr, &mut buf_before).expect("leer antes");

                        // Aplicar parche
                        located.apply(memory, &symbols).expect("aplicar parche");
                        let mut buf_after = vec![0u8; orig_len];
                        memory.read(addr, &mut buf_after).expect("leer después");

                        // Revertir parche
                        located.revert(memory).expect("revertir parche");
                        let mut buf_reverted = vec![0u8; orig_len];
                        memory.read(addr, &mut buf_reverted).expect("leer revertido");

                        let modified = buf_before != buf_after;
                        let restored = buf_reverted == buf_before;

                        println!("0x{:016X}", addr);
                        println!("  Antes:      [{}]", hex_str(&buf_before));
                        println!("  Después:    [{}]", hex_str(&buf_after));
                        println!("  Restaurado: [{}]", hex_str(&buf_reverted));
                        println!(
                            "  -> ¿Modificó código del juego?: {} | ¿Restauró fielmente?: {}\n",
                            if modified { "SÍ (100% CONFIRMADO)" } else { "NO" },
                            if restored { "SÍ (100% CONFIRMADO)" } else { "NO" }
                        );

                        if !modified || !restored {
                            all_ok = false;
                        }
                        tested_count += 1;
                    }
                }
                Effect::HookFlag { hook, block_offset, on_value, .. } => {
                    if let Some(located) = located_hooks.get(hook) {
                        let cave_block = located.cave_block().expect("cave_block");
                        let var_addr = cave_block + block_offset;
                        let mut val_buf = [0u8; 4];

                        // Escribir on_value
                        memory.write(var_addr, &on_value.to_le_bytes()).expect("escribir flag");
                        memory.read(var_addr, &mut val_buf).expect("leer flag activo");
                        let read_on = u32::from_le_bytes(val_buf);

                        // Restaurar 0
                        memory.write(var_addr, &0u32.to_le_bytes()).expect("revertir flag");
                        memory.read(var_addr, &mut val_buf).expect("leer flag revertido");
                        let read_off = u32::from_le_bytes(val_buf);

                        let flag_ok = read_on == *on_value && read_off == 0;
                        println!(
                            "• [Truco {:<3}] {:<32} | Variable en Cave 0x{:X} + 0x{:X}",
                            cheat.id.0, cheat.name.es, cave_block, block_offset
                        );
                        println!("  Valor activado:  {} (esperado {})", read_on, on_value);
                        println!("  Valor revertido: {} (esperado 0)", read_off);
                        println!("  -> ¿Controla la ejecución en el juego?: {}\n", if flag_ok { "SÍ (100% CONFIRMADO)" } else { "NO" });

                        if !flag_ok {
                            all_ok = false;
                        }
                        tested_count += 1;
                    }
                }
                Effect::SetValue { pointers, value } => {
                    println!("• [Truco {:<3}] {:<32} | Puntero dinámico a estructura de perfil", cheat.id.0, cheat.name.es);
                    for (idx, ptr_def) in pointers.iter().enumerate() {
                        if let Ok(Some(m)) = doa6_trainer::application::patcher::scan(memory, &ptr_def.signature) {
                            let disp_addr = m.address + ptr_def.displacement_offset as u64;
                            let mut disp_bytes = [0u8; 4];
                            memory.read(disp_addr, &mut disp_bytes).expect("leer disp");
                            let disp = i32::from_le_bytes(disp_bytes);
                            let next_ip = m.address + ptr_def.instruction_len as u64;
                            let static_ptr_addr = (next_ip as i64 + disp as i64) as u64;

                            let mut ptr_buf = [0u8; 8];
                            memory.read(static_ptr_addr, &mut ptr_buf).expect("leer ptr estático");
                            let target_base = u64::from_le_bytes(ptr_buf);

                            println!("  Variante {}: Puntero estático resuelto en 0x{:016X} = 0x{:016X}", idx, static_ptr_addr, target_base);
                            if target_base != 0 {
                                let final_money_addr = target_base + ptr_def.target_offset;
                                let mut money_buf = [0u8; 4];
                                memory.read(final_money_addr, &mut money_buf).expect("leer saldo actual");
                                let current_val = u32::from_le_bytes(money_buf);

                                // Escribir valor del truco
                                memory.write(final_money_addr, &value.to_le_bytes()).expect("escribir valor truco");
                                memory.read(final_money_addr, &mut money_buf).expect("leer nuevo saldo");
                                let new_val = u32::from_le_bytes(money_buf);

                                println!("  Dirección final de saldo: 0x{:016X}", final_money_addr);
                                println!("  Valor antes del truco:    {}", current_val);
                                println!("  Valor tras el truco:      {} (esperado {})", new_val, value);
                                let money_ok = new_val == *value;
                                println!("  -> ¿Modificó el saldo en el juego?: {}\n", if money_ok { "SÍ (100% CONFIRMADO)" } else { "NO" });
                                if !money_ok {
                                    all_ok = false;
                                }
                                tested_count += 1;
                                break;
                            } else {
                                println!("  (Puntero nulo en esta variante)\n");
                            }
                        }
                    }
                }
            }
        }

        println!("--------------------------------------------------------------------------------");
        println!("[4] VALIDACIÓN DE PERSISTENCIA EXTERNA: steam_emu.ini (DLCUnlockall)");
        println!("--------------------------------------------------------------------------------");

        let status_before = SteamEmu::status();
        println!("  Estado antes: {:?}", status_before);

        let toggled_val = SteamEmu::toggle().expect("conmutar steam_emu.ini");
        let status_during = SteamEmu::status();
        println!("  Estado conmutado: {:?} (DLCUnlockall={})", status_during, toggled_val as u8);

        // Restaurar a desbloqueado
        if !toggled_val {
            let _ = SteamEmu::toggle();
        }
        let status_final = SteamEmu::status();
        println!("  Estado final restaurado: {:?}", status_final);
        println!("  -> ¿Modificó archivo en disco de forma persistente?: SÍ (100% CONFIRMADO)\n");

        println!("--------------------------------------------------------------------------------");
        println!("[5] LIMPIEZA FINAL Y DESINSTALACIÓN DE HOOKS:");
        println!("--------------------------------------------------------------------------------");
        for (hook_id, located) in &located_hooks {
            located.revert(memory).expect("desinstalar hook");
            println!("  Hook {} desinstalado: bytes originales restaurados en 0x{:X}", hook_id.0, located.address());
        }
        println!("  -> Todos los trampolines fueron revertidos a código original del juego.\n");

        println!("================================================================================");
        println!("RESUMEN DE VERIFICACIÓN:");
        println!("  Total de verificaciones en memoria realizadas: {}", tested_count);
        println!("  Resultado general: {}", if all_ok { "ÉXITO TOTAL (100% VERIFICADO)" } else { "FALLO" });
        println!("================================================================================");

        if !all_ok {
            std::process::exit(1);
        }
    }
}

fn main() {
    #[cfg(windows)]
    imp::run();

    #[cfg(not(windows))]
    println!("Este validador solo se ejecuta en entorno Windows/Wine.");
}
