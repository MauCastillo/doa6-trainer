# DOA6 Trainer +17

**English** · [Español](README.es.md)

**DOA6 Trainer +17** is a modular, high-performance trainer for **Dead or Alive 6** (16 cheats + persistent DLC unlock), written in **Rust** following **Clean Architecture** principles, with a modern bilingual (English / Spanish) graphical interface built with **egui**.

It runs natively on **Windows 10/11 x64** and on **Linux through Wine, Proton and Lutris** (Steam Deck included).

---

## Download

Get the prebuilt `doa6-trainer.exe` from GitHub:

1. **GitHub Releases (recommended):** open **[Releases](https://github.com/MauCastillo/doa6-trainer/releases)** and download `doa6-trainer.exe` from the latest stable version.
2. **GitHub Actions artifacts:** every commit to `master` is built automatically. Open the **Actions** tab, pick the latest `Build & Release Artifacts` run and download **`doa6-trainer-windows-x64.zip`**.

---

## The Interface

| Element | What it does |
| :--- | :--- |
| **Language selector** (top right) | Switches the whole interface between **English** and **Español**. The choice is saved in `%APPDATA%\doa6-trainer\settings.ini` (Windows / Wine) or `~/.config/doa6-trainer/settings.ini` (Linux). On first run the system language is used. |
| **`?` buttons** | Every option, section and indicator has a `?` button. Hover it for a quick tooltip, or click it to expand the explanation inline (click again to collapse). |
| **Connection status** | Amber *Waiting for the game…* until `DOA6.exe` / `DOA6LR.exe` is found, then green *Connected · PID*. Options stay disabled until the game is connected. |
| **Global hotkeys switch** | Turns numpad shortcuts on or off. Also toggled anywhere with **`Ctrl + Shift + Home`**. |
| **Turn all off** | Disables every active cheat at once and restores the game's original code. Shows how many cheats are active. |
| **Option switches** | Each cheat has an animated ON/OFF switch with a text label, so the state never depends on color alone. |
| **Apply button** | One-shot actions (Infinite Money) use an *Apply* button and briefly show *Applied* on success. |
| **Error banner** | Errors appear in the selected language and can be dismissed. |

The visual design (dark palette, purple primary color, contrast of at least 4.5:1, visible keyboard focus, no emoji icons) follows the recommendations of the [UI/UX Pro Max](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) design system for gaming utilities. The colors live as tokens in `src/ui/theme.rs`.

---

## Supported Game Versions

The trainer finds the game in memory with adaptive AOB (*Array of Bytes*) signatures and RIP-relative pointers:

| Executable | Game version | Support | Notes |
| :--- | :--- | :---: | :--- |
| **`DOA6.exe`** | v1.0.0 to v1.20 (Steam / original release) | **16/16 cheats** | Includes *Super Speed* and *Slow Motion*. |
| **`DOA6LR.exe`** | Dead or Alive 6 Last Round (2024+ builds) | **14 runtime cheats + Money + DLC** | Game speed is not available because the compiler changed the frame loop. |

### Options by Version

| Option | Hotkey | `DOA6.exe` (v1.0–v1.20) | `DOA6LR.exe` (Last Round) | Technique |
| :--- | :---: | :---: | :---: | :--- |
| **Infinite Health (P1)** | `Num 1` | Yes | Yes | Code cave + flag |
| **Max Break Gauge** | `Num 2` | Yes | Yes | Code cave + flag |
| **Unlock All Costumes** | `Num 3` | Yes | Yes | Code patch |
| **Always Clear Challenges** | `Num 4` | Yes | Yes | Code patch |
| **Freeze Timer** | `Num 5` | Yes | Yes | Code patch |
| **Max Score** | `Num 6` | Yes | Yes | Code patch |
| **Super Speed** | `Num 7` | Yes | No | Code cave + flag (excludes Slow Motion) |
| **Slow Motion** | `Num 8` | Yes | No | Code cave + flag (excludes Super Speed) |
| **Empty Opponent's Break Gauge** | `Num 9` | Yes | Yes | Code cave + flag |
| **One Hit K.O.** | `Num 0` | Yes | Yes | Code cave + flag |
| **Infinite Money (99,999,999)** | `Ctrl + Num 1` | Yes | Yes | Multi-level static pointer (one-shot) |
| **Unlock All Hairstyles** | `Ctrl + Num 2` | Yes | Yes | Code patch |
| **Unlock All Glasses** | `Ctrl + Num 3` | Yes | Yes | Code patch |
| **Unlock All Music** | `Ctrl + Num 4` | Yes | Yes | Code patch |
| **Unlock All Encyclopedia** | `Ctrl + Num 5` | Yes | Yes | Code patch |
| **Unlock All Trivia** | `Ctrl + Num 6` | Yes | Yes | Code patch |
| **Unlock All DLC** | GUI switch | Yes | Yes | Persistent in `steam_emu.ini` |

> [!TIP]
> Press **`Ctrl + Shift + Home`** to enable or disable every hotkey at once, so you don't trigger cheats by accident while typing or playing.

> [!NOTE]
> Purchases made in the wardrobe after applying **Infinite Money** are saved permanently in `SYSTEMDATA`. The other unlocks only last while the cheat is on.

---

## Running the Trainer

### Windows 11 / Windows 10

1. **Start the game** (`DOA6.exe` or `DOA6LR.exe`) from Steam or your usual launcher.
2. **Run `doa6-trainer.exe`**, preferably with right-click → **Run as administrator** so it gets `PROCESS_ALL_ACCESS`.
3. The status indicator turns green (**Connected · PID …**) once the game is detected.
4. Turn options on with their switches or with the numpad hotkeys.
5. **When you close the trainer**, every hook, jump and modified byte in game memory is restored to its original value.

### Linux (Lutris / Wine / Proton)

On Linux the game runs inside an isolated Wine prefix. The trainer must run inside the **same prefix (`WINEPREFIX`)** as the game to access its memory.

#### Option A: run it manually from a terminal (recommended)

1. Start the game from Lutris:
   ```bash
   lutris -d lutris:rungame/dead-or-alive-the-last-round
   ```

2. Launch the trainer with the same Wine prefix and Proton runner (adjust the paths to your setup):
   ```bash
   WINEPREFIX="$HOME/Games/dead-or-alive-the-last-round" \
   DISPLAY=:0 \
   "$HOME/.local/share/Steam/compatibilitytools.d/GE-Proton11-7-x86_64/files/bin/wine" \
   "$HOME/Games/dead-or-alive-the-last-round/drive_c/Program Files/DEAD OR ALIVE 6 Last Round/doa6-trainer.exe"
   ```

#### Option B: launch it from Lutris

1. Open **Lutris**, right-click the game and choose **Configure**.
2. Go to the **System options** tab.
3. Under **Pre-launch script** (or as a concurrent session), add the call to `doa6-trainer.exe`.

---

## Building for Windows

### Native build on Windows

Install [Rust and Cargo](https://rustup.rs) and run in PowerShell or CMD:
```powershell
cargo build --release
```
The executable is created at:
```
target\release\doa6-trainer.exe
```

### Cross-compiling from Linux to Windows x64

1. Install the `mingw-w64` cross-compiler:
   - **Arch Linux / Manjaro:**
     ```bash
     sudo pacman -S mingw-w64-gcc
     ```
   - **Ubuntu / Debian:**
     ```bash
     sudo apt install gcc-mingw-w64-x86-64
     ```
   - **Fedora:**
     ```bash
     sudo dnf install mingw64-gcc
     ```

2. Add the Windows x64 target to Rust:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

3. Build in release mode:
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```
The optimized Windows binary is created at:
```
target/x86_64-pc-windows-gnu/release/doa6-trainer.exe
```

### Previewing the interface on Linux

The interface can be run natively on Linux to check the design (it uses stub adapters, so it never connects to the game):
```bash
cargo run
```

---

## Tests

### 1. Domain and logic unit tests
Tests the AOB signature engine, the x86_64 jump encoder, the cheat catalog, the translations and the orchestrator against a simulated in-memory process (no Windows or Wine needed):
```bash
cargo test --lib
```

### 2. Wine integration smoke test
Runs a fake C executable (`fake_doa6.c`) under Wine with the game's exact instructions and checks the full cycle of injecting, enabling and reverting hooks:
```bash
tools/wine-smoke/run.sh
```

### 3. Live check against the real game
Connects to the running game (`DOA6.exe` or `DOA6LR.exe`), toggles every option, checks the results and restores all hooks:
```bash
cargo run --example verify_live
```

### 4. Byte-level validation in virtual memory
Inspects each of the game's 19 memory addresses before, during and after enabling each cheat, checking that opcodes and values match byte for byte:
```bash
cargo run --example validate_modifications
```

---

## Code Architecture

The project follows **Hexagonal / Clean Architecture**, keeping business logic strictly separate from operating-system details and the graphical interface:

```
src/
├── domain/            # Pure domain logic (no OS dependencies)
│   ├── pattern.rs     # AOB scanner with wildcards, captures and templates
│   ├── x86.rs         # x86_64 jump encoding (rel32, absolute)
│   ├── cheat.rs       # Entities: Cheat, Effect, Patch, Hook, GameCatalog
│   └── doa6.rs        # The 16 cheats, their bilingual names/descriptions and AOB signatures
├── application/       # Use cases and orchestration
│   ├── ports.rs       # Ports/interfaces: GameMemory, ProcessFinder, HotkeyListener
│   ├── patcher.rs     # AOB signatures, direct patches and code caves
│   ├── trainer.rs     # Cheat lifecycle, atomic install and rollback
│   ├── steam_emu.rs   # Persistent ini editing (DLC)
│   └── controller.rs  # Background control loop and global hotkeys
├── infrastructure/    # Infrastructure adapters
│   ├── windows.rs     # Win32 implementation (OpenProcess, VirtualAllocEx, ...)
│   └── stub.rs        # Simulated adapters for non-Windows builds and tests
├── ui/                # Presentation layer (egui + glow OpenGL renderer)
│   ├── app.rs         # Main window, layout and event binding
│   ├── i18n.rs        # Languages, UI strings, sections and help texts
│   ├── settings.rs    # Saved user preferences (language)
│   ├── theme.rs       # Design tokens: colors, radii, text sizes
│   └── widgets.rs     # Switch, "?" help button, hotkey chip, status pill
├── lib.rs             # Public module exports
└── main.rs            # Application entry point and eframe startup
```
