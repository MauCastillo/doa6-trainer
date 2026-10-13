# Dead or Alive 6 Trainer (Rust)

Trainer modular y de alto rendimiento para **Dead or Alive 6** escrito en **Rust**, diseñado bajo principios de **Arquitectura Limpia (Clean Architecture)** con interfaz gráfica moderna construida en **egui**.

El trainer está optimizado tanto para **Windows 10/11 x64** nativo como para entornos **Linux bajo Wine, Proton y Lutris** (incluyendo Steam Deck).

---

## Descargar el Ejecutable (Artefactos en GitHub)

Puedes obtener el binario precompilado `doa6-trainer.exe` directamente desde el repositorio de GitHub:

1. **Releases de GitHub (Recomendado):**
   - Dirígete a la sección de **[Releases](https://github.com/MauCastillo/doa6-trainer/releases)** en GitHub.
   - Descarga el archivo adjunto `doa6-trainer.exe` de la última versión estable (`v1.0.0`).
2. **GitHub Actions Artifacts:**
   - En la pestaña **Actions** de GitHub, cada commit a `master` compila automáticamente el ejecutable.
   - Haz clic en la última ejecución de `Build & Release Artifacts` y descarga el artefacto **`doa6-trainer-windows-x64.zip`**.

---

## Versiones Compatibles del Juego

El trainer detecta automáticamente el ejecutable en memoria mediante firmas de bytes AOB (*Array of Bytes*) adaptativas y punteros relativos a RIP:

| Ejecutable | Versión del Juego | Compatibilidad | Notas |
| :--- | :--- | :---: | :--- |
| **`DOA6.exe`** | v1.0.0 a v1.20 (Steam / Lanzamiento original) | **100% (16/16 cheats)** | Incluye *Super Speed* y *Slow Motion*. |
| **`DOA6LR.exe`** | Dead or Alive 6 Last Round (Builds 2024+) | **100% (14 runtime + Dinero + DLCs)** | Todos los trucos de salud, combate, desbloqueos, misiones, dinero infinito y DLC permanente. *(Velocidad de reloj omitida por cambios del compilador en el loop de frames)*. |

### Matriz de Opciones por Versión

| Opción / Truco | Atajo de Teclado | `DOA6.exe` (v1.0–v1.20) | `DOA6LR.exe` (Last Round) | Tipo de Inyección |
| :--- | :---: | :---: | :---: | :--- |
| **Salud Infinita (P1)** | `Num 1` | Sí | Sí | Code Cave + Flag P1/P2 |
| **Break Gauge Máximo** | `Num 2` | Sí | Sí | Code Cave + Flag |
| **Break Blow Fácil** | `Num 3` | Sí | Sí | Code Cave + Flag |
| **One Hit KO** | `Num 4` | Sí | Sí | Code Cave + Flag |
| **Congelar Tiempo** | `Num 5` | Sí | Sí | Code Cave Trampoline |
| **Puntuación Máxima** | `Num 6` | Sí | Sí | Code Cave Trampoline |
| **Completar Desafíos al Instante** | `Num 7` | Sí | Sí | Code Cave Trampoline |
| **Desbloquear Todos los Trajes** | `Num 8` | Sí | Sí | Parche Directo (`mov al, 1`) |
| **Desbloquear Todos los Peinados**| `Num 9` | Sí | Sí | Parche Directo (`mov al, 1`) |
| **Desbloquear Todas las Gafas** | `Num 0` | Sí | Sí | Parche Directo (`mov al, 1`) |
| **Desbloquear Toda la Música** | `Ctrl + Num 1` | Sí | Sí | Parche Directo (`mov al, 1`) |
| **Desbloquear Enciclopedia** | `Ctrl + Num 2` | Sí | Sí | Parche Directo (`mov al, 1`) |
| **Desbloquear Preguntas Trivia** | `Ctrl + Num 3` | Sí | Sí | Parche Directo (`mov al, 1`) |
| **Dinero Infinito (99,999,999)** | `Ctrl + Num 4` | Sí | Sí | Puntero Estático Multinivel |
| **Super Velocidad** | `Ctrl + Num 5` | Sí | No *(v1.0-v1.20)* | Reloj de frames (AOB clásico) |
| **Cámara Lenta** | `Ctrl + Num 6` | Sí | No *(v1.0-v1.20)* | Reloj de frames (AOB clásico) |
| **Desbloquear Todos los DLCs** | Botón GUI | Sí | Sí | Persistente en `steam_emu.ini` |

> [!TIP]
> Puedes habilitar o deshabilitar todos los atajos de teclado simultáneamente pulsando **`Ctrl + Shift + Inicio`** (para evitar activaciones accidentales mientras escribes o juegas).

---

## Cómo Ejecutar el Proyecto

### 1. En Windows 11 / Windows 10

1. **Inicia el juego** (`DOA6.exe` o `DOA6LR.exe`) a través de Steam o tu lanzador habitual.
2. **Ejecuta `doa6-trainer.exe`** (preferiblemente haciendo clic derecho y seleccionando **"Ejecutar como Administrador"** para otorgar privilegios `PROCESS_ALL_ACCESS`).
3. El trainer detectará automáticamente el proceso del juego y la etiqueta cambiará a color verde: **`Juego conectado (DOA6.exe)`** o **`Juego conectado (DOA6LR.exe)`**.
4. Activa las opciones que desees utilizando los botones interactivos o mediante los atajos del teclado numérico (`Numpad`).
5. **Al cerrar el trainer**, todos los ganchos (*hooks*), saltos y bytes modificados en la memoria del juego se restauran inmediatamente a sus valores originales de fábrica.

---

### 2. En Linux (Lutris / Wine / Proton)

En Linux, los juegos se ejecutan dentro de prefijos de Wine aislados. Para que el trainer pueda interactuar con la memoria de `DOA6LR.exe`, debe ejecutarse dentro del **mismo prefijo (`WINEPREFIX`)** que el juego.

#### Opción A: Ejecución manual desde Terminal (Recomendado)

1. Inicia el juego desde Lutris:
   ```bash
   lutris -d lutris:rungame/dead-or-alive-the-last-round
   ```

2. Abre una terminal y lanza el trainer usando el mismo prefijo de Wine y runner de Proton:
   ```bash
   WINEPREFIX="/home/wau/Games/dead-or-alive-the-last-round" \
   DISPLAY=:0 \
   /home/wau/.local/share/Steam/compatibilitytools.d/GE-Proton11-7-x86_64/files/bin/wine \
   "/home/wau/Games/dead-or-alive-the-last-round/drive_c/Program Files/DEAD OR ALIVE 6 Last Round/doa6-trainer.exe"
   ```

#### Opción B: Integrado automáticamente en Lutris

1. Abre **Lutris**, haz clic derecho sobre el juego y selecciona **Configurar**.
2. Ve a la pestaña **Opciones del sistema**.
3. En la sección **Script previo al lanzamiento** (o agregándolo a una sesión concurrente), configura la llamada a `doa6-trainer.exe`.

---

## Cómo Compilar para Windows

### 1. Compilación Nativa en Windows
Instala [Rust y Cargo](https://rustup.rs) y ejecuta en PowerShell o CMD:
```powershell
cargo build --release
```
El ejecutable resultante estará en:
```
target\release\doa6-trainer.exe
```

### 2. Compilación Cruzada desde Linux hacia Windows x64
Si desarrollas en Linux (como Arch Linux, Ubuntu, Fedora, etc.):

1. Instala el paquete de compilación cruzada `mingw-w64`:
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

2. Agrega el target de Windows x64 a Rust:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

3. Compila el proyecto en modo release:
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```
El binario optimizado para Windows se encontrará en:
```
target/x86_64-pc-windows-gnu/release/doa6-trainer.exe
```

---

## Ejecución de Pruebas (Tests)

El proyecto incluye múltiples suites de pruebas automatizadas:

### 1. Pruebas Unitarias de Lógica y Dominio
Prueba el motor de firmas AOB, codificador de saltos x86_64, catalogo de trucos y orquestador contra un proceso simulado en memoria (no requiere Windows ni Wine):
```bash
cargo test --lib
```

### 2. Smoke Test de Integración en Wine
Ejecuta un ejecutable falso en C (`fake_doa6.c`) dentro de Wine con las instrucciones exactas del juego y verifica el ciclo completo de inyección, activación y reversión de hooks:
```bash
tools/wine-smoke/run.sh
```

### 3. Verificación en Vivo contra el Juego Real
Se conecta al proceso real en ejecución (`DOA6.exe` o `DOA6LR.exe`), conmuta cada opción, valida respuestas y restaura todos los ganchos:
```bash
cargo run --example verify_live
```

### 4. Validación Profunda a Nivel de Bytes en Memoria Virtual
Inspecciona directamente cada una de las 19 direcciones de memoria virtual del juego antes, durante y después de activar cada truco, comprobando que los opcodes y valores coinciden byte a byte con lo esperado:
```bash
cargo run --example validate_modifications
```

---

## Arquitectura del Código

El proyecto está estructurado siguiendo **Arquitectura Hexagonal / Clean Architecture**, separando estrictamente la lógica de negocio de los detalles de bajo nivel del sistema operativo y de la interfaz gráfica:

```
src/
├── domain/            # Lógica pura del dominio (sin dependencias del SO)
│   ├── pattern.rs     # Escáner AOB con wildcards, capturas y plantillas
│   ├── x86.rs         # Ensamblado y codificación de saltos x86_64 (rel32, absoluto)
│   ├── cheat.rs       # Entidades: Cheat, Effect, Patch, Hook, GameCatalog
│   └── doa6.rs        # Catálogo formal de los 16 trucos y sus firmas AOB
├── application/       # Casos de uso y orquestación
│   ├── ports.rs       # Puertos/Interfaces: GameMemory, ProcessFinder, HotkeyListener
│   ├── patcher.rs     # Gestión de firmas AOB, parches directos y cuevas de código
│   ├── trainer.rs     # Ciclo de vida de trucos, instalación atómica y reversión
│   ├── steam_emu.rs   # Manipulación persistente de ini (DLCs)
│   └── controller.rs  # Loop de control en segundo plano y atajos globales
├── infrastructure/    # Adaptadores de infraestructura
│   ├── windows.rs     # Implementación Win32 (OpenProcess, VirtualAllocEx, etc.)
│   └── stub.rs        # Adaptadores simulados para entornos no-Windows / tests
├── ui/                # Capa de presentación (egui + renderizador glow OpenGL)
│   ├── app.rs         # Ventana principal, layout visual y binding de eventos
│   └── theme.rs       # Estilos, colores y fuentes
├── lib.rs             # Exportación pública modular
└── main.rs            # Entrada de la aplicación y arranque de eframe
```
