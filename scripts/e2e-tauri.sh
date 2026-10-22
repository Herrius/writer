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
if ! command -v setsid >/dev/null 2>&1; then
  echo "falta setsid (util-linux) para aislar el driver y la app" >&2
  exit 2
fi
if ! command -v xdotool >/dev/null 2>&1; then
  echo "falta xdotool para probar el cierre real de la ventana" >&2
  exit 2
fi

temporal="$(mktemp -d)"
driver_pid=""
puerto_nativo=""
aplicacion="$raiz/apps/desktop/src-tauri/target/release/writer-desktop"
arrollador="$temporal/writer-desktop-e2e"
cat >"$arrollador" <<EOF
#!/bin/bash
export GDK_BACKEND=x11
export XDG_SESSION_TYPE=x11
exec "$aplicacion" "\$@"
EOF
chmod +x "$arrollador"

limpiar() {
  # El driver y la app comparten grupo de procesos: cerrar el grupo evita
  # ventanas y procesos huérfanos entre repeticiones.
  if [[ -n "$driver_pid" ]]; then
    kill -- -"$driver_pid" 2>/dev/null || kill "$driver_pid" 2>/dev/null || true
    wait "$driver_pid" 2>/dev/null || true
  fi
  pkill -f "$aplicacion" 2>/dev/null || true
  if [[ -n "$puerto_nativo" ]]; then
    pkill -f "WebKitWebDriver.*--port=$puerto_nativo" 2>/dev/null || true
  fi
  rm -rf "$temporal"
}

detener_app() {
  # Cada escenario arranca una app nueva. Esperar a que el proceso anterior
  # desaparezca evita que WebDriver reutilice una sesión de una ventana muerta.
  pkill -f "$aplicacion" 2>/dev/null || true
  for _ in $(seq 1 25); do
    if ! pgrep -f "$aplicacion" >/dev/null 2>&1; then
      break
    fi
    sleep 0.2
  done
  pkill -9 -f "$aplicacion" 2>/dev/null || true
  sleep 1
}
trap limpiar EXIT INT TERM

cargo build --quiet --locked --bin writer
"$raiz/target/debug/writer" init "$temporal/novela" --titulo "E2E Tauri" >/dev/null

# La feature `e2e` solo existe en esta compilación: la app de producción no
# autoabre proyectos ni conserva el hook.
pnpm --dir apps/desktop exec tauri build --no-bundle --features e2e >/dev/null

export WRITER_E2E_PROJECT="$temporal/novela"
export WRITER_DESKTOP_BIN="$arrollador"
export GDK_BACKEND=x11
export WEBKIT_DISABLE_COMPOSITING_MODE=1

puerto="$(python3 - <<'PY'
import socket
with socket.socket() as s:
    s.bind(("127.0.0.1", 0))
    print(s.getsockname()[1])
PY
)"
export WRITER_E2E_PORT="$puerto"

puerto_nativo="$(python3 - <<'PY'
import socket
with socket.socket() as s:
    s.bind(("127.0.0.1", 0))
    print(s.getsockname()[1])
PY
)"

if (exec 3<>"/dev/tcp/127.0.0.1/$puerto") 2>/dev/null; then
  echo "el puerto $puerto ya está ocupado; no se puede aislar el E2E" >&2
  exit 2
fi

setsid tauri-driver \
  --port "$puerto" \
  --native-port "$puerto_nativo" \
  --native-host 127.0.0.1 \
  --native-driver "$(command -v WebKitWebDriver)" \
  >"$temporal/tauri-driver.log" 2>&1 &
driver_pid=$!

listo=0
for _ in $(seq 1 50); do
  if ! kill -0 "$driver_pid" 2>/dev/null; then
    echo "tauri-driver terminó antes de escuchar; log:" >&2
    cat "$temporal/tauri-driver.log" >&2
    exit 1
  fi
  if (exec 3<>"/dev/tcp/127.0.0.1/$puerto") 2>/dev/null; then
    exec 3>&-
    listo=1
    break
  fi
  sleep 0.2
done
if [[ "$listo" -ne 1 ]]; then
  echo "tauri-driver no escuchó en el puerto $puerto; log:" >&2
  cat "$temporal/tauri-driver.log" >&2
  exit 1
fi

detener_app
WRITER_E2E_MODE=guardar node e2e-tauri/tauri.test.mjs

# Cada escenario arranca la app de cero: WebKit es más fiable con una sola
# sesión de teclado por proceso y así se prueba también el arranque limpio.
detener_app
WRITER_E2E_MODE=cerrar node e2e-tauri/tauri.test.mjs
