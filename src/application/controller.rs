//! Punto de entrada para la interfaz: estado compartido y un hilo que detecta el
//! juego y atiende los atajos de teclado aunque la ventana no tenga el foco.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::error::TrainerError;
use super::ports::{HotkeyEvent, HotkeyListener, ProcessFinder};
use super::trainer::Trainer;
use crate::domain::cheat::{CheatId, GameCatalog};

const HOTKEY_POLL_INTERVAL: Duration = Duration::from_millis(25);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Lo que la interfaz necesita para dibujarse.
#[derive(Debug, Clone, Default)]
pub struct TrainerSnapshot {
    pub game_pid: Option<u32>,
    pub active: HashSet<CheatId>,
    pub hotkeys_enabled: bool,
    pub last_error: Option<TrainerError>,
}

struct State {
    trainer: Trainer,
    hotkeys_enabled: bool,
    last_error: Option<TrainerError>,
}

impl State {
    fn toggle(&mut self, id: CheatId) {
        self.last_error = self.trainer.toggle(id).err();
    }

    fn poll_process(&mut self, finder: &dyn ProcessFinder) -> bool {
        let was_attached = self.trainer.game_pid().is_some();
        if self.trainer.refresh() {
            return false;
        }
        match finder.find(self.trainer.catalog().process_name) {
            Ok(Some(process)) => {
                self.trainer.attach(process);
                self.last_error = None;
            }
            Ok(None) => {}
            Err(error) => self.last_error = Some(TrainerError::Memory(error)),
        }
        was_attached != self.trainer.game_pid().is_some()
    }

    fn handle_hotkey(&mut self, event: HotkeyEvent) {
        match event {
            HotkeyEvent::ToggleHotkeys => self.hotkeys_enabled = !self.hotkeys_enabled,
            HotkeyEvent::Cheat(hotkey) if self.hotkeys_enabled => {
                if let Some(id) = self.trainer.catalog().cheat_for_hotkey(hotkey).map(|cheat| cheat.id) {
                    self.toggle(id);
                }
            }
            HotkeyEvent::Cheat(_) => {}
        }
    }
}

pub struct Controller {
    state: Arc<Mutex<State>>,
    catalog: GameCatalog,
    stop: Arc<AtomicBool>,
    monitor: Option<JoinHandle<()>>,
}

impl Controller {
    pub fn new(catalog: GameCatalog) -> Self {
        let state = State { trainer: Trainer::new(catalog.clone()), hotkeys_enabled: true, last_error: None };
        Self { state: Arc::new(Mutex::new(state)), catalog, stop: Arc::new(AtomicBool::new(false)), monitor: None }
    }

    pub fn catalog(&self) -> &GameCatalog {
        &self.catalog
    }

    /// Arranca el hilo de fondo. `on_change` se llama cuando cambia algo visible.
    pub fn start(
        &mut self,
        finder: Box<dyn ProcessFinder>,
        mut hotkeys: Box<dyn HotkeyListener>,
        on_change: impl Fn() + Send + 'static,
    ) {
        let state = Arc::clone(&self.state);
        let stop = Arc::clone(&self.stop);
        self.monitor = Some(thread::spawn(move || {
            let mut next_process_poll = Instant::now();
            while !stop.load(Ordering::Relaxed) {
                let events = hotkeys.poll();
                let mut changed = !events.is_empty();
                {
                    let mut state = lock(&state);
                    for event in events {
                        state.handle_hotkey(event);
                    }
                    if Instant::now() >= next_process_poll {
                        changed |= state.poll_process(finder.as_ref());
                        next_process_poll = Instant::now() + PROCESS_POLL_INTERVAL;
                    }
                }
                if changed {
                    on_change();
                }
                thread::sleep(HOTKEY_POLL_INTERVAL);
            }
        }));
    }

    pub fn snapshot(&self) -> TrainerSnapshot {
        let state = lock(&self.state);
        TrainerSnapshot {
            game_pid: state.trainer.game_pid(),
            active: self.catalog.cheats.iter().map(|cheat| cheat.id).filter(|id| state.trainer.is_active(*id)).collect(),
            hotkeys_enabled: state.hotkeys_enabled,
            last_error: state.last_error.clone(),
        }
    }

    pub fn toggle(&self, id: CheatId) {
        lock(&self.state).toggle(id);
    }

    pub fn set_hotkeys_enabled(&self, enabled: bool) {
        lock(&self.state).hotkeys_enabled = enabled;
    }

    pub fn dismiss_error(&self) {
        lock(&self.state).last_error = None;
    }

    /// Detiene el hilo y deja el juego como estaba.
    pub fn shutdown(&mut self) -> Result<(), TrainerError> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(monitor) = self.monitor.take() {
            let _ = monitor.join();
        }
        lock(&self.state).trainer.detach()
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Si otro hilo entró en pánico con el candado, el estado sigue siendo utilizable.
fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::fake_memory::FakeMemory;
    use crate::application::ports::{GameProcess, MemoryError};
    use crate::domain::cheat::Hotkey;
    use crate::domain::doa6;

    struct GameAlwaysOpen;

    impl ProcessFinder for GameAlwaysOpen {
        fn find(&self, _process_name: &str) -> Result<Option<GameProcess>, MemoryError> {
            Ok(Some(GameProcess { pid: 7, memory: Box::new(FakeMemory::with_code(0x1000, &[0; 64])) }))
        }
    }

    fn state() -> State {
        State { trainer: Trainer::new(doa6::catalog()), hotkeys_enabled: true, last_error: None }
    }

    #[test]
    fn attaches_when_game_appears() {
        let mut state = state();
        assert!(state.poll_process(&GameAlwaysOpen));
        assert_eq!(state.trainer.game_pid(), Some(7));
        assert!(!state.poll_process(&GameAlwaysOpen));
    }

    #[test]
    fn hotkey_toggle_disables_cheat_hotkeys() {
        let mut state = state();
        state.handle_hotkey(HotkeyEvent::ToggleHotkeys);
        state.handle_hotkey(HotkeyEvent::Cheat(Hotkey::numpad(1)));
        assert!(!state.hotkeys_enabled);
        assert_eq!(state.last_error, None);
    }

    #[test]
    fn hotkey_without_game_reports_error() {
        let mut state = state();
        state.handle_hotkey(HotkeyEvent::Cheat(Hotkey::numpad(1)));
        assert_eq!(state.last_error, Some(TrainerError::GameNotRunning));
    }
}
