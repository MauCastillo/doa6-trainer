# Changelog

[English](CHANGELOG.md) · **Español**

Todas las novedades y cambios notables de este proyecto se documentan en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto se adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [Sin publicar]

### Añadido
- **Selector de idioma (English / Español):** toda la interfaz, las ayudas y los mensajes de error están disponibles en ambos idiomas. La elección se guarda en `settings.ini` dentro de `%APPDATA%\doa6-trainer` (Windows / Wine) o `~/.config/doa6-trainer` (Linux); el primer arranque usa el idioma del sistema.
- **Ayuda contextual (`?`):** cada opción, sección, el estado de conexión, el interruptor de atajos, el selector de idioma y la opción de DLC tienen un botón `?`. Al pasar el ratón muestra una ayuda rápida; al hacer clic despliega la explicación debajo.
- **Descripciones bilingües** de los 16 trucos en el catálogo de dominio (`Cheat::description`).
- Botón **Desactivar todo** con contador en vivo de trucos activos.
- Botón **Aplicar** con confirmación *Aplicado* para acciones de un solo uso (Dinero infinito), en lugar de un interruptor que nunca quedaba activo.
- Banner que explica qué hacer mientras no se detecta el juego.
- **Metadatos del ejecutable de Windows:** icono de la aplicación (`assets/icon.ico`), información de versión (nombre del producto, versión 1.0.0, descripción, copyright) y manifiesto de aplicación (`asInvoker`, Windows 10/11, DPI por monitor) incrustados con `winresource`. La ventana también usa `assets/icon.png`.
- Documentación en inglés (`README.md`, `CHANGELOG.md`) y español (`README.es.md`, `CHANGELOG.es.md`).

### Cambiado
- La aplicación pasa a llamarse **DOA6 Trainer +17** (título de la ventana y encabezado); el ejecutable sigue siendo `doa6-trainer.exe`.
- **Rediseño de la interfaz** según el sistema de diseño UI/UX Pro Max para utilidades de videojuegos: paleta oscura con color primario morado, tarjetas, interruptores SÍ/NO animados con etiqueta de texto, chips de atajo de ancho fijo, indicador de estado dibujado (sin emojis como iconos), foco de teclado visible, texto mínimo de 12 px y contraste mínimo de 4.5:1.
- Tokens de diseño y estilo global en `src/ui/theme.rs`; widgets reutilizables en `src/ui/widgets.rs`.
- Las opciones de trucos quedan deshabilitadas hasta que el juego esté conectado, con un tooltip que explica el motivo.
- El estado de `steam_emu.ini` se lee una vez por segundo en lugar de en cada frame, y los errores al alternarlo ahora se muestran al usuario.
- Tamaño de ventana por defecto aumentado a 600×820.
- El CI ahora compila en `windows-latest` con la toolchain MSVC (`x86_64-pc-windows-msvc`) y runtime de C estático, en lugar de compilación cruzada con MinGW. Esto reduce los falsos positivos heurísticos de los antivirus.
- Las builds de release ya no abren una ventana de consola junto al trainer.
- Versión del crate fijada en `1.0.0` para coincidir con la versión publicada.

### Corregido
- La tabla de atajos del README ahora coincide con los atajos realmente asignados en el catálogo.

## [1.0.0] - 2026-10-13

### Añadido
- **Soporte multiversión:**
  - Compatibilidad total con `DOA6.exe` (v1.0 a v1.20).
  - Compatibilidad total con `DOA6LR.exe` (Dead or Alive 6 Last Round, build 2024).
  - Detección automática del proceso en ejecución (`DOA6.exe` o `DOA6LR.exe`).
- **16 trucos en tiempo de ejecución:**
  - `Vida infinita (J1)`: hook en code cave con verificación de puntero J1/J2.
  - `Barra de Break al máximo`: code cave que satura la barra de Break a 1000.0.
  - `Vaciar la barra de Break del rival`: flag en code cave que vacía la barra del oponente.
  - `K.O. de un golpe`: hook en la rutina de resta de daño que anula la vida del oponente.
  - `Congelar el tiempo`: trampolín que anula el incremento del delta de tiempo del round.
  - `Puntuación máxima`: doble escritura en los puntos de suma, fijando la puntuación en 99.999.999.
  - `Completar siempre los desafíos`: trampolín de salto condicional que marca las misiones como completadas.
  - `Desbloquear todos los trajes`: parche de byte en el catálogo que fuerza la disponibilidad.
  - `Desbloquear todos los peinados`: parche directo en la rutina del catálogo de peinados.
  - `Desbloquear todas las gafas`: parche directo en el selector de accesorios.
  - `Desbloquear toda la música`: desbloqueo forzado de pistas en la selección de BGM.
  - `Desbloquear toda la enciclopedia`: parche directo en la tabla de entradas de la enciclopedia.
  - `Desbloquear todas las curiosidades`: parche directo en los cuestionarios de trivia.
  - `Dinero infinito`: resolución de puntero estático multinivel (`base + 0x08158EC8 -> +0x8CB8`) que fija el dinero en 99.999.999.
  - `Supervelocidad`: modificador de la velocidad del reloj (v1.0-v1.20).
  - `Cámara lenta`: factor de ralentización del reloj (v1.0-v1.20).
- **Desbloqueo persistente de contenido:**
  - Editor de `steam_emu.ini` (`DLCUnlockall=1/0`) para desbloquear permanentemente todos los trajes, personajes y paquetes DLC sin modificar el ejecutable.
- **Arquitectura limpia en Rust:**
  - **Capa de dominio (`domain`):** reglas de negocio puras, firmas AOB con comodines y capturas relativas a RIP, codificador x86_64 (`relative_jump`, `absolute_jump`).
  - **Capa de aplicación (`application`):** `Trainer`, `Patcher`, `SteamEmu`, `Controller` y los puertos (`GameMemory`, `ProcessFinder`, `HotkeyListener`).
  - **Capa de infraestructura (`infrastructure`):** implementación Win32 (`OpenProcess`, `VirtualAllocEx`, `ReadProcessMemory`, `WriteProcessMemory`, `RegisterHotKey`).
  - **Capa de interfaz de usuario (`ui`):** GUI moderna construida con `egui` y el renderizador nativo OpenGL.
- **Herramientas de validación y pruebas:**
  - 30 tests unitarios que se ejecutan sin dependencias del sistema operativo (`cargo test --lib`).
  - Entorno de smoke test con emulador en Wine (`fake_doa6.c` y `tools/wine-smoke/run.sh`).
  - Suite de verificación en vivo contra el juego real (`examples/verify_live.rs`).
  - Herramienta de validación a nivel de byte en memoria virtual (`examples/validate_modifications.rs`) que comprueba las 19 posiciones de memoria del juego.
