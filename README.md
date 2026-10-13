# DOA6 Trainer (Rust)

Reescritura en Rust del trainer de FLiNG para **Dead or Alive 6 (v1.0–v1.20 y Dead or Alive 6 Last Round / DOA6LR.exe)** con 16 trucos en tiempo de ejecución, desbloqueo persistente de DLCs y gui `egui`.
Se ejecuta en Windows 10/11 x64 y bajo Wine/Proton en Lutris.

## Compilar

En Windows (con [rustup](https://rustup.rs)):

```powershell
cargo build --release
# -> target\release\doa6-trainer.exe
```

Desde Linux (compilación cruzada):

```sh
sudo pacman -S mingw-w64-gcc        # o el paquete equivalente de tu distribución
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# -> target/x86_64-pc-windows-gnu/release/doa6-trainer.exe
```

`cargo test` ejecuta los tests en cualquier sistema: la lógica se prueba contra un proceso simulado.

### Prueba de integración bajo Wine

```sh
tools/wine-smoke/run.sh
```

Compila un `DOA6.exe` falso (`tools/wine-smoke/fake_doa6.c`) que contiene las mismas instrucciones
que buscan las firmas y las ejecuta en bucle, y lanza `examples/wine_smoke.rs`, que usa el trainer
real (núcleo + infraestructura Win32) para activar cada truco, comprobar su efecto en la memoria del
proceso y verificar que todo se restaura. No sustituye a probar con el juego: solo valida el motor.

## Uso

1. Abre el juego y el trainer (en cualquier orden): el trainer detecta `DOA6.exe` solo.
2. Activa los trucos con los botones o con los atajos (`Num 0-9`, `Ctrl+Num 1-6`).
3. `Ctrl+Shift+Inicio` activa o desactiva todos los atajos.
4. Al cerrar el trainer se restaura el código original del juego.

Si aparece un error de memoria, ejecuta el trainer como administrador.

## Arquitectura

```
src/
├── domain/            Reglas puras, sin dependencias del SO
│   ├── pattern.rs     Firmas AOB ("48 8B * s1") y plantillas de bytes
│   ├── x86.rs         Codificación de saltos (jmp rel32 / jmp absoluto)
│   ├── cheat.rs       Modelo: Cheat, Effect, Patch, Hook, GameCatalog
│   └── doa6.rs        Los 16 trucos de DOA6 como datos
├── application/       Casos de uso; solo conoce el dominio y los puertos
│   ├── ports.rs       Interfaces: GameMemory, ProcessFinder, HotkeyListener
│   ├── patcher.rs     Escaneo de firmas, aplicar/revertir parches y code caves
│   ├── trainer.rs     Activar/desactivar trucos, instalar hooks, restaurar
│   └── controller.rs  Estado compartido + hilo de detección del juego y atajos
├── infrastructure/    Implementación de los puertos (windows.rs: Win32; stub.rs: otros SO)
├── ui/                Interfaz egui (renderizador OpenGL/glow)
├── lib.rs             Expone todas las capas
└── main.rs            Raíz de composición
```

Las dependencias apuntan siempre hacia dentro: `ui` e `infrastructure` → `application` → `domain`.
Para añadir un juego nuevo basta con escribir otro catálogo como `domain/doa6.rs`.
