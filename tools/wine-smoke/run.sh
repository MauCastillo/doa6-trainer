#!/bin/sh
# Prueba de integración del trainer bajo Wine con un DOA6.exe falso.
# Uso: tools/wine-smoke/run.sh   (necesita wine y mingw-w64-gcc)
set -eu

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
WORK=${WORK:-"$ROOT/target/wine-smoke"}
export WINEPREFIX=${WINEPREFIX:-"$WORK/prefix"}
export WINEDEBUG=-all

mkdir -p "$WORK"
x86_64-w64-mingw32-gcc -O1 -Wall -o "$WORK/DOA6.exe" "$ROOT/tools/wine-smoke/fake_doa6.c"
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" --target x86_64-pc-windows-gnu --example wine_smoke
cp "$ROOT/target/x86_64-pc-windows-gnu/debug/examples/wine_smoke.exe" "$WORK/"

cd "$WORK"
rm -f fake_doa6_state.txt
[ -d "$WINEPREFIX" ] || wineboot --init >/dev/null 2>&1

wine DOA6.exe &
GAME=$!
trap 'kill $GAME 2>/dev/null; wineserver -k 2>/dev/null || true' EXIT
for _ in $(seq 50); do [ -s fake_doa6_state.txt ] && break; sleep 0.2; done

wine wine_smoke.exe
