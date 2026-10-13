# Changelog

**English** · [Español](CHANGELOG.es.md)

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Language selector (English / Español):** the whole interface, help texts and error messages are available in both languages. The choice is saved in `settings.ini` under `%APPDATA%\doa6-trainer` (Windows / Wine) or `~/.config/doa6-trainer` (Linux); the first run follows the system language.
- **Contextual help (`?`):** every option, section, the connection status, the hotkeys switch, the language selector and the DLC option have a `?` button. Hover shows a tooltip; click expands the explanation inline.
- **Bilingual descriptions** for the 16 cheats in the domain catalog (`Cheat::description`).
- **Turn all off** button with a live counter of active cheats.
- **Apply** button with *Applied* confirmation for one-shot actions (Infinite Money), instead of a switch that never stayed on.
- Banner explaining what to do while the game is not detected.
- **Windows executable metadata:** application icon (`assets/icon.ico`), version information (product name, version 1.0.0, description, copyright) and an application manifest (`asInvoker`, Windows 10/11, per-monitor DPI) embedded with `winresource`. The window also uses `assets/icon.png`.
- Documentation in English (`README.md`, `CHANGELOG.md`) and Spanish (`README.es.md`, `CHANGELOG.es.md`).

### Changed
- Application renamed to **DOA6 Trainer +17** (window title and header); the executable is still `doa6-trainer.exe`.
- **Interface redesign** following the UI/UX Pro Max design system for gaming utilities: dark palette with purple primary color, cards, animated ON/OFF switches with text labels, fixed-width hotkey chips, status pill with drawn indicator (no emoji icons), visible keyboard focus, minimum text size of 12 px and contrast of at least 4.5:1.
- Design tokens and global style moved to `src/ui/theme.rs`; reusable widgets in `src/ui/widgets.rs`.
- Cheat options are disabled until the game is connected, with a tooltip explaining why.
- `steam_emu.ini` status is read once per second instead of on every frame, and toggle errors are now shown to the user.
- Default window size increased to 600×820.
- CI now builds on `windows-latest` with the MSVC toolchain (`x86_64-pc-windows-msvc`) and a static C runtime, instead of cross-compiling with MinGW. This reduces heuristic antivirus false positives.
- Release builds no longer open a console window next to the trainer.
- Crate version set to `1.0.0` to match the released version.

### Fixed
- The README hotkey table now matches the hotkeys actually assigned in the catalog.

## [1.0.0] - 2026-10-13

### Added
- **Multi-version support:**
  - Full support for `DOA6.exe` (v1.0 to v1.20).
  - Full support for `DOA6LR.exe` (Dead or Alive 6 Last Round, 2024 build).
  - Automatic detection of the running process (`DOA6.exe` or `DOA6LR.exe`).
- **16 runtime cheats:**
  - `Infinite Health (P1)`: code cave hook with P1/P2 pointer check.
  - `Max Break Gauge`: code cave that saturates the Break Gauge at 1000.0.
  - `Empty Opponent's Break Gauge`: code cave flag that drains the opponent's gauge.
  - `One Hit K.O.`: hook in the damage subtraction routine that zeroes the opponent's health.
  - `Freeze Timer`: trampoline that cancels the round time delta.
  - `Max Score`: double write at the score accumulation points, fixing the score at 99,999,999.
  - `Always Clear Challenges`: conditional jump trampoline that marks missions as completed.
  - `Unlock All Costumes`: byte patch in the catalog forcing availability.
  - `Unlock All Hairstyles`: direct patch in the hairstyle catalog routine.
  - `Unlock All Glasses`: direct patch in the accessory selector.
  - `Unlock All Music`: forced unlock of tracks in BGM selection.
  - `Unlock All Encyclopedia`: direct patch in the encyclopedia entry table.
  - `Unlock All Trivia`: direct patch in the trivia quizzes.
  - `Infinite Money`: multi-level static pointer resolution (`base + 0x08158EC8 -> +0x8CB8`) setting money to 99,999,999.
  - `Super Speed`: clock speed modifier (v1.0-v1.20).
  - `Slow Motion`: clock slowdown factor (v1.0-v1.20).
- **Persistent content unlock:**
  - `steam_emu.ini` editor (`DLCUnlockall=1/0`) to permanently unlock every costume, character and DLC pack without modifying the executable.
- **Clean Architecture in Rust:**
  - **Domain layer (`domain`):** pure business rules, AOB signatures with wildcards and RIP-relative captures, x86_64 encoder (`relative_jump`, `absolute_jump`).
  - **Application layer (`application`):** `Trainer`, `Patcher`, `SteamEmu`, `Controller` and the ports (`GameMemory`, `ProcessFinder`, `HotkeyListener`).
  - **Infrastructure layer (`infrastructure`):** Win32 implementation (`OpenProcess`, `VirtualAllocEx`, `ReadProcessMemory`, `WriteProcessMemory`, `RegisterHotKey`).
  - **User interface layer (`ui`):** modern GUI built with `egui` and the native OpenGL renderer.
- **Validation and test tools:**
  - 30 unit tests that run without OS dependencies (`cargo test --lib`).
  - Smoke test environment with a Wine emulator (`fake_doa6.c` and `tools/wine-smoke/run.sh`).
  - Live verification suite against the real game (`examples/verify_live.rs`).
  - Byte-level virtual memory validation tool (`examples/validate_modifications.rs`) checking the game's 19 memory locations.
