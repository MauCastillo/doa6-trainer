# DOA6 Trainer +17

[English](README.md) · **Español**

**DOA6 Trainer +17** es un trainer modular y de alto rendimiento para **Dead or Alive 6** (16 trucos + desbloqueo persistente de DLC), escrito en **Rust** siguiendo los principios de **Arquitectura Limpia (Clean Architecture)**, con una interfaz gráfica moderna y bilingüe (inglés / español) construida con **egui**.

Funciona de forma nativa en **Windows 10/11 x64** y en **Linux mediante Wine, Proton y Lutris** (incluida la Steam Deck).

---

## Descarga

Obtén el binario precompilado `doa6-trainer.exe` desde GitHub:

1. **Releases de GitHub (recomendado):** abre **[Releases](https://github.com/MauCastillo/doa6-trainer/releases)** y descarga `doa6-trainer.exe` de la última versión estable.
2. **Artefactos de GitHub Actions:** cada commit a `master` se compila automáticamente. Abre la pestaña **Actions**, elige la última ejecución de `Build & Release Artifacts` y descarga **`doa6-trainer-windows-x64.zip`**.

> [!NOTE]
> **Avisos del antivirus:** un trainer lee y escribe la memoria de otro proceso (`OpenProcess`, `VirtualAllocEx`, `WriteProcessMemory`), las mismas técnicas que usa algún malware, por lo que unos pocos motores basados en reputación pueden marcar las builds nuevas (por ejemplo, McAfee `Ti!<hash>` o Bkav `W32.Malware`). Todo el código fuente está en este repositorio y puedes compilarlo tú mismo.

---

## La Interfaz

| Elemento | Qué hace |
| :--- | :--- |
| **Selector de idioma** (arriba a la derecha) | Cambia toda la interfaz entre **English** y **Español**. La elección se guarda en `%APPDATA%\doa6-trainer\settings.ini` (Windows / Wine) o `~/.config/doa6-trainer/settings.ini` (Linux). En el primer arranque se usa el idioma del sistema. |
| **Botones `?`** | Cada opción, sección e indicador tiene un botón `?`. Pasa el ratón por encima para ver una ayuda rápida, o haz clic para desplegar la explicación debajo (otro clic la oculta). |
| **Estado de conexión** | *Esperando al juego…* en ámbar hasta encontrar `DOA6.exe` / `DOA6LR.exe`; después *Conectado · PID* en verde. Las opciones permanecen deshabilitadas hasta que el juego esté conectado. |
| **Interruptor de atajos globales** | Activa o pausa los atajos del teclado numérico. También se alterna desde cualquier lugar con **`Ctrl + Shift + Inicio`**. |
| **Desactivar todo** | Desactiva de una vez todos los trucos activos y restaura el código original del juego. Muestra cuántos trucos hay activos. |
| **Interruptores de opciones** | Cada truco tiene un interruptor animado SÍ/NO con etiqueta de texto, para que el estado nunca dependa solo del color. |
| **Botón Aplicar** | Las acciones de un solo uso (Dinero infinito) usan un botón *Aplicar* y muestran *Aplicado* brevemente al completarse. |
| **Banner de error** | Los errores se muestran en el idioma seleccionado y se pueden cerrar. |

El diseño visual (paleta oscura, color primario morado, contraste mínimo de 4.5:1, foco de teclado visible, sin emojis como iconos) sigue las recomendaciones del sistema de diseño [UI/UX Pro Max](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) para utilidades de videojuegos. Los colores están definidos como tokens en `src/ui/theme.rs`.

---

## Versiones Compatibles del Juego

El trainer localiza el juego en memoria mediante firmas AOB (*Array of Bytes*) adaptativas y punteros relativos a RIP:

| Ejecutable | Versión del juego | Compatibilidad | Notas |
| :--- | :--- | :---: | :--- |
| **`DOA6.exe`** | v1.0.0 a v1.20 (Steam / lanzamiento original) | **16/16 trucos** | Incluye *Supervelocidad* y *Cámara lenta*. |
| **`DOA6LR.exe`** | Dead or Alive 6 Last Round (builds 2024+) | **14 trucos en tiempo real + Dinero + DLC** | La velocidad del juego no está disponible porque el compilador cambió el bucle de frames. |

### Opciones por Versión

| Opción | Atajo | `DOA6.exe` (v1.0–v1.20) | `DOA6LR.exe` (Last Round) | Técnica |
| :--- | :---: | :---: | :---: | :--- |
| **Vida infinita (J1)** | `Num 1` | Sí | Sí | Code cave + flag |
| **Barra de Break al máximo** | `Num 2` | Sí | Sí | Code cave + flag |
| **Desbloquear todos los trajes** | `Num 3` | Sí | Sí | Parche de código |
| **Completar siempre los desafíos** | `Num 4` | Sí | Sí | Parche de código |
| **Congelar el tiempo** | `Num 5` | Sí | Sí | Parche de código |
| **Puntuación máxima** | `Num 6` | Sí | Sí | Parche de código |
| **Supervelocidad** | `Num 7` | Sí | No | Code cave + flag (excluye Cámara lenta) |
| **Cámara lenta** | `Num 8` | Sí | No | Code cave + flag (excluye Supervelocidad) |
| **Vaciar la barra de Break del rival** | `Num 9` | Sí | Sí | Code cave + flag |
| **K.O. de un golpe** | `Num 0` | Sí | Sí | Code cave + flag |
| **Dinero infinito (99.999.999)** | `Ctrl + Num 1` | Sí | Sí | Puntero estático multinivel (un solo uso) |
| **Desbloquear todos los peinados** | `Ctrl + Num 2` | Sí | Sí | Parche de código |
| **Desbloquear todas las gafas** | `Ctrl + Num 3` | Sí | Sí | Parche de código |
| **Desbloquear toda la música** | `Ctrl + Num 4` | Sí | Sí | Parche de código |
| **Desbloquear toda la enciclopedia** | `Ctrl + Num 5` | Sí | Sí | Parche de código |
| **Desbloquear todas las curiosidades** | `Ctrl + Num 6` | Sí | Sí | Parche de código |
| **Desbloquear todos los DLC** | Interruptor en la GUI | Sí | Sí | Persistente en `steam_emu.ini` |

> [!TIP]
> Pulsa **`Ctrl + Shift + Inicio`** para activar o desactivar todos los atajos a la vez y evitar activaciones accidentales mientras escribes o juegas.

> [!NOTE]
> Lo que compres en el armario tras aplicar **Dinero infinito** se guarda para siempre en `SYSTEMDATA`. Los demás desbloqueos solo duran mientras el truco está activo.

---

## Cómo Ejecutar el Trainer

### Windows 11 / Windows 10

1. **Inicia el juego** (`DOA6.exe` o `DOA6LR.exe`) desde Steam o tu lanzador habitual.
2. **Ejecuta `doa6-trainer.exe`**, preferiblemente con clic derecho → **Ejecutar como administrador** para obtener `PROCESS_ALL_ACCESS`.
3. El indicador de estado se pone en verde (**Conectado · PID …**) al detectar el juego.
4. Activa las opciones con sus interruptores o con los atajos del teclado numérico.
5. **Al cerrar el trainer**, todos los hooks, saltos y bytes modificados en la memoria del juego se restauran a sus valores originales.

### Linux (Lutris / Wine / Proton)

En Linux el juego se ejecuta dentro de un prefijo de Wine aislado. El trainer debe ejecutarse dentro del **mismo prefijo (`WINEPREFIX`)** que el juego para acceder a su memoria.

#### Opción A: ejecución manual desde la terminal (recomendado)

1. Inicia el juego desde Lutris:
   ```bash
   lutris -d lutris:rungame/dead-or-alive-the-last-round
   ```

2. Lanza el trainer con el mismo prefijo de Wine y runner de Proton (ajusta las rutas a tu instalación):
   ```bash
   WINEPREFIX="$HOME/Games/dead-or-alive-the-last-round" \
   DISPLAY=:0 \
   "$HOME/.local/share/Steam/compatibilitytools.d/GE-Proton11-7-x86_64/files/bin/wine" \
   "$HOME/Games/dead-or-alive-the-last-round/drive_c/Program Files/DEAD OR ALIVE 6 Last Round/doa6-trainer.exe"
   ```

#### Opción B: lanzarlo desde Lutris

1. Abre **Lutris**, haz clic derecho sobre el juego y elige **Configurar**.
2. Ve a la pestaña **Opciones del sistema**.
3. En **Script previo al lanzamiento** (o como sesión concurrente), añade la llamada a `doa6-trainer.exe`.

---

## Cómo Compilar para Windows

### Compilación nativa en Windows

Instala [Rust y Cargo](https://rustup.rs) y ejecuta en PowerShell o CMD:
```powershell
cargo build --release
```
El ejecutable se genera en:
```
target\release\doa6-trainer.exe
```

### Compilación cruzada desde Linux hacia Windows x64

1. Instala el compilador cruzado `mingw-w64`:
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

2. Añade el target de Windows x64 a Rust:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

3. Compila en modo release:
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```
El binario optimizado para Windows se genera en:
```
target/x86_64-pc-windows-gnu/release/doa6-trainer.exe
```

### Previsualizar la interfaz en Linux

La interfaz puede ejecutarse de forma nativa en Linux para revisar el diseño (usa adaptadores simulados, por lo que nunca se conecta al juego):
```bash
cargo run
```

---

## Pruebas

### 1. Pruebas unitarias de dominio y lógica
Prueba el motor de firmas AOB, el codificador de saltos x86_64, el catálogo de trucos, las traducciones y el orquestador contra un proceso simulado en memoria (no requiere Windows ni Wine):
```bash
cargo test --lib
```

### 2. Smoke test de integración en Wine
Ejecuta un ejecutable falso en C (`fake_doa6.c`) dentro de Wine con las instrucciones exactas del juego y verifica el ciclo completo de inyección, activación y reversión de hooks:
```bash
tools/wine-smoke/run.sh
```

### 3. Verificación en vivo contra el juego real
Se conecta al juego en ejecución (`DOA6.exe` o `DOA6LR.exe`), alterna cada opción, valida los resultados y restaura todos los hooks:
```bash
cargo run --example verify_live
```

### 4. Validación a nivel de bytes en memoria virtual
Inspecciona cada una de las 19 direcciones de memoria del juego antes, durante y después de activar cada truco, comprobando que opcodes y valores coinciden byte a byte:
```bash
cargo run --example validate_modifications
```

---

## Arquitectura del Código

El proyecto sigue la **Arquitectura Hexagonal / Clean Architecture**, separando estrictamente la lógica de negocio de los detalles del sistema operativo y de la interfaz gráfica:

```
src/
├── domain/            # Lógica pura del dominio (sin dependencias del SO)
│   ├── pattern.rs     # Escáner AOB con comodines, capturas y plantillas
│   ├── x86.rs         # Codificación de saltos x86_64 (rel32, absoluto)
│   ├── cheat.rs       # Entidades: Cheat, Effect, Patch, Hook, GameCatalog
│   └── doa6.rs        # Los 16 trucos, sus nombres/descripciones bilingües y firmas AOB
├── application/       # Casos de uso y orquestación
│   ├── ports.rs       # Puertos/interfaces: GameMemory, ProcessFinder, HotkeyListener
│   ├── patcher.rs     # Firmas AOB, parches directos y code caves
│   ├── trainer.rs     # Ciclo de vida de los trucos, instalación atómica y reversión
│   ├── steam_emu.rs   # Edición persistente del ini (DLC)
│   └── controller.rs  # Bucle de control en segundo plano y atajos globales
├── infrastructure/    # Adaptadores de infraestructura
│   ├── windows.rs     # Implementación Win32 (OpenProcess, VirtualAllocEx, ...)
│   └── stub.rs        # Adaptadores simulados para compilaciones no-Windows y tests
├── ui/                # Capa de presentación (egui + renderizador glow OpenGL)
│   ├── app.rs         # Ventana principal, layout y enlace de eventos
│   ├── i18n.rs        # Idiomas, textos de la interfaz, secciones y ayudas
│   ├── settings.rs    # Preferencias guardadas del usuario (idioma)
│   ├── theme.rs       # Tokens de diseño: colores, radios, tamaños de texto
│   └── widgets.rs     # Interruptor, botón de ayuda "?", chip de atajo, indicador de estado
├── build.rs           # (raíz del proyecto) incrusta icono, versión y manifiesto en el .exe
├── lib.rs             # Exportación pública de módulos
└── main.rs            # Punto de entrada y arranque de eframe
```
