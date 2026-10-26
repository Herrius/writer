#!/usr/bin/env python3
"""Orquestador de gates de writer.

Una sola definición de las comprobaciones, ejecutable en local y en CI.
Los grupos son:

  calidad    Rust: formato, lints y tests (los mismos flags que en CI).
  interfaz   pnpm: instalación reproducible, tipos TS, build de UI y cáscara Tauri.
  e2e        Playwright: flujos de demo en Chromium.
  e2e-tauri  Tauri/WebDriver: flujo real de escritorio en Linux.
  empaquetado Tauri: binario e instaladores de la plataforma.
  producto   Características del software: esquema, fixtures, determinismo,
             hallazgos esperados del fixture roto, documentación y presupuesto.
  seguridad  Cadena de suministro y secretos: cargo-deny, gitleaks y zizmor.

Uso:
    python scripts/ci.py calidad
    python scripts/ci.py interfaz
    python scripts/ci.py producto --presupuesto 60
    python scripts/ci.py seguridad
    python scripts/ci.py todo

Códigos de salida: 0 = verde, 1 = gate roto, 2 = uso o herramienta ausente.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
from pathlib import Path
from urllib.parse import unquote

RAIZ = Path(__file__).resolve().parent.parent
GENERADOR = RAIZ / "scripts" / "generar-fixtures.py"
FIXTURES = RAIZ / "fixtures"
PEQUENO = FIXTURES / "proyecto-pequeno"
ROTO = FIXTURES / "proyecto-roto"
DOC_ESQUEMA = RAIZ / "docs" / "esquema-v1.md"
ESPERADOS = ROTO / "esperados.txt"

# Parámetros canónicos del fixture versionado (ver fixtures/README.md).
ESCENAS_PEQUENO = 3
SEMILLA_PEQUENO = 1
ESCENAS_RENDIMIENTO = 1000
PRESUPUESTO_POR_DEFECTO = 60.0
BUNDLE = (
    RAIZ
    / "apps"
    / "desktop"
    / "src-tauri"
    / "target"
    / "release"
    / "bundle"
)


class Fallo(Exception):
    """Un gate no se cumple."""


def _entorno() -> dict[str, str]:
    entorno = os.environ.copy()
    entorno["RUSTFLAGS"] = "-D warnings"
    entorno.setdefault("CARGO_TERM_COLOR", "never")
    local = Path.home() / ".local" / "bin"
    if local.is_dir():
        entorno["PATH"] = f"{local}{os.pathsep}{entorno.get('PATH', '')}"
    return entorno


def pnpm_bin() -> str:
    """Resuelve pnpm en Linux y en Windows (donde el shim es `pnpm.cmd`)."""
    return shutil.which("pnpm.cmd") or shutil.which("pnpm") or "pnpm"


def ejecutar(
    comando: list[str],
    *,
    capturar: bool = False,
    timeout: float | None = None,
) -> subprocess.CompletedProcess[str]:
    # En Windows, `pnpm` es un shim `.cmd`; CreateProcess no lo ejecuta directo.
    if (
        sys.platform == "win32"
        and comando
        and comando[0].lower().endswith((".cmd", ".bat"))
    ):
        comando = ["cmd", "/c", *comando]
    print("$ " + " ".join(comando), flush=True)
    try:
        proceso = subprocess.run(
            comando,
            cwd=RAIZ,
            env=_entorno(),
            text=True,
            encoding="utf-8",
            errors="replace",
            capture_output=capturar,
            timeout=timeout,
        )
    except FileNotFoundError as error:
        raise Fallo(f"herramienta no encontrada: {comando[0]}") from error
    except subprocess.TimeoutExpired as error:
        raise Fallo(f"tiempo agotado ({timeout} s): {' '.join(comando)}") from error
    if not capturar and proceso.returncode != 0:
        raise Fallo(f"falló con código {proceso.returncode}: {' '.join(comando)}")
    return proceso


def python(*argumentos: str, capturar: bool = False) -> subprocess.CompletedProcess[str]:
    return ejecutar([sys.executable, str(GENERADOR), *argumentos], capturar=capturar)


def _cargo(*argumentos: str) -> None:
    ejecutar(["cargo", *argumentos])


# --------------------------------------------------------------------------- #
# Calidad
# --------------------------------------------------------------------------- #


def calidad() -> None:
    _cargo("fmt", "--all", "--check")
    _cargo("clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings")
    _cargo("test", "--workspace", "--all-targets", "--locked")


# --------------------------------------------------------------------------- #
# Interfaz
# --------------------------------------------------------------------------- #


def interfaz() -> None:
    if shutil.which("pnpm") is None and shutil.which("pnpm.cmd") is None:
        raise Fallo("falta pnpm (instala pnpm 12; npm/npx no se usan)")
    ejecutar([pnpm_bin(), "install", "--frozen-lockfile"])
    ejecutar([pnpm_bin(), "typecheck"])
    ejecutar([pnpm_bin(), "ui:test"])
    ejecutar([pnpm_bin(), "ui:build"])
    ejecutar(
        [
            "cargo",
            "fmt",
            "--manifest-path",
            str(RAIZ / "apps" / "desktop" / "src-tauri" / "Cargo.toml"),
            "--check",
        ]
    )
    ejecutar(
        [
            "cargo",
            "clippy",
            "--manifest-path",
            str(RAIZ / "apps" / "desktop" / "src-tauri" / "Cargo.toml"),
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ]
    )
    ejecutar(
        [
            "cargo",
            "check",
            "--manifest-path",
            str(RAIZ / "apps" / "desktop" / "src-tauri" / "Cargo.toml"),
            "--locked",
        ]
    )
    ejecutar(
        [
            "cargo",
            "test",
            "--manifest-path",
            str(RAIZ / "apps" / "desktop" / "src-tauri" / "Cargo.toml"),
            "--locked",
        ]
    )
    print("Interfaz: tipos, build de UI, cáscara Tauri y comandos OK", flush=True)


# --------------------------------------------------------------------------- #
# Producto
# --------------------------------------------------------------------------- #


def _huellas(raiz: Path) -> dict[str, str]:
    return {
        ruta.relative_to(raiz).as_posix(): hashlib.sha256(ruta.read_bytes()).hexdigest()
        for ruta in sorted(raiz.rglob("*"))
        if ruta.is_file()
    }


def _comparar_arboles(esperado: Path, generado: Path) -> None:
    referencia = _huellas(esperado)
    obtenido = _huellas(generado)
    faltan = sorted(set(referencia) - set(obtenido))
    sobran = sorted(set(obtenido) - set(referencia))
    distintos = sorted(clave for clave in referencia.keys() & obtenido.keys() if referencia[clave] != obtenido[clave])
    problemas: list[str] = []
    if faltan:
        problemas.append(f"faltan {len(faltan)} archivo(s): {', '.join(faltan[:5])}")
    if sobran:
        problemas.append(f"sobran {len(sobran)} archivo(s): {', '.join(sobran[:5])}")
    if distintos:
        problemas.append(f"{len(distintos)} archivo(s) distintos: {', '.join(distintos[:5])}")
    if problemas:
        raise Fallo(
            "el fixture versionado no coincide con el generador: "
            + "; ".join(problemas)
            + f". Regenera con: python scripts/generar-fixtures.py --destino {PEQUENO} "
            f"--escenas {ESCENAS_PEQUENO} --semilla {SEMILLA_PEQUENO} --forzar"
        )


def determinismo() -> None:
    with tempfile.TemporaryDirectory() as temporal:
        destino = Path(temporal) / "proyecto-pequeno"
        python(
            "--destino",
            str(destino),
            "--escenas",
            str(ESCENAS_PEQUENO),
            "--semilla",
            str(SEMILLA_PEQUENO),
        )
        _comparar_arboles(PEQUENO, destino)
    print(f"Determinismo: OK ({ESCENAS_PEQUENO} escenas, semilla {SEMILLA_PEQUENO})", flush=True)


def _leer_esperados() -> list[str]:
    if not ESPERADOS.is_file():
        raise Fallo(f"falta {ESPERADOS}")
    return [
        linea.strip()
        for linea in ESPERADOS.read_text(encoding="utf-8").splitlines()
        if linea.strip() and not linea.lstrip().startswith("#")
    ]


def hallazgos_esperados() -> None:
    proceso = python("--verificar", str(ROTO), "--json", capturar=True)
    if proceso.returncode == 0:
        raise Fallo("el fixture roto no produjo errores; el verificador se ha vuelto ciego")
    try:
        datos = json.loads(proceso.stdout)
    except json.JSONDecodeError as error:
        raise Fallo(f"salida del verificador no es JSON: {error}") from error

    errores = datos.get("errores", [])
    esperados = _leer_esperados()
    restantes = list(errores)
    faltan: list[str] = []
    for esperado in esperados:
        for indice, error in enumerate(restantes):
            if esperado in error:
                del restantes[indice]
                break
        else:
            faltan.append(esperado)

    problemas: list[str] = []
    if faltan:
        problemas.append("no se detectaron: " + " | ".join(faltan))
    if restantes:
        problemas.append("errores no esperados: " + " | ".join(restantes))
    if len(errores) != len(esperados):
        problemas.append(f"se esperaban {len(esperados)} errores y hay {len(errores)}")
    if problemas:
        raise Fallo("fixture roto fuera del contrato: " + "; ".join(problemas))
    print(
        f"Hallazgos esperados: OK ({len(esperados)} errores, {len(datos.get('avisos', []))} avisos)",
        flush=True,
    )


def rendimiento(presupuesto: float) -> None:
    with tempfile.TemporaryDirectory() as temporal:
        destino = Path(temporal) / "proyecto-1000"
        inicio = time.monotonic()
        generacion = python("--destino", str(destino), "--escenas", str(ESCENAS_RENDIMIENTO), capturar=True)
        if generacion.returncode != 0:
            raise Fallo(f"no se pudo generar el fixture de rendimiento: {generacion.stderr.strip()}")
        verificacion = python("--verificar", str(destino), capturar=True)
        if verificacion.returncode != 0:
            raise Fallo(f"el fixture de rendimiento no verifica: {verificacion.stdout.strip()}")
        duracion = time.monotonic() - inicio
    if duracion > presupuesto:
        raise Fallo(
            f"generación + verificación de {ESCENAS_RENDIMIENTO} escenas: "
            f"{duracion:.1f} s > {presupuesto:.0f} s"
        )
    print(
        f"Rendimiento: OK ({ESCENAS_RENDIMIENTO} escenas generadas y verificadas "
        f"en {duracion:.1f} s de {presupuesto:.0f} s)",
        flush=True,
    )


RE_ENLACE = re.compile(r"\[[^\]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
DIRECTORIOS_IGNORADOS = {".git", "target", "node_modules", ".venv", "__pycache__"}


def enlaces_documentacion() -> None:
    problemas: list[str] = []
    for ruta in sorted(RAIZ.rglob("*.md")):
        partes = set(ruta.relative_to(RAIZ).parts)
        if partes & DIRECTORIOS_IGNORADOS or "proyecto-1000" in partes:
            continue
        texto = ruta.read_text(encoding="utf-8", errors="replace")
        dentro_de_bloque = False
        for numero, linea in enumerate(texto.splitlines(), start=1):
            if linea.strip().startswith("```"):
                dentro_de_bloque = not dentro_de_bloque
                continue
            if dentro_de_bloque:
                continue
            for destino in RE_ENLACE.findall(linea):
                if destino.startswith(("#", "http://", "https://", "mailto:", "tel:")):
                    continue
                limpio = unquote(destino.split("#", 1)[0])
                if not limpio:
                    continue
                objetivo = (ruta.parent / limpio).resolve()
                if not objetivo.exists():
                    problemas.append(f"{ruta.relative_to(RAIZ)}:{numero}: enlace roto -> {destino}")
    if problemas:
        raise Fallo("enlaces rotos en la documentación: " + " | ".join(problemas))
    print("Documentación: enlaces relativos OK", flush=True)


def motor() -> None:
    """Verifica los fixtures con el binario Rust, no solo con el script Python."""
    ejecutar(
        [
            "cargo",
            "run",
            "--quiet",
            "--locked",
            "--bin",
            "writer",
            "--",
            "verify",
            str(PEQUENO),
        ]
    )
    proceso = ejecutar(
        [
            "cargo",
            "run",
            "--quiet",
            "--locked",
            "--bin",
            "writer",
            "--",
            "verify",
            str(ROTO),
            "--json",
        ],
        capturar=True,
    )
    if proceso.returncode == 0:
        raise Fallo("el verificador Rust no detectó los defectos del fixture roto")
    try:
        datos = json.loads(proceso.stdout)
    except json.JSONDecodeError as error:
        raise Fallo(f"salida del verificador Rust no es JSON: {error}") from error
    errores = datos.get("errores", [])
    esperados = _leer_esperados()
    faltan = [esperado for esperado in esperados if not any(esperado in error for error in errores)]
    if faltan:
        raise Fallo("el verificador Rust no detectó: " + " | ".join(faltan))
    if len(errores) != len(esperados):
        raise Fallo(
            f"el verificador Rust devolvió {len(errores)} errores; "
            f"el contrato fija {len(esperados)}"
        )
    print(
        f"CLI Rust: fixture pequeño OK · {len(errores)} hallazgos esperados en el roto",
        flush=True,
    )


def producto(presupuesto: float) -> None:
    python("--autoprueba")
    python("--verificar-docs", str(DOC_ESQUEMA))
    python("--verificar", str(PEQUENO))
    determinismo()
    hallazgos_esperados()
    rendimiento(presupuesto)
    motor()
    enlaces_documentacion()


# --------------------------------------------------------------------------- #
# Seguridad
# --------------------------------------------------------------------------- #

HERRAMIENTAS = {
    "cargo-deny": "cargo install cargo-deny --locked (o usa la acción en CI)",
    "gitleaks": "winget install Gitleaks.Gitleaks · brew install gitleaks",
    "zizmor": "winget install zizmor.zizmor · pipx install zizmor",
}


def _exigir_herramientas() -> None:
    requeridas = {
        nombre: pista
        for nombre, pista in HERRAMIENTAS.items()
        if nombre != "zizmor" or (RAIZ / ".github").is_dir()
    }
    ausentes = [
        f"{nombre} ({pista})"
        for nombre, pista in requeridas.items()
        if shutil.which(nombre) is None
    ]
    if ausentes:
        raise Fallo("faltan herramientas de seguridad: " + " | ".join(ausentes))


def _copiar_versionados(destino: Path) -> None:
    """Copia solo archivos versionados para el escaneo de árbol.

    `gitleaks dir .` recorre `target/`, `node_modules/` y otros artefactos no
    versionados, lo que produce falsos positivos y hace que el gate dependa del
    estado local. El árbol de trabajo versionado es la superficie que importa.
    """
    listado = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=RAIZ,
        capture_output=True,
        check=True,
    ).stdout
    for crudo in listado.split(b"\0"):
        if not crudo:
            continue
        relativa = Path(os.fsdecode(crudo))
        origen = RAIZ / relativa
        if not origen.is_file():
            print(f"aviso: archivo versionado ausente en el árbol: {relativa}", flush=True)
            continue
        copia = destino / relativa
        copia.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(origen, copia)


def seguridad() -> None:
    _exigir_herramientas()
    ejecutar(
        [
            "cargo",
            "deny",
            "--all-features",
            "--manifest-path",
            str(RAIZ / "Cargo.toml"),
            "check",
        ]
    )
    ejecutar(
        [
            "cargo",
            "deny",
            "--manifest-path",
            str(RAIZ / "apps" / "desktop" / "src-tauri" / "Cargo.toml"),
            "check",
        ]
    )
    ejecutar(["gitleaks", "git", ".", "--redact", "--no-banner"])
    with tempfile.TemporaryDirectory() as temporal:
        arbol = Path(temporal) / "versionados"
        arbol.mkdir()
        _copiar_versionados(arbol)
        ejecutar(["gitleaks", "dir", str(arbol), "--redact", "--no-banner"])
    if (RAIZ / ".github").is_dir():
        ejecutar(["zizmor", "."])
    else:
        print("zizmor: sin .github/ ni workflows; se omite", flush=True)


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #

def _ar_contenido(archivo: Path, miembro: str) -> bytes:
    ar = shutil.which("ar")
    if ar is None:
        raise Fallo("falta `ar` para inspeccionar el .deb")
    proceso = subprocess.run(
        [ar, "p", str(archivo), miembro],
        capture_output=True,
        check=False,
    )
    if proceso.returncode != 0:
        raise Fallo(
            f"no se pudo extraer {miembro} de {archivo.name}: "
            + proceso.stderr.decode("utf-8", "replace").strip()
        )
    return proceso.stdout


def _comprobar_deb() -> None:
    paquetes = sorted((BUNDLE / "deb").glob("*.deb"))
    if not paquetes:
        raise Fallo("empaquetado: no se generó ningún .deb")
    paquete = paquetes[-1]
    datos = _ar_contenido(paquete, "data.tar.gz")
    with tarfile.open(fileobj=io.BytesIO(datos), mode="r:gz") as tar:
        miembros = {m.name.lstrip("./"): m for m in tar.getmembers()}
        binario = miembros.get("usr/bin/writer-desktop")
        if binario is None or not binario.isfile():
            raise Fallo("el .deb no contiene usr/bin/writer-desktop")
        if not binario.mode & 0o111:
            raise Fallo("el ejecutable del .deb no tiene permiso de ejecución")
        escritorio = miembros.get("usr/share/applications/writer.desktop")
        if escritorio is None:
            raise Fallo("el .deb no contiene writer.desktop")
        contenido = tar.extractfile(escritorio)
        if contenido is None or b"Exec=writer-desktop" not in contenido.read():
            raise Fallo("writer.desktop no apunta a writer-desktop")
        iconos = [
            nombre
            for nombre in miembros
            if nombre.startswith("usr/share/icons/hicolor/") and nombre.endswith(".png")
        ]
        if len(iconos) < 3:
            raise Fallo("el .deb no incluye los iconos esperados")

        control = _ar_contenido(paquete, "control.tar.gz")
        with tarfile.open(fileobj=io.BytesIO(control), mode="r:gz") as control_tar:
            md5 = control_tar.extractfile("md5sums")
            if md5 is not None:
                for linea in md5.read().decode("utf-8").splitlines():
                    if not linea.strip():
                        continue
                    esperado, ruta = re.split(r"\s+", linea.strip(), maxsplit=1)
                    ruta = ruta.lstrip("./")
                    miembro = miembros.get(ruta)
                    if miembro is None:
                        raise Fallo(f"md5sums apunta a un archivo ausente: {ruta}")
                    archivo = tar.extractfile(miembro)
                    if archivo is None:
                        raise Fallo(f"no se pudo leer {ruta} del .deb")
                    actual = hashlib.md5(archivo.read()).hexdigest()
                    if actual != esperado:
                        raise Fallo(f"checksum incorrecto en el .deb: {ruta}")
    print(f"Empaquetado: {paquete.name} verificado (estructura, permisos, md5)")


def _comprobar_rpm() -> None:
    paquetes = sorted((BUNDLE / "rpm").glob("*.rpm"))
    if not paquetes:
        raise Fallo("empaquetado: no se generó ningún .rpm")
    paquete = paquetes[-1]
    rpm = shutil.which("rpm")
    if rpm is None:
        raise Fallo("falta `rpm` para inspeccionar el paquete")
    listado = subprocess.run(
        [rpm, "-qplv", str(paquete)],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    if listado.returncode != 0:
        raise Fallo(f"rpm -qplv falló: {listado.stderr.strip()}")
    lineas = listado.stdout.splitlines()
    binario = next(
        (linea for linea in lineas if linea.endswith(" /usr/bin/writer-desktop")),
        None,
    )
    if binario is None or not binario.split()[0].startswith("-rwx"):
        raise Fallo("el RPM no contiene un writer-desktop ejecutable")
    if not any(linea.endswith(" /usr/share/applications/writer.desktop") for linea in lineas):
        raise Fallo("el RPM no contiene writer.desktop")
    iconos = [
        linea
        for linea in lineas
        if "/usr/share/icons/hicolor/" in linea and linea.endswith(".png")
    ]
    if len(iconos) < 3:
        raise Fallo("el RPM no incluye los iconos esperados")
    firma = subprocess.run(
        [rpm, "-K", "--nosignature", str(paquete)],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    if firma.returncode != 0:
        raise Fallo(f"los digests del RPM no son válidos: {firma.stderr.strip()}")
    print(f"Empaquetado: {paquete.name} verificado (estructura, permisos, digests)")


def _comprobar_windows() -> None:
    """Comprueba que Tauri generó instaladores Windows utilizables.

    No sustituye una instalación real en una máquina limpia, pero detecta el
    fallo más común del empaquetado: que no se genere ningún artefacto o que el
    archivo no sea un PE/MSI válido.
    """
    msi = sorted((BUNDLE / "msi").glob("*.msi"))
    nsis = sorted((BUNDLE / "nsis").glob("*.exe"))
    if not msi and not nsis:
        raise Fallo("Windows: Tauri no generó ningún MSI ni instalador NSIS")
    for paquete in [*msi, *nsis]:
        if paquete.stat().st_size < 1_000_000:
            raise Fallo(f"Windows: instalador sospechosamente pequeño: {paquete}")
    for paquete in nsis:
        with paquete.open("rb") as archivo:
            cabecera = archivo.read(2)
        if cabecera != b"MZ":
            raise Fallo(f"Windows: el instalador NSIS no es un PE válido: {paquete}")
    for paquete in msi:
        with paquete.open("rb") as archivo:
            cabecera = archivo.read(8)
        if cabecera != b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1":
            raise Fallo(f"Windows: el MSI no tiene cabecera OLE: {paquete}")
    nombres = ", ".join(paquete.name for paquete in [*msi, *nsis])
    print(f"Empaquetado Windows: {nombres} verificados")


def _comprobar_paquetes() -> None:
    if sys.platform == "win32":
        _comprobar_windows()
        return
    if not sys.platform.startswith("linux"):
        print("Empaquetado: verificación estructural omitida fuera de Linux/Windows")
        return
    _comprobar_deb()
    _comprobar_rpm()


def e2e() -> None:
    if shutil.which("pnpm") is None and shutil.which("pnpm.cmd") is None:
        raise Fallo("falta pnpm (instala pnpm 12; npm/npx no se usan)")
    ejecutar([pnpm_bin(), "install", "--frozen-lockfile"])
    ejecutar([pnpm_bin(), "e2e"])


def e2e_tauri() -> None:
    if not sys.platform.startswith("linux"):
        print("E2E Tauri: omitido fuera de Linux")
        return
    if shutil.which("tauri-driver") is None:
        raise Fallo(
            "falta tauri-driver (cargo install tauri-driver --version 2.0.6 --locked)"
        )
    if shutil.which("WebKitWebDriver") is None:
        raise Fallo("falta WebKitWebDriver")
    if shutil.which("pnpm") is None and shutil.which("pnpm.cmd") is None:
        raise Fallo("falta pnpm (instala pnpm 12; npm/npx no se usan)")
    ejecutar([pnpm_bin(), "install", "--frozen-lockfile"])
    ejecutar([pnpm_bin(), "e2e:tauri"])


def empaquetado() -> None:
    if shutil.which("pnpm") is None and shutil.which("pnpm.cmd") is None:
        raise Fallo("falta pnpm (instala pnpm 12; npm/npx no se usan)")
    ejecutar([pnpm_bin(), "build"])
    _comprobar_paquetes()


GRUPOS = {
    "calidad": calidad,
    "interfaz": interfaz,
    "e2e": e2e,
    "e2e-tauri": e2e_tauri,
    "empaquetado": empaquetado,
    "producto": producto,
    "seguridad": seguridad,
}


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser(
        description="Ejecuta los gates de writer en local o en CI.",
        epilog=__doc__.split("Uso:")[0].strip(),
    )
    parser.add_argument(
        "grupo",
        nargs="?",
        default="todo",
        choices=[*GRUPOS, "todo"],
        help="grupo de comprobaciones (por defecto: todo)",
    )
    parser.add_argument(
        "--presupuesto",
        type=float,
        default=PRESUPUESTO_POR_DEFECTO,
        metavar="SEGUNDOS",
        help=f"presupuesto de rendimiento (por defecto {PRESUPUESTO_POR_DEFECTO:.0f} s)",
    )
    opciones = parser.parse_args(argv)

    grupos = list(GRUPOS) if opciones.grupo == "todo" else [opciones.grupo]
    try:
        for grupo in grupos:
            print(f"\n== {grupo.upper()} ==", flush=True)
            inicio = time.monotonic()
            if grupo == "producto":
                producto(opciones.presupuesto)
            else:
                GRUPOS[grupo]()
            print(f"== {grupo}: OK en {time.monotonic() - inicio:.1f} s ==", flush=True)
    except Fallo as error:
        print(f"\nGATE ROTO: {error}", file=sys.stderr)
        return 1
    print("\nTodos los gates en verde.", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
