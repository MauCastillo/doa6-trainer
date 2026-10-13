//! Catálogo de Dead or Alive 6 (v1.0-v1.20), extraído del trainer de FLiNG.

use super::cheat::{
    Cheat, CheatId, Effect, GameCatalog, Hook, HookId, Hotkey, LocalizedText, Patch, PatchBody, PatchVariant,
    RipRelativePointer, SymbolExport, cave_layout,
};
use super::pattern::{Signature, Template};

pub const PLAYER: HookId = HookId("player");
pub const ONE_HIT_KO: HookId = HookId("one_hit_ko");
pub const BREAK_GAUGE: HookId = HookId("break_gauge");
pub const GAME_SPEED: HookId = HookId("game_speed");

const FLAG_ON: u32 = 1;
const SPEED_FAST: u32 = 2.5f32.to_bits();
const SPEED_SLOW: u32 = 0.25f32.to_bits();
const MAX_MONEY: u32 = 99_999_999;

pub fn catalog() -> GameCatalog {
    GameCatalog {
        title: "Dead or Alive 6",
        supported_versions: "v1.0 - v1.20",
        process_name: "DOA6.exe",
        hooks: hooks(),
        cheats: cheats(),
    }
}

fn hooks() -> Vec<Hook> {
    vec![
        Hook {
            id: PLAYER,
            requires: vec![],
            // Guarda el puntero del jugador y, con el flag activo, rellena su vida.
            patch: required(
                "player",
                vec![cave(
                    "66 83 B8 s1 00 00 14 77 * 8B 03 3D * * 00 00 74 * 3D",
                    0,
                    "66 83 B8 s1 00 00 14",
                    "50 8F 05 F9 03 00 00 83 3D F2 01 00 00 01 75 09 66 C7 80 s1 00 00 A8 61 66 83 B8 s1 00 00 14",
                )],
            ),
            exports: vec![SymbolExport { symbol: "player", block_offset: cave_layout::DATA }],
        },
        Hook {
            id: ONE_HIT_KO,
            requires: vec![PLAYER],
            patch: required(
                "one_hit_ko",
                vec![
                    // DOA6 v1.0 - v1.20
                    cave(
                        "66 2B CA 66 3B C1 66 0F 42 C8 66 89 8E * * 00 00",
                        0,
                        "66 2B CA 66 3B C1",
                        "83 3D F9 01 00 00 01 75 1A 50 48 A1 $player 48 39 F0 74 09 66 83 FA 00 7E 03 66 89 D1 58 66 29 D1 66 39 C8",
                    ),
                    // DOA6 Last Round (DOA6LR.exe)
                    cave(
                        "0F B7 C7 66 2B C1 0F B7 8D 82 05 00 00 66 89 85 80 05 00 00",
                        0,
                        "0F B7 C7 66 2B C1",
                        "0F B7 C7 66 2B C1 83 3D F3 01 00 00 01 75 18 51 89 C1 48 A1 $player 48 39 C5 89 C8 59 74 03 66 31 C0",
                    ),
                ],
            ),
            exports: vec![],
        },
        Hook {
            id: BREAK_GAUGE,
            requires: vec![PLAYER],
            // Flag +0x200: llenar la barra del jugador. Flag +0x204: vaciar la del rival.
            patch: required(
                "break_gauge",
                vec![
                    // DOA6 v1.0 - v1.20
                    cave(
                        "83 7F 18 02 89 4F 20 75 * 85 * 7E",
                        0,
                        "83 7F 18 02 89 4F 20",
                        "50 48 A1 $player 48 39 47 08 75 13 83 3D E8 01 00 00 01 75 15 3B 4F 20 7E 10 8B 4F 1C EB 0B \
                         83 3D D9 01 00 00 01 75 02 31 C9 89 4F 20 58 83 7F 18 02",
                    ),
                    // DOA6 Last Round (DOA6LR.exe)
                    cave(
                        "48 8B 46 08 89 88 B4 05 00 00 48 8B 46 08 38 56",
                        0,
                        "48 8B 46 08 89 88 B4 05 00 00",
                        "48 8B 46 08 50 48 A1 $player 48 39 46 08 58 75 11 83 3D E3 01 00 00 01 75 16 8B 4E 1C 89 4E 20 EB 0E 83 3D D6 01 00 00 01 75 05 31 C9 89 4E 20 89 88 B4 05 00 00",
                    ),
                ],
            ),
            exports: vec![],
        },
        Hook {
            id: GAME_SPEED,
            requires: vec![],
            // Si el flag es distinto de 0, se usa como multiplicador (float) de velocidad.
            patch: optional(
                "game_speed",
                vec![cave(
                    "41 8B 8E s1 00 00 41 89 * C7 44 24 * 01 00 00 00 * D2",
                    0,
                    "41 8B 8E s1 00 00",
                    "41 8B 8E s1 00 00 83 3D F2 01 00 00 00 74 06 8B 0D EA 01 00 00",
                )],
            ),
            exports: vec![],
        },
    ]
}

fn cheats() -> Vec<Cheat> {
    vec![
        Cheat {
            id: CheatId("1"),
            hotkey: Hotkey::numpad(1),
            name: text("Vida infinita", "Infinite Health"),
            effect: hook_flag(PLAYER, 0, FLAG_ON, None),
        },
        Cheat {
            id: CheatId("2"),
            hotkey: Hotkey::numpad(2),
            name: text("Barra de Break al máximo", "Max Break Gauge"),
            effect: hook_flag(BREAK_GAUGE, 0, FLAG_ON, None),
        },
        Cheat {
            id: CheatId("3"),
            hotkey: Hotkey::numpad(3),
            name: text("Desbloquear todos los trajes", "Unlock All Costumes"),
            effect: Effect::TogglePatches(vec![required(
                "costumes",
                vec![
                    // DOA6 v1.0 - v1.20
                    replace(
                        "48 03 C3 F6 00 1A 0F 95 C0 48 83 C4 * * C3 48 05 20 4E 00 00 F6 00 1A",
                        0,
                        "48 03 C3 F6 00 1A",
                        "48 85 C0 90 90 90",
                    ),
                    replace(
                        "48 03 C3 F6 00 0A 0F 95 C0 48 83 C4 * * C3 48 05 20 4E 00 00 F6 00 0A",
                        0,
                        "48 03 C3 F6 00 0A",
                        "48 85 C0 90 90 90",
                    ),
                    // DOA6 Last Round (DOA6LR.exe)
                    replace(
                        "B8 20 4E 00 00 48 3B C8 48 0F 43 C8 F6 04 19 1A 48 8B 5C 24 30 0F 95 C0",
                        21,
                        "0F 95 C0",
                        "B0 01 90",
                    ),
                    replace(
                        "B8 20 4E 00 00 48 3B C8 48 0F 43 C8 F6 04 19 1B 48 8B 5C 24 30 0F 95 C0",
                        21,
                        "0F 95 C0",
                        "B0 01 90",
                    ),
                ],
            )]),
        },
        Cheat {
            id: CheatId("4"),
            hotkey: Hotkey::numpad(4),
            name: text("Completar siempre los desafíos", "Always Clear Challenges"),
            effect: Effect::TogglePatches(vec![required(
                "challenges",
                vec![
                    cave(
                        "39 86 s1 00 00 0F 9D C0 EB * 32 C0 4C * * 24",
                        0,
                        "39 86 s1 00 00 0F 9D C0",
                        "89 86 s1 00 00 B0 01",
                    ),
                    cave(
                        "39 86 s1 00 00 0F 9D C0 EB * * C0 48 8B * 24",
                        0,
                        "39 86 s1 00 00 0F 9D C0",
                        "89 86 s1 00 00 B0 01",
                    ),
                ],
            )]),
        },
        Cheat {
            id: CheatId("5"),
            hotkey: Hotkey::numpad(5),
            name: text("Congelar el tiempo", "Freeze Timer"),
            effect: Effect::TogglePatches(vec![
                required(
                    "timer_tick",
                    vec![
                        cave(
                            "F3 0F 58 46 08 F3 0F 11 46 08 0F B6 * * * 00 00 66 0F 6E",
                            0,
                            "F3 0F 58 46 08",
                            "83 7E 08 00 74 03 0F 57 C0 F3 0F 58 46 08",
                        ),
                        cave(
                            "F3 0F 58 47 08 F3 0F 11 47 08 0F B6 * * * 00 00 66 0F 6E",
                            0,
                            "F3 0F 58 47 08",
                            "83 7F 08 00 74 03 0F 57 C0 F3 0F 58 47 08",
                        ),
                    ],
                ),
                optional(
                    "timer_rate",
                    vec![cave(
                        "F3 0F 10 49 14 F3 48 0F 2A C0 F3 0F 5E C8 0F 57 C0 F3 0F 58 8B * * 00 00",
                        0,
                        "F3 0F 10 49 14",
                        "C7 41 14 17 B7 D1 38 F3 0F 10 49 14",
                    )],
                ),
            ]),
        },
        Cheat {
            id: CheatId("6"),
            hotkey: Hotkey::numpad(6),
            name: text("Puntuación máxima", "Max Score"),
            effect: Effect::TogglePatches(vec![
                required(
                    "score",
                    vec![cave(
                        "48 0F AF 83 s1 00 00 48 C7 83 * * 00 00 00 00 00 00 48 01 83 * * 00 00",
                        0,
                        "48 0F AF 83 s1 00 00",
                        "48 8D 83 s1 00 00 C7 40 08 FF E0 F5 05 48 31 C0",
                    )],
                ),
                optional(
                    "score_total",
                    vec![
                        cave(
                            "48 01 81 s1 00 00 48 C7 81 * * 00 00 00 00 00 00 49 * * 8B 41 10",
                            0,
                            "48 01 81 s1 00 00",
                            "C7 81 s1 00 00 FF E0 F5 05",
                        ),
                        cave(
                            "8B 40 08 48 01 83 30 01 00 00 48 8B 5C 24 30",
                            3,
                            "48 01 83 30 01 00 00",
                            "C7 83 30 01 00 00 FF E0 F5 05",
                        ),
                    ],
                ),
            ]),
        },
        Cheat {
            id: CheatId("7"),
            hotkey: Hotkey::numpad(7),
            name: text("Supervelocidad", "Super Speed"),
            effect: hook_flag(GAME_SPEED, 0, SPEED_FAST, Some(CheatId("8"))),
        },
        Cheat {
            id: CheatId("8"),
            hotkey: Hotkey::numpad(8),
            name: text("Cámara lenta", "Slow Motion"),
            effect: hook_flag(GAME_SPEED, 0, SPEED_SLOW, Some(CheatId("7"))),
        },
        Cheat {
            id: CheatId("9"),
            hotkey: Hotkey::numpad(9),
            name: text("Vaciar la barra de Break del rival", "Empty Opponent's Break Gauge"),
            effect: hook_flag(BREAK_GAUGE, 4, FLAG_ON, None),
        },
        Cheat {
            id: CheatId("0"),
            hotkey: Hotkey::numpad(0),
            name: text("K.O. de un golpe", "One Hit K.O."),
            effect: hook_flag(ONE_HIT_KO, 0, FLAG_ON, None),
        },
        Cheat {
            id: CheatId("C1"),
            hotkey: Hotkey::ctrl_numpad(1),
            name: text("Dinero infinito", "Infinite Money"),
            effect: Effect::SetValue {
                pointers: vec![
                    // DOA6 v1.0 - v1.20
                    RipRelativePointer {
                        signature: signature("48 8B 15 * * * * B8 * * 00 00 * 3B * 48 0F 42 * F6 84 02 * * * * 02"),
                        displacement_offset: 3,
                        instruction_len: 7,
                        target_offset: 0,
                    },
                    // DOA6 Last Round (DOA6LR.exe)
                    RipRelativePointer {
                        signature: signature("48 8B 05 * * * * C7 80 0C 7F 01 00 20 B2 17 03 48 8B 4C"),
                        displacement_offset: 3,
                        instruction_len: 7,
                        target_offset: 0x8CB8,
                    },
                ],
                value: MAX_MONEY,
            },
        },
        Cheat {
            id: CheatId("C2"),
            hotkey: Hotkey::ctrl_numpad(2),
            name: text("Desbloquear todos los peinados", "Unlock All Hairstyles"),
            effect: Effect::TogglePatches(vec![
                required(
                    "hairstyles",
                    vec![
                        replace("EB 06 48 05 E8 03 00 00 F6 00 0A", 8, "F6 00 0A", "48 85 C0"),
                        replace(
                            "B8 20 4E 00 00 48 3B C8 48 0F 43 C8 0F B6 84 39 69 17 00 00 C0 E8 02 24 01",
                            20,
                            "C0 E8 02 24 01",
                            "B0 01 90 90 90",
                        ),
                    ],
                ),
                optional(
                    "hairstyles_menu",
                    vec![
                        replace(
                            "B8 E8 03 00 00 * * * * * * * F6 04 01 0A 0F 84 * * 00 00",
                            12,
                            "F6 04 01 0A",
                            "48 85 C0 90",
                        ),
                        replace(
                            "B8 E8 03 00 00 48 3B C8 48 0F 43 C8 0F B6 84 39 8A 65 00 00 C0 E8 02 24 01",
                            20,
                            "C0 E8 02 24 01",
                            "B0 01 90 90 90",
                        ),
                    ],
                ),
            ]),
        },
        Cheat {
            id: CheatId("C3"),
            hotkey: Hotkey::ctrl_numpad(3),
            name: text("Desbloquear todas las gafas", "Unlock All Glasses"),
            effect: Effect::TogglePatches(vec![required(
                "glasses",
                vec![
                    replace("48 05 40 1F 00 00 F6 00 0A 75 * 40 84 * 75 * * C0", 6, "F6 00 0A", "48 85 C0"),
                    replace(
                        "B8 40 1F 00 00 48 3B C8 48 0F 43 C8 0F B6 84 39 73 69 00 00 C0 E8 02 24 01",
                        20,
                        "C0 E8 02 24 01",
                        "B0 01 90 90 90",
                    ),
                ],
            )]),
        },
        Cheat {
            id: CheatId("C4"),
            hotkey: Hotkey::ctrl_numpad(4),
            name: text("Desbloquear toda la música", "Unlock All Music"),
            effect: unlock_flag_patch("music", "66 13", "E2 13"),
        },
        Cheat {
            id: CheatId("C5"),
            hotkey: Hotkey::ctrl_numpad(5),
            name: text("Desbloquear toda la enciclopedia", "Unlock All Encyclopedia"),
            effect: unlock_flag_patch("encyclopedia", "C0 15", "3C 16"),
        },
        Cheat {
            id: CheatId("C6"),
            hotkey: Hotkey::ctrl_numpad(6),
            name: text("Desbloquear todas las curiosidades", "Unlock All Trivia"),
            effect: unlock_flag_patch("trivia", "93 14", "0F 15"),
        },
    ]
}

/// Permite leer siempre como "desbloqueado" el elemento consultado.
/// Soporta tanto DOA6 v1.0-v1.20 como DOA6 Last Round (DOA6LR.exe).
fn unlock_flag_patch(name: &'static str, v1_offset: &str, lr_offset: &str) -> Effect {
    let original = format!("0F B6 94 08 {v1_offset} 00 00");
    Effect::TogglePatches(vec![required(
        name,
        vec![
            // DOA6 v1.0 - v1.20
            replace(
                &format!("{original} 48 8B 4C 24 * 80 E2 01 * * * * E8"),
                0,
                &original,
                "BA 01 00 00 00 90 90 90",
            ),
            // DOA6 Last Round (DOA6LR.exe)
            replace(
                &format!("48 0F 43 C8 0F B6 84 39 {lr_offset} 00 00 24 01 48 83 C4 20"),
                12,
                "24 01",
                "B0 01",
            ),
        ],
    )])
}

fn hook_flag(hook: HookId, flag_index: u64, on_value: u32, exclusive_with: Option<CheatId>) -> Effect {
    Effect::HookFlag { hook, block_offset: cave_layout::FLAGS + flag_index, on_value, exclusive_with }
}

fn required(name: &'static str, variants: Vec<PatchVariant>) -> Patch {
    Patch { name, variants, required: true }
}

fn optional(name: &'static str, variants: Vec<PatchVariant>) -> Patch {
    Patch { name, variants, required: false }
}

fn cave(signature_text: &str, offset: usize, original: &str, code: &str) -> PatchVariant {
    PatchVariant {
        signature: signature(signature_text),
        offset,
        original: template(original),
        body: PatchBody::CodeCave(template(code)),
    }
}

fn replace(signature_text: &str, offset: usize, original: &str, replacement: &str) -> PatchVariant {
    PatchVariant {
        signature: signature(signature_text),
        offset,
        original: template(original),
        body: PatchBody::Replace(template(replacement)),
    }
}

fn text(es: &'static str, en: &'static str) -> LocalizedText {
    LocalizedText { es, en }
}

// Los datos del catálogo son constantes del programa: un error aquí es un bug,
// y los tests de este módulo lo detectan antes de compilar en release.
fn signature(text: &str) -> Signature {
    Signature::parse(text).unwrap_or_else(|error| panic!("firma inválida `{text}`: {error}"))
}

fn template(text: &str) -> Template {
    Template::parse(text).unwrap_or_else(|error| panic!("plantilla inválida `{text}`: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn all_patches(catalog: &GameCatalog) -> Vec<&Patch> {
        let from_hooks = catalog.hooks.iter().map(|hook| &hook.patch);
        let from_cheats = catalog.cheats.iter().flat_map(|cheat| match &cheat.effect {
            Effect::TogglePatches(patches) => patches.iter().collect(),
            _ => Vec::new(),
        });
        from_hooks.chain(from_cheats).collect()
    }

    #[test]
    fn has_the_sixteen_cheats_with_unique_ids_and_hotkeys() {
        let catalog = catalog();
        let ids: HashSet<_> = catalog.cheats.iter().map(|cheat| cheat.id).collect();
        let hotkeys: HashSet<_> = catalog.cheats.iter().map(|cheat| cheat.hotkey).collect();

        assert_eq!(catalog.cheats.len(), 16);
        assert_eq!(ids.len(), 16);
        assert_eq!(hotkeys.len(), 16);
    }

    #[test]
    fn original_bytes_agree_with_the_signature() {
        let catalog = catalog();
        for patch in all_patches(&catalog) {
            for variant in &patch.variants {
                let original = variant.original.render(&sample_captures(), &Default::default()).unwrap();
                for (index, byte) in original.iter().enumerate() {
                    if let Some(expected) = variant.signature.byte_at(variant.offset + index) {
                        assert_eq!(expected, *byte, "parche {} byte {index}", patch.name);
                    }
                }
                assert!(variant.offset + original.len() <= variant.signature.len(), "parche {}", patch.name);
            }
        }
    }

    #[test]
    fn replacements_keep_the_original_size_and_caves_fit_a_jump() {
        let catalog = catalog();
        for patch in all_patches(&catalog) {
            for variant in &patch.variants {
                match &variant.body {
                    PatchBody::Replace(bytes) => assert_eq!(bytes.len(), variant.original.len(), "{}", patch.name),
                    PatchBody::CodeCave(code) => {
                        assert!(variant.original.len() >= 5, "{}", patch.name);
                        assert!(code.len() + 14 <= cave_layout::CODE_CAPACITY, "{}", patch.name);
                    }
                }
            }
        }
    }

    #[test]
    fn hook_dependencies_and_flags_point_to_known_hooks() {
        let catalog = catalog();
        for hook in &catalog.hooks {
            for dependency in &hook.requires {
                assert!(catalog.hook(*dependency).is_some());
            }
        }
        for cheat in &catalog.cheats {
            if let Effect::HookFlag { hook, exclusive_with, .. } = &cheat.effect {
                assert!(catalog.hook(*hook).is_some());
                if let Some(other) = exclusive_with {
                    assert!(catalog.cheat(*other).is_some());
                }
            }
        }
    }

    fn sample_captures() -> crate::domain::pattern::Captures {
        crate::domain::pattern::Captures::from([("s1".to_owned(), vec![0, 0])])
    }
}
