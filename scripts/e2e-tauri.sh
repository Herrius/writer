#!/usr/bin/env bash
# E2E de escritorio Linux: tauri-driver + WebKitWebDriver.
set -euo pipefail

raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$raiz"

if ! command -v tauri-driver >/dev/null 2>&1; then
  echo "falta tauri-driver (cargo install tauri-driver --version 2.0.6 --locked)" >&2
  exit 2
fi
if ! command -v WebKitWebDriver >/dev/null 2>&1; then
  echo "falta WebKitWebDriver" >&2
  exit 2
fi

temporal="$(mktemp -d)"
driver_pid=""
limpiar() {
  if [[ -n "$driver_pid" ]]; then
    kill "$driver_pid" 2>/dev/null || true
    wait "$driver_pid" 2>/dev/null || true
  fi
  rm -rf "$temporal"
}
trap limpiar EXIT

cargo build --quiet --locked --bin writer
"$raiz/target/debug/writer" init "$temporal/novela" --titulo "E2E Tauri" >/dev/null

pnpm --dir apps/desktop exec tauri build --no-bundle >/dev/null

export WRITER_E2E_PROJECT="$temporal/novela"
export WRITER_DESKTOP_BIN="$raiz/apps/desktop/src-tauri/target/release/writer-desktop"
export GDK_BACKEND=x11
export WEBKIT_DISABLE_COMPOSITING_MODE=1

tauri-driver \
  --port 4444 \
  --native-driver "$(command -v WebKitWebDriver)" \
  >"$temporal/tauri-driver.log" 2>&1 &
driver_pid=$!

for _ in $(seq 1 50); do
  if (exec 3<>/dev/tcp/127.0.0.1/4444) 2>/dev/null; then
    exec 3>&-
    break
  fi
  sleep 0.2
done

node e2e-tauri/tauri.test.mjs
