# Changelog

Todas las novedades y cambios notables de este proyecto se documentan en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y se adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [1.0.0] - 2026-10-13

### Añadido
- **Soporte Multi-Versión:**
  - Compatibilidad total con `DOA6.exe` (v1.0 a v1.20).
  - Compatibilidad completa con `DOA6LR.exe` (Dead or Alive 6 Last Round, build 2024).
  - Escaneo automático del proceso en ejecución (`DOA6.exe` o `DOA6LR.exe`).
- **16 Trucos en Tiempo de Ejecución (Runtime):**
  - `Salud Infinita (P1)`: Hook en cueva de código con verificación de puntero P1/P2.
  - `Break Gauge Máximo`: Cueva de código que satura la barra de ruptura a 1000.0.
  - `Break Blow Fácil`: Flag en cueva de código que permite Break Blows sin coste.
  - `One Hit KO`: Hook en la rutina de resta de daño que anula la vida del oponente.
  - `Congelar Tiempo`: Trampolín que anula el incremento del delta de tiempo del round.
  - `Puntuación Máxima`: Sobrescritura en doble punto de suma fijando el score en 99,999,999.
  - `Completar Desafíos al Instante`: Trampolín de salto condicional que marca misiones como completadas.
  - `Desbloquear Trajes`: Parche de byte en catálogo forzando disponibilidad.
  - `Desbloquear Peinados`: Parche directo en la rutina de catálogo de peinados.
  - `Desbloquear Gafas`: Parche directo en el selector de accesorios.
  - `Desbloquear Música`: Desbloqueo forzado de pistas en la selección de BGM.
  - `Desbloquear Enciclopedia`: Parche directo en la tabla de entradas de la enciclopedia.
  - `Desbloquear Trivia`: Parche directo en los cuestionarios de trivia.
  - `Dinero Infinito`: Resolución de puntero estático multinivel (`base + 0x08158EC8 -> +0x8CB8`) fijando monedas en 99,999,999.
  - `Super Velocidad`: Modificador de velocidad de reloj (v1.0-v1.20).
  - `Cámara Lenta`: Factor de ralentización del reloj (v1.0-v1.20).
- **Desbloqueo Persistente de Contenido:**
  - Modificador de `steam_emu.ini` (`DLCUnlockall=1/0`) para desbloquear permanentemente todos los trajes, personajes y paquetes DLC sin modificar el ejecutable.
- **Arquitectura Limpia (Clean Architecture en Rust):**
  - Capa de **Dominio (`domain`)**: Reglas de negocio puras, definición de firmas AOB con wildcards y capturas RIP-relative, codificador x86_64 (`relative_jump`, `absolute_jump`).
  - Capa de **Aplicación (`application`)**: `Trainer`, `Patcher`, `SteamEmu`, `Controller`, interfaces de puertos (`GameMemory`, `ProcessFinder`, `HotkeyListener`).
  - Capa de **Infraestructura (`infrastructure`)**: Implementación Win32 con `windows-sys` (`OpenProcess`, `VirtualAllocEx`, `ReadProcessMemory`, `WriteProcessMemory`, `RegisterHotKey`).
  - Capa de **Interfaz de Usuario (`ui`)**: GUI moderna construida con `egui` y renderizador nativo OpenGL.
- **Herramientas de Validación y Pruebas:**
  - 30 tests unitarios que se ejecutan sin dependencias de sistema operativo (`cargo test --lib`).
  - Entorno smoke test con emulador en Wine (`fake_doa6.c` y `tools/wine-smoke/run.sh`).
  - Suite de verificación en vivo contra el juego real (`examples/verify_live.rs`).
  - Herramienta de validación a nivel de byte en memoria virtual (`examples/validate_modifications.rs`) comprobando los 19 puntos de memoria del juego.
