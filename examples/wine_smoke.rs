//! Prueba de integración contra `tools/wine-smoke/fake_doa6.c` (ver `tools/wine-smoke/run.sh`).
//!
//! Usa el núcleo y la infraestructura reales del trainer: busca `DOA6.exe`, activa cada
//! truco, comprueba su efecto leyendo la memoria del proceso falso y verifica que al
//! desactivarlo (o al soltar el proceso) todo vuelve al estado original.

use std::collections::HashMap;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

use doa6_trainer::application::ports::GameMemory;
use doa6_trainer::application::trainer::{CheatOutcome, Trainer};
use doa6_trainer::domain::cheat::CheatId;
use doa6_trainer::domain::doa6;
use doa6_trainer::infrastructure::default_process_finder;

const STATE_FILE: &str = "fake_doa6_state.txt";
const MAX_MONEY: i32 = 99_999_999;

/// Índices del struct `Observed` de fake_doa6.c (todos int32).
#[derive(Clone, Copy, Debug)]
enum Field {
    Ticks,
    PlayerHp,
    EnemyHpAfterHit,
    PlayerHpAfterHit,
    PlayerGauge,
    EnemyGauge,
    Speed,
    ChallengeOk,
    ChallengeProgress,
    Timer,
    Rate,
    RateField,
    ScoreTotal,
    ScoreSlot,
    Total,
    Money,
}

const FIELD_COUNT: usize = 16;

fn f32_bits(value: f32) -> i32 {
    value.to_bits() as i32
}

fn baseline() -> Vec<(Field, i32)> {
    vec![
        (Field::PlayerHp, 100),
        (Field::EnemyHpAfterHit, 490),
        (Field::PlayerHpAfterHit, 490),
        (Field::PlayerGauge, 20),
        (Field::EnemyGauge, 20),
        (Field::Speed, f32_bits(1.0)),
        (Field::ChallengeOk, 0),
        (Field::ChallengeProgress, 3),
        (Field::Timer, f32_bits(9.5)),
        (Field::Rate, f32_bits(1.0)),
        (Field::RateField, f32_bits(1.0)),
        (Field::ScoreTotal, 10),
        (Field::ScoreSlot, 0),
        (Field::Total, 8),
    ]
}

struct Probe {
    memory: Box<dyn GameMemory>,
    observed: u64,
    addresses: HashMap<String, u64>,
}

impl Probe {
    fn snapshot(&self) -> [i32; FIELD_COUNT] {
        let mut bytes = [0u8; FIELD_COUNT * 4];
        self.memory.read(self.observed, &mut bytes).expect("leer Observed");
        std::array::from_fn(|i| i32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap()))
    }

    /// Espera unos ticks del juego para que el cambio se haya ejecutado.
    fn observe(&self) -> Result<[i32; FIELD_COUNT], String> {
        let start = self.snapshot()[Field::Ticks as usize];
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let now = self.snapshot();
            if now[Field::Ticks as usize] >= start + 3 {
                return Ok(now);
            }
            if Instant::now() > deadline {
                return Err("el juego falso dejó de avanzar (¿se colgó o cerró?)".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    fn bytes(&self, site: &str, offset: u64, len: usize) -> Vec<u8> {
        let mut buffer = vec![0; len];
        self.memory.read(self.addresses[site] + offset, &mut buffer).expect("leer sitio");
        buffer
    }
}

struct Site {
    name: &'static str,
    offset: u64,
    original: &'static [u8],
    patched: &'static [u8],
}

const UNLOCK_FLAG: &[u8] = &[0xBA, 0x01, 0x00, 0x00, 0x00, 0x90, 0x90, 0x90];

fn sites_of(id: &str) -> Vec<Site> {
    let site = |name, offset, original, patched| Site { name, offset, original, patched };
    match id {
        "3" => vec![site("costumes", 0, &[0x48, 0x03, 0xC3, 0xF6, 0x00, 0x1A][..], &[0x48, 0x85, 0xC0, 0x90, 0x90, 0x90][..])],
        "C2" => vec![
            site("hair", 8, &[0xF6, 0x00, 0x0A][..], &[0x48, 0x85, 0xC0][..]),
            site("hair_menu", 12, &[0xF6, 0x04, 0x01, 0x0A][..], &[0x48, 0x85, 0xC0, 0x90][..]),
        ],
        "C3" => vec![site("glasses", 6, &[0xF6, 0x00, 0x0A][..], &[0x48, 0x85, 0xC0][..])],
        "C4" => vec![site("music", 0, &[0x0F, 0xB6, 0x94, 0x08, 0x66, 0x13, 0x00, 0x00][..], UNLOCK_FLAG)],
        "C5" => vec![site("encyclopedia", 0, &[0x0F, 0xB6, 0x94, 0x08, 0xC0, 0x15, 0x00, 0x00][..], UNLOCK_FLAG)],
        "C6" => vec![site("trivia", 0, &[0x0F, 0xB6, 0x94, 0x08, 0x93, 0x14, 0x00, 0x00][..], UNLOCK_FLAG)],
        _ => vec![],
    }
}

/// Efecto esperado de cada truco sobre `Observed` (además de los sitios parcheados).
fn effects_of(id: &str) -> Vec<(Field, i32)> {
    match id {
        "1" => vec![(Field::PlayerHp, 25_000)],
        "2" => vec![(Field::PlayerGauge, 1000)],
        "4" => vec![(Field::ChallengeOk, 1), (Field::ChallengeProgress, 10)],
        "5" => vec![
            (Field::Timer, f32_bits(10.0)),
            (Field::Rate, 0x38D1_B717),
            (Field::RateField, 0x38D1_B717),
        ],
        "6" => vec![(Field::ScoreTotal, 0), (Field::ScoreSlot, MAX_MONEY), (Field::Total, MAX_MONEY)],
        "7" => vec![(Field::Speed, f32_bits(2.5))],
        "8" => vec![(Field::Speed, f32_bits(0.25))],
        "9" => vec![(Field::EnemyGauge, 0)],
        "0" => vec![(Field::EnemyHpAfterHit, 0)],
        _ => vec![],
    }
}

/// `baseline` con los cambios de `effects` aplicados encima.
fn expected(effects: &[(Field, i32)]) -> Vec<(Field, i32)> {
    let mut values = baseline();
    for (field, value) in effects {
        if let Some(entry) = values.iter_mut().find(|(f, _)| *f as usize == *field as usize) {
            entry.1 = *value;
        }
    }
    values
}

struct Report {
    failures: Vec<String>,
    checks: usize,
}

impl Report {
    fn check(&mut self, context: &str, ok: bool, detail: impl FnOnce() -> String) {
        self.checks += 1;
        if !ok {
            let message = format!("{context}: {}", detail());
            println!("    FALLO {message}");
            self.failures.push(message);
        }
    }

    fn check_observed(&mut self, context: &str, now: &[i32; FIELD_COUNT], expected: &[(Field, i32)]) {
        for (field, value) in expected {
            let actual = now[*field as usize];
            self.check(context, actual == *value, || format!("{field:?} = {actual:#X}, esperado {value:#X}"));
        }
    }

    fn check_sites(&mut self, context: &str, probe: &Probe, id: &str, patched: bool) {
        for site in sites_of(id) {
            let want = if patched { site.patched } else { site.original };
            let actual = probe.bytes(site.name, site.offset, want.len());
            self.check(context, actual == want, || format!("{} = {actual:02X?}, esperado {want:02X?}", site.name));
        }
    }
}

fn read_state_file() -> HashMap<String, u64> {
    let text = std::fs::read_to_string(STATE_FILE).expect("leer fake_doa6_state.txt");
    text.lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| {
            let hex = value.trim().trim_start_matches("0x").trim_start_matches("0X");
            (key.to_owned(), u64::from_str_radix(hex, 16).expect("dirección hex"))
        })
        .collect()
}

fn main() -> ExitCode {
    let catalog = doa6::catalog();
    let finder = default_process_finder();
    let find = || finder.find(catalog.process_name).expect("acceso al proceso");

    let Some(process) = find() else {
        println!("No se encontró {}", catalog.process_name);
        return ExitCode::FAILURE;
    };
    println!("Conectado a {} (PID {})", catalog.process_name, process.pid);
    let addresses = read_state_file();
    let probe = Probe { memory: find().unwrap().memory, observed: addresses["observed"], addresses };

    let mut trainer = Trainer::new(catalog.clone());
    trainer.attach(process);
    let mut report = Report { failures: Vec::new(), checks: 0 };

    let now = probe.observe().expect("el juego falso no avanza");
    report.check_observed("estado inicial", &now, &baseline());
    report.check("estado inicial", now[Field::Money as usize] == 500, || "dinero inicial".into());

    let toggles: Vec<CheatId> = catalog.cheats.iter().map(|cheat| cheat.id).filter(|id| id.0 != "C1").collect();
    for id in &toggles {
        let name = catalog.cheat(*id).unwrap().name.es;
        println!("[{}] {name}", id.0);

        let outcome = trainer.toggle(*id);
        report.check(id.0, outcome == Ok(CheatOutcome::Enabled), || format!("activar: {outcome:?}"));
        match probe.observe() {
            Ok(now) => report.check_observed(&format!("{} activo", id.0), &now, &expected(&effects_of(id.0))),
            Err(error) => report.check(id.0, false, || error),
        }
        report.check_sites(&format!("{} activo", id.0), &probe, id.0, true);

        let outcome = trainer.toggle(*id);
        report.check(id.0, outcome == Ok(CheatOutcome::Disabled), || format!("desactivar: {outcome:?}"));
        match probe.observe() {
            Ok(now) => report.check_observed(&format!("{} inactivo", id.0), &now, &baseline()),
            Err(error) => report.check(id.0, false, || error),
        }
        report.check_sites(&format!("{} inactivo", id.0), &probe, id.0, false);
    }

    println!("[7→8] supervelocidad y cámara lenta se excluyen");
    trainer.toggle(CheatId("7")).ok();
    trainer.toggle(CheatId("8")).ok();
    let now = probe.observe().expect("el juego falso no avanza");
    report.check_observed("7→8", &now, &expected(&effects_of("8")));
    report.check("7→8", !trainer.is_active(CheatId("7")) && trainer.is_active(CheatId("8")), || {
        "estado de los botones".into()
    });
    trainer.toggle(CheatId("8")).ok();

    println!("[C1] Dinero infinito");
    let outcome = trainer.toggle(CheatId("C1"));
    report.check("C1", outcome == Ok(CheatOutcome::Applied), || format!("{outcome:?}"));
    let now = probe.observe().expect("el juego falso no avanza");
    report.check("C1", now[Field::Money as usize] == MAX_MONEY, || format!("dinero = {}", now[Field::Money as usize]));

    println!("[todo] activar todos los trucos a la vez y soltar el proceso");
    for id in &toggles {
        if !trainer.is_active(*id) && *id != CheatId("8") {
            trainer.toggle(*id).ok();
        }
    }
    let mut all_effects: Vec<(Field, i32)> = toggles.iter().filter(|id| id.0 != "8").flat_map(|id| effects_of(id.0)).collect();
    // Con K.O. y vida infinita a la vez, el jugador sigue intacto y el rival cae.
    all_effects.push((Field::PlayerHpAfterHit, 490));
    let now = probe.observe().expect("el juego falso no avanza");
    report.check_observed("todos activos", &now, &expected(&all_effects));

    let detached = trainer.detach();
    report.check("soltar", detached.is_ok(), || format!("{detached:?}"));
    match probe.observe() {
        Ok(now) => report.check_observed("tras soltar", &now, &baseline()),
        Err(error) => report.check("tras soltar", false, || error),
    }
    for id in &toggles {
        report.check_sites("tras soltar", &probe, id.0, false);
    }
    thread::sleep(Duration::from_millis(300));
    report.check("tras soltar", probe.observe().is_ok(), || "el juego falso se cerró".into());

    println!();
    if report.failures.is_empty() {
        println!("OK: {} comprobaciones superadas", report.checks);
        ExitCode::SUCCESS
    } else {
        println!("{} de {} comprobaciones fallaron", report.failures.len(), report.checks);
        ExitCode::FAILURE
    }
}
