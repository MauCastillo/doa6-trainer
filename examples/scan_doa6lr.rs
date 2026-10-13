use std::fs;
use std::time::Instant;
use doa6_trainer::domain::doa6;
use doa6_trainer::domain::cheat::Effect;
use doa6_trainer::domain::pattern::Signature;

fn find_all_matches(sig: &Signature, bytes: &[u8]) -> Vec<(usize, doa6_trainer::domain::pattern::Captures)> {
    let mut matches = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        if let Some(rel) = sig.find_in(&bytes[pos..]) {
            let abs_pos = pos + rel;
            let caps = sig.read_captures(&bytes[abs_pos..abs_pos + sig.len()]);
            matches.push((abs_pos, caps));
            pos = abs_pos + 1;
        } else {
            break;
        }
    }
    matches
}

fn main() {
    let path = "/home/wau/Games/dead-or-alive-the-last-round/drive_c/Program Files/DEAD OR ALIVE 6 Last Round/DOA6LR.exe";
    println!("Loading {}...", path);
    let t0 = Instant::now();
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error reading file: {e}");
            return;
        }
    };
    println!("Loaded {:.1} MB in {:?}", bytes.len() as f64 / 1_048_576.0, t0.elapsed());

    let catalog = doa6::catalog();

    println!("\n=================== HOOKS ===================");
    for hook in &catalog.hooks {
        println!("\n[Hook] {:?}", hook.id);
        let patch = &hook.patch;
        for (i, variant) in patch.variants.iter().enumerate() {
            let t = Instant::now();
            let matches = find_all_matches(&variant.signature, &bytes);
            println!("  Variant {}: found {} match(es) ({:?})", i, matches.len(), t.elapsed());
            for (offset, caps) in &matches {
                println!("    Offset: 0x{:08X}, Captures: {:?}", offset, caps);
            }
            if matches.is_empty() {
                println!("    FAILED: Signature not found: {}", variant.signature);
            }
        }
    }

    println!("\n=================== CHEATS ===================");
    for cheat in &catalog.cheats {
        let prefix = if cheat.hotkey.ctrl { "Ctrl+Num" } else { "Num" };
        println!("\n[Cheat {}] [{} {}] {} / {}", cheat.id.0, prefix, cheat.hotkey.numpad, cheat.name.es, cheat.name.en);
        match &cheat.effect {
            Effect::HookFlag { hook, block_offset, on_value, exclusive_with } => {
                println!("  Type: HookFlag (hook: {:?}, offset: {}, on_value: 0x{:X}, exclusive_with: {:?})",
                    hook, block_offset, on_value, exclusive_with);
            }
            Effect::TogglePatches(patches) => {
                for patch in patches {
                    for (i, variant) in patch.variants.iter().enumerate() {
                        let t = Instant::now();
                        let matches = find_all_matches(&variant.signature, &bytes);
                        println!("  Patch '{}' var {}: {} match(es) ({:?})", patch.name, i, matches.len(), t.elapsed());
                        for (offset, caps) in &matches {
                            println!("    Offset: 0x{:08X}, Captures: {:?}", offset, caps);
                        }
                        if matches.is_empty() {
                            println!("    FAILED: Signature not found: {}", variant.signature);
                        }
                    }
                }
            }
            Effect::SetValue { pointers, value } => {
                for (i, pointer) in pointers.iter().enumerate() {
                    let t = Instant::now();
                    let matches = find_all_matches(&pointer.signature, &bytes);
                    println!("  SetValue (val: {}) ptr {}: {} match(es) ({:?}) target_offset=0x{:X}",
                        value, i, matches.len(), t.elapsed(), pointer.target_offset);
                    for (offset, caps) in &matches {
                        println!("    Offset: 0x{:08X}, Captures: {:?}", offset, caps);
                    }
                    if matches.is_empty() {
                        println!("    Variant {}: Signature not found: {}", i, pointer.signature);
                    }
                }
            }
        }
    }
}
