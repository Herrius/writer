#!/usr/bin/env python3
"""Genera y verifica fixtures de `writer` sin dependencias externas.

Este script es la implementación de referencia del subconjunto YAML y de los
invariantes de `docs/esquema-v1.md`. Si el script y el documento discrepan, gana
el documento y hay que arreglar el script.

Uso:
    python scripts/generar-fixtures.py --destino fixtures/proyecto-pequeno --escenas 3 --semilla 1
    python scripts/generar-fixtures.py --destino fixtures/proyecto-1000 --escenas 1000
    python scripts/generar-fixtures.py --verificar fixtures/proyecto-pequeno
    python scripts/generar-fixtures.py --verificar fixtures/proyecto-roto --json
    python scripts/generar-fixtures.py --autoprueba
    python scripts/generar-fixtures.py --verificar-docs docs/esquema-v1.md

Códigos de salida: 0 = todo bien (puede haber avisos), 1 = errores, 2 = uso.
"""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import random
import re
import shutil
import sys
import tempfile
import unicodedata
from pathlib import Path
from typing import Any

ALFABETO_ULID = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
RE_ULID = re.compile(r"^[0-7][0-9A-HJKMNP-TV-Z]{25}$")
RE_ENTERO = re.compile(r"^-?\d+$")
RE_CLAVE = re.compile(r"^[a-z_][a-z0-9_]*$")

TIPOS_NOTA = ("tramas", "personajes", "lugares", "cronologia", "objetos")
ESTADOS_VALIDOS = ("borrador", "revision", "final")
POLITICAS = ("auto", "proponer", "bloquear")
RETENCIONES = ("completa", "diaria", "por_tamano")

CAMPOS_PROYECTO = (
    "esquema",
    "id",
    "titulo",
    "autor",
    "idioma",
    "creado",
    "retencion",
    "guardar_prompts",
    "politica_estructura",
    "politica_metadatos",
    "politica_notas",
    "politica_prosa",
    "politica_exportar",
)
CAMPOS_ESCENA = (
    "id",
    "titulo",
    "orden",
    "estado",
    "pov",
    "tramas",
    "personajes",
    "fecha_narrativa",
    "etiquetas",
    "palabras",
)
CAMPOS_NOTA = ("id", "tipo", "titulo", "fija", "etiquetas", "refs")

# 2026-01-01T00:00:00Z, base estable para fixtures reproducibles.
TS_BASE_MS = 1_767_225_600_000


class ErrorEsquema(Exception):
    """Un archivo no cumple el esquema v1."""


# --------------------------------------------------------------------------- #
# ULID
# --------------------------------------------------------------------------- #


def generar_ulid(momento_ms: int, rng: random.Random) -> str:
    if not 0 <= momento_ms < 1 << 48:
        raise ValueError(f"timestamp fuera de rango: {momento_ms}")
    valor = (momento_ms << 80) | rng.getrandbits(80)
    return "".join(
        ALFABETO_ULID[(valor >> (5 * posicion)) & 31] for posicion in range(25, -1, -1)
    )


def es_ulid(texto: str) -> bool:
    return bool(RE_ULID.match(texto))


# --------------------------------------------------------------------------- #
# Subconjunto YAML
# --------------------------------------------------------------------------- #


ESCAPES = {"\\": "\\", '"': '"', "n": "\n", "t": "\t"}
INICIO_PROHIBIDO = "\"'[]{}&*!|>@`%#"


def _texto_entre_comillas(valor: str, contexto: str) -> str:
    """Lee un escalar entre comillas dobles y exige que no quede nada fuera."""
    salida: list[str] = []
    indice = 1
    while indice < len(valor):
        caracter = valor[indice]
        if caracter == "\\":
            if indice + 1 >= len(valor):
                raise ErrorEsquema(f"{contexto}: escape incompleto al final del texto")
            siguiente = valor[indice + 1]
            if siguiente not in ESCAPES:
                raise ErrorEsquema(f"{contexto}: escape no admitido \\{siguiente}")
            salida.append(ESCAPES[siguiente])
            indice += 2
            continue
        if caracter == '"':
            if indice != len(valor) - 1:
                raise ErrorEsquema(
                    f"{contexto}: comilla interior sin escapar o texto tras el cierre"
                )
            return "".join(salida)
        if ord(caracter) < 0x20:
            raise ErrorEsquema(f"{contexto}: carácter de control en el texto")
        salida.append(caracter)
        indice += 1
    raise ErrorEsquema(f"{contexto}: texto entre comillas sin cerrar")


def _dividir_lista(interior: str, contexto: str) -> list[str]:
    elementos: list[str] = []
    actual: list[str] = []
    entre_comillas = False
    escapado = False
    for caracter in interior:
        if escapado:
            actual.append(caracter)
            escapado = False
            continue
        if entre_comillas:
            if caracter == "\\":
                actual.append(caracter)
                escapado = True
            elif caracter == '"':
                actual.append(caracter)
                entre_comillas = False
            else:
                actual.append(caracter)
            continue
        if caracter == '"':
            entre_comillas = True
            actual.append(caracter)
        elif caracter == ",":
            elementos.append("".join(actual))
            actual = []
        else:
            actual.append(caracter)
    if entre_comillas:
        raise ErrorEsquema(f"{contexto}: comillas sin cerrar en lista")
    elementos.append("".join(actual))
    return elementos


def parsear_escalar(texto: str, contexto: str) -> Any:
    valor = texto.strip()
    if not valor:
        raise ErrorEsquema(f"{contexto}: valor vacío")
    primero = valor[0]
    if primero == "'":
        raise ErrorEsquema(
            f"{contexto}: comillas simples no admitidas; usa comillas dobles"
        )
    if primero == '"':
        return _texto_entre_comillas(valor, contexto)
    if primero == "[":
        return parsear_lista(valor, contexto)
    if primero == "{":
        _fuera_de_subconjunto(contexto)
    if primero in INICIO_PROHIBIDO:
        raise ErrorEsquema(f"{contexto}: indicador YAML no admitido: {valor!r}")
    if valor in ("true", "false"):
        return valor == "true"
    if valor == "null":
        return None
    if RE_ENTERO.match(valor):
        return int(valor)
    if primero in "-?":
        raise ErrorEsquema(f"{contexto}: construcción YAML no admitida: {valor!r}")
    if "#" in valor:
        raise ErrorEsquema(
            f"{contexto}: '#' en texto plano inicia un comentario; usa comillas dobles"
        )
    if '"' in valor or "'" in valor:
        raise ErrorEsquema(
            f"{contexto}: comillas sin escapar en un texto plano; usa comillas dobles"
        )
    if any(ord(caracter) < 0x20 for caracter in valor):
        raise ErrorEsquema(f"{contexto}: carácter de control en el texto")
    return valor


def _fuera_de_subconjunto(contexto: str) -> Any:
    raise ErrorEsquema(f"{contexto}: mapas anidados fuera del subconjunto YAML v1")


def parsear_lista(texto: str, contexto: str) -> list[Any]:
    if not texto.endswith("]"):
        raise ErrorEsquema(f"{contexto}: lista sin cerrar")
    interior = texto[1:-1].strip()
    if not interior:
        return []
    elementos = _dividir_lista(interior, contexto)
    for elemento in elementos:
        if elemento.strip().startswith(("[", "{")):
            raise ErrorEsquema(
                f"{contexto}: listas o mapas anidados fuera del subconjunto YAML v1"
            )
    return [parsear_escalar(elemento, contexto) for elemento in elementos]


def parsear_documento(texto: str, contexto: str) -> tuple[dict[str, Any], str]:
    """Devuelve (frontmatter, cuerpo). No tolera BOM ni CRLF: eso lo decide el llamador."""
    if not texto.startswith("---\n"):
        raise ErrorEsquema(f"{contexto}: el archivo no empieza por '---'")
    resto = texto[4:]
    cierre = re.search(r"^---$", resto, flags=re.MULTILINE)
    if cierre is None:
        raise ErrorEsquema(f"{contexto}: frontmatter sin cerrar")
    cabecera = resto[: cierre.start()]
    cuerpo = resto[cierre.end() :]
    if cuerpo.startswith("\n"):
        cuerpo = cuerpo[1:]
    datos: dict[str, Any] = {}
    for numero, linea in enumerate(cabecera.splitlines(), start=1):
        donde = f"{contexto}: línea {numero + 1}"
        if not linea.strip():
            continue
        if linea.startswith((" ", "\t")):
            raise ErrorEsquema(f"{donde}: indentación fuera del subconjunto YAML v1")
        if linea.lstrip().startswith("#"):
            raise ErrorEsquema(f"{donde}: comentarios no admitidos en el frontmatter")
        if linea.startswith("- "):
            raise ErrorEsquema(f"{donde}: listas en bloque fuera del subconjunto YAML v1")
        clave, separador, valor = linea.partition(":")
        if not separador:
            raise ErrorEsquema(f"{donde}: falta ':' tras la clave")
        clave = clave.strip()
        if not RE_CLAVE.match(clave):
            raise ErrorEsquema(f"{donde}: clave no válida: {clave!r}")
        if clave in datos:
            raise ErrorEsquema(f"{donde}: clave duplicada: {clave}")
        datos[clave] = parsear_escalar(valor, donde)
    return datos, cuerpo


def _decodificar(crudo: bytes, origen: object) -> str:
    if crudo.startswith(b"\xef\xbb\xbf"):
        crudo = crudo[3:]
    try:
        return crudo.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ErrorEsquema(f"{origen}: no es UTF-8 válido ({error})") from error


def leer_documento(
    ruta: Path, etiqueta: str | None = None
) -> tuple[dict[str, Any], str]:
    nombre = etiqueta if etiqueta is not None else str(ruta)
    texto = _decodificar(ruta.read_bytes(), nombre)
    texto = texto.replace("\r\n", "\n").replace("\r", "\n")
    return parsear_documento(texto, nombre)


# --------------------------------------------------------------------------- #
# Escritura de documentos
# --------------------------------------------------------------------------- #


def _citar(texto: str) -> str:
    escapado = (
        texto.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", "\\n")
        .replace("\t", "\\t")
    )
    return f'"{escapado}"'


def _elemento_lista(valor: str) -> str:
    if (
        not valor
        or valor != valor.strip()
        or any(caracter in valor for caracter in ',"\'[]{}#')
        or valor[0] in "-?&*!|>@`%"
    ):
        return _citar(valor)
    return valor


def _lista(valores: list[str]) -> str:
    return "[" + ", ".join(_elemento_lista(valor) for valor in valores) + "]"


def _escribir(ruta: Path, contenido: str) -> None:
    ruta.parent.mkdir(parents=True, exist_ok=True)
    with ruta.open("w", encoding="utf-8", newline="\n") as archivo:
        archivo.write(contenido)


def slug(texto: str) -> str:
    normalizado = unicodedata.normalize("NFKD", texto)
    sin_acentos = "".join(c for c in normalizado if not unicodedata.combining(c))
    limpio = re.sub(r"[^a-z0-9]+", "-", sin_acentos.lower()).strip("-")
    return limpio or "sin-titulo"


# --------------------------------------------------------------------------- #
# Generación
# --------------------------------------------------------------------------- #

PALABRAS = (
    "umbral memoria río invierno ventana carta sombra puerto silencio cordillera "
    "cuaderno linterna mercado domingo marea cobre niebla camino tejado farol "
    "piedra humo oficio balcón tormenta arena reloj pizarra galería barca "
    "estación provincia molino acequia taller escalera cuadra helecho"
).split()

NOMBRES_PERSONAJE = (
    "Marta",
    "Iván",
    "Lucía",
    "Tomás",
    "Elena",
    "Bruno",
    "Nadia",
    "Ciro",
)

TEMAS_TRAMA = (
    "La deuda del padre",
    "El mapa incompleto",
    "La casa heredada",
    "El verano que no volvió",
)

LUGARES = ("Puerto Viejo", "La casa del molino", "Estación Central", "El faro")

OBJETOS = ("El cuaderno azul", "La llave de bronce", "La fotografía rota")

HITOS = (
    "Llegada a Puerto Viejo",
    "Incendio del molino",
    "Primera carta sin remitente",
    "Regreso de Iván",
)


def _prosa(rng: random.Random, escena: int, personajes: list[str]) -> str:
    parrafos: list[str] = []
    for _ in range(3):
        oraciones: list[str] = []
        for _ in range(rng.randint(2, 4)):
            palabras = [rng.choice(PALABRAS) for _ in range(rng.randint(8, 14))]
            if rng.random() < 0.5:
                palabras.insert(rng.randrange(len(palabras)), rng.choice(personajes))
            oracion = " ".join(palabras)
            oraciones.append(oracion[0].upper() + oracion[1:] + ".")
        parrafos.append(" ".join(oraciones))
    return f"\n\n".join(parrafos) + f"\n\n<!-- escena de relleno {escena} -->\n"


def crear_proyecto(
    destino: Path,
    escenas: int,
    semilla: int,
    capitulos: int | None,
    forzar: bool,
) -> None:
    if destino.exists():
        if any(destino.iterdir()) and not forzar:
            raise SystemExit(
                f"{destino} ya existe y no está vacío; usa --forzar para sobrescribirlo"
            )
        shutil.rmtree(destino)
    destino.mkdir(parents=True)

    rng = random.Random(semilla)
    contador = 0

    def nuevo_ulid() -> str:
        nonlocal contador
        contador += 1
        return generar_ulid(TS_BASE_MS + contador * 1000, rng)

    id_proyecto = nuevo_ulid()
    personajes = [
        (nuevo_ulid(), NOMBRES_PERSONAJE[i % len(NOMBRES_PERSONAJE)])
        for i in range(max(3, min(8, escenas)))
    ]
    tramas = [(nuevo_ulid(), tema) for tema in TEMAS_TRAMA]
    lugares = [(nuevo_ulid(), nombre) for nombre in LUGARES]
    objetos = [(nuevo_ulid(), nombre) for nombre in OBJETOS]
    hitos = [(nuevo_ulid(), titulo) for titulo in HITOS]

    proyecto = "\n".join(
        [
            "esquema: 1",
            f"id: {id_proyecto}",
            f"titulo: {_citar('Proyecto de prueba ' + str(escenas) + ' escenas')}",
            f"autor: {_citar('Equipo writer')}",
            "idioma: es",
            f"creado: {_citar('2026-01-01T00:00:00.000Z')}",
            "retencion: completa",
            "guardar_prompts: false",
            "politica_estructura: auto",
            "politica_metadatos: auto",
            "politica_notas: auto",
            "politica_prosa: proponer",
            "politica_exportar: proponer",
        ]
    )
    _escribir(destino / "proyecto.yml", proyecto + "\n")

    for identificador, nombre in personajes:
        _escribir(
            destino / "notas" / "personajes" / f"{slug(nombre)}.md",
            "\n".join(
                [
                    "---",
                    f"id: {identificador}",
                    "tipo: personajes",
                    f"titulo: {_citar(nombre)}",
                    "fija: false",
                    f"etiquetas: {_lista(['reparto'])}",
                    f"refs: {_lista([id_proyecto])}",
                    "---",
                    "",
                    f"Ficha de {nombre}. Relleno de fixture.",
                    "",
                ]
            ),
        )

    for identificador, tema in tramas:
        _escribir(
            destino / "notas" / "tramas" / f"{slug(tema)}.md",
            "\n".join(
                [
                    "---",
                    f"id: {identificador}",
                    "tipo: tramas",
                    f"titulo: {_citar(tema)}",
                    "fija: false",
                    f"etiquetas: {_lista(['trama'])}",
                    f"refs: {_lista([id_proyecto])}",
                    "---",
                    "",
                    f"Trama: {tema}.",
                    "",
                ]
            ),
        )

    for carpeta, grupo in (("lugares", lugares), ("objetos", objetos)):
        for identificador, nombre in grupo:
            _escribir(
                destino / "notas" / carpeta / f"{slug(nombre)}.md",
                "\n".join(
                    [
                        "---",
                        f"id: {identificador}",
                        f"tipo: {carpeta}",
                        f"titulo: {_citar(nombre)}",
                        "fija: false",
                        f"refs: {_lista([id_proyecto])}",
                        "---",
                        "",
                        f"Nota de {carpeta}: {nombre}.",
                        "",
                    ]
                ),
            )

    for identificador, titulo in hitos:
        _escribir(
            destino / "notas" / "cronologia" / f"{slug(titulo)}.md",
            "\n".join(
                [
                    "---",
                    f"id: {identificador}",
                    "tipo: cronologia",
                    f"titulo: {_citar(titulo)}",
                    "fija: false",
                    f"refs: {_lista([id_proyecto])}",
                    "---",
                    "",
                    f"Hito: {titulo}.",
                    "",
                ]
            ),
        )

    if capitulos is None:
        capitulos = max(1, min(10, (escenas + 99) // 100))
    capitulos = max(1, min(capitulos, escenas))
    por_capitulo = escenas // capitulos
    resto = escenas % capitulos
    nombres_personajes = [nombre for _, nombre in personajes]
    libro = "01-libro"

    escena_actual = 0
    for indice_capitulo in range(1, capitulos + 1):
        cantidad = por_capitulo + (1 if indice_capitulo <= resto else 0)
        carpeta_capitulo = f"{indice_capitulo:02d}-capitulo"
        for posicion in range(1, cantidad + 1):
            escena_actual += 1
            identificador = nuevo_ulid()
            personaje_pov = personajes[(escena_actual - 1) % len(personajes)]
            tramas_escena = [
                tramas[(escena_actual + desplazamiento) % len(tramas)][0]
                for desplazamiento in range(2)
            ]
            reparto = [
                personajes[(escena_actual + desplazamiento) % len(personajes)][0]
                for desplazamiento in range(3)
            ]
            estado = ESTADOS_VALIDOS[escena_actual % len(ESTADOS_VALIDOS)]
            titulo = f"Escena {escena_actual}"
            año = 1998 + (indice_capitulo - 1) // 12
            mes = (indice_capitulo - 1) % 12 + 1
            cuerpo = "\n".join(
                [
                    "---",
                    f"id: {identificador}",
                    f"titulo: {_citar(titulo)}",
                    f"orden: {posicion * 10}",
                    f"estado: {estado}",
                    f"pov: {personaje_pov[0]}",
                    f"tramas: {_lista(tramas_escena)}",
                    f"personajes: {_lista(reparto)}",
                    f"fecha_narrativa: {_citar(f'{año}-{mes:02d}')}",
                    "---",
                    "",
                    _prosa(rng, escena_actual, nombres_personajes),
                ]
            )
            _escribir(
                destino
                / "manuscrito"
                / libro
                / carpeta_capitulo
                / f"{posicion:03d}-{slug(titulo)}.md",
                cuerpo,
            )

    total_notas = len(personajes) + len(tramas) + len(lugares) + len(objetos) + len(hitos)
    plural_capitulos = "capítulos" if capitulos != 1 else "capítulo"
    print(
        f"Generado {destino}: 1 libro, {capitulos} {plural_capitulos}, {escenas} escenas, "
        f"{total_notas} notas, semilla {semilla}"
    )


# --------------------------------------------------------------------------- #
# Verificación
# --------------------------------------------------------------------------- #


class Informe:
    def __init__(self) -> None:
        self.errores: list[str] = []
        self.avisos: list[str] = []
        self.resumen: dict[str, int] = {}

    def error(self, mensaje: str) -> None:
        self.errores.append(mensaje)

    def aviso(self, mensaje: str) -> None:
        self.avisos.append(mensaje)


def _campo_texto(
    datos: dict[str, Any],
    clave: str,
    etiqueta: object,
    informe: Informe,
    *,
    obligatorio: bool = False,
) -> str | None:
    if clave not in datos:
        if obligatorio:
            informe.error(f"{etiqueta}: falta '{clave}'")
        return None
    valor = datos[clave]
    if not isinstance(valor, str):
        informe.error(f"{etiqueta}: '{clave}' debe ser texto, no {type(valor).__name__}")
        return None
    if obligatorio and not valor.strip():
        informe.error(f"{etiqueta}: '{clave}' está vacío")
        return None
    return valor


def verificar(raiz: Path, imprimir: bool = True) -> Informe:
    informe = Informe()
    proyecto_yml = raiz / "proyecto.yml"
    id_proyecto: str | None = None

    if not proyecto_yml.is_file():
        informe.error("falta proyecto.yml")
    else:
        try:
            _revisar_bytes(proyecto_yml.read_bytes(), "proyecto.yml", informe)
            datos, cuerpo = leer_documento_plano(proyecto_yml)
            if cuerpo.strip():
                informe.aviso("proyecto.yml: contenido después de las claves (se ignora)")
            requeridos = ("esquema", "id")
            for clave in requeridos:
                if clave not in datos:
                    informe.error(f"proyecto.yml: falta '{clave}'")
            if datos.get("esquema") != 1 or isinstance(datos.get("esquema"), bool):
                informe.error(f"proyecto.yml: esquema no soportado: {datos.get('esquema')!r}")
            if "id" in datos:
                if not isinstance(datos["id"], str) or not es_ulid(datos["id"]):
                    informe.error("proyecto.yml: 'id' no es un ULID válido")
                else:
                    id_proyecto = datos["id"]
            _campo_texto(datos, "titulo", "proyecto.yml", informe, obligatorio=True)
            for clave in ("autor", "idioma", "creado"):
                _campo_texto(datos, clave, "proyecto.yml", informe)
            if "guardar_prompts" in datos and not isinstance(datos["guardar_prompts"], bool):
                informe.error("proyecto.yml: 'guardar_prompts' debe ser booleano")
            for clave, permitidos in (
                ("retencion", RETENCIONES),
                ("politica_estructura", POLITICAS),
                ("politica_metadatos", POLITICAS),
                ("politica_notas", POLITICAS),
                ("politica_prosa", POLITICAS),
                ("politica_exportar", POLITICAS),
            ):
                if clave in datos and datos[clave] not in permitidos:
                    informe.error(f"proyecto.yml: '{clave}' no válido: {datos[clave]!r}")
            if datos.get("politica_prosa") == "auto":
                informe.error("proyecto.yml: 'politica_prosa' no puede ser 'auto'")
            for clave in datos:
                if clave not in CAMPOS_PROYECTO:
                    informe.aviso(f"proyecto.yml: campo desconocido '{clave}' (se conserva)")
        except ErrorEsquema as error:
            informe.error(str(error))

    ids: dict[str, str] = {}
    ids_escenas: set[str] = set()
    tipos_nota: dict[str, str] = {}
    ordenes: dict[Path, dict[int, str]] = {}
    escenas: list[tuple[Path, dict[str, Any]]] = []
    notas: list[tuple[Path, dict[str, Any]]] = []

    for ruta in sorted(raiz.rglob("*.md")):
        if ".escritor" in ruta.parts:
            continue
        relativa = ruta.relative_to(raiz)
        if ruta == proyecto_yml:
            continue
        if relativa.parts[0] == "manuscrito":
            if len(relativa.parts) != 4:
                informe.error(f"{relativa}: una escena debe estar en manuscrito/libro/capitulo/")
                continue
            escenas.append((ruta, _leer_fixture(ruta, relativa, informe)))
        elif relativa.parts[0] == "notas":
            if len(relativa.parts) != 3 or relativa.parts[1] not in TIPOS_NOTA:
                informe.error(f"{relativa}: nota fuera de notas/<tipo>/")
                continue
            notas.append((ruta, _leer_fixture(ruta, relativa, informe)))
        else:
            informe.aviso(f"{relativa}: Markdown fuera de manuscrito/ y notas/")

    referencias: list[tuple[Path, str, str]] = []

    for ruta, datos in escenas:
        relativa = ruta.relative_to(raiz)
        if not datos:
            continue
        identificador = _validar_id(datos, relativa, ids, informe)
        if identificador is not None:
            ids_escenas.add(identificador)
        _campo_texto(datos, "titulo", relativa, informe, obligatorio=True)
        _validar_orden(datos, ruta.parent, relativa, ordenes, informe)
        if "estado" in datos and datos["estado"] not in ESTADOS_VALIDOS:
            informe.aviso(f"{relativa}: estado desconocido {datos['estado']!r}")
        if "palabras" in datos:
            informe.aviso(f"{relativa}: 'palabras' no debe escribirse a mano")
        if "fecha_narrativa" in datos:
            _campo_texto(datos, "fecha_narrativa", relativa, informe)
        _validar_etiquetas(datos, relativa, informe)
        for clave in datos:
            if clave not in CAMPOS_ESCENA:
                informe.aviso(f"{relativa}: campo desconocido '{clave}' (se conserva)")
        if "pov" in datos:
            if not isinstance(datos["pov"], str):
                informe.error(f"{relativa}: 'pov' debe ser un ULID (texto)")
            else:
                referencias.append((relativa, "pov", datos["pov"]))
        for campo in ("tramas", "personajes"):
            if campo in datos:
                if not isinstance(datos[campo], list):
                    informe.error(f"{relativa}: '{campo}' debe ser lista")
                    continue
                for valor in datos[campo]:
                    if not isinstance(valor, str):
                        informe.error(
                            f"{relativa}: '{campo}' contiene un valor no textual: {valor!r}"
                        )
                        continue
                    referencias.append((relativa, campo, valor))
        if identificador is not None and identificador == id_proyecto:
            informe.aviso(f"{relativa}: el id de la escena coincide con el del proyecto")

    for ruta, datos in notas:
        relativa = ruta.relative_to(raiz)
        if not datos:
            continue
        identificador = _validar_id(datos, relativa, ids, informe)
        tipo_carpeta = relativa.parts[1]
        if datos.get("tipo") != tipo_carpeta:
            informe.error(
                f"{relativa}: 'tipo' {datos.get('tipo')!r} no coincide con la carpeta "
                f"{tipo_carpeta!r}"
            )
        if identificador:
            tipos_nota[identificador] = tipo_carpeta
        _campo_texto(datos, "titulo", relativa, informe, obligatorio=True)
        if "fija" in datos and not isinstance(datos["fija"], bool):
            informe.error(f"{relativa}: 'fija' debe ser booleano")
        _validar_etiquetas(datos, relativa, informe)
        for clave in datos:
            if clave not in CAMPOS_NOTA:
                informe.aviso(f"{relativa}: campo desconocido '{clave}' (se conserva)")
        if "refs" in datos:
            if not isinstance(datos["refs"], list):
                informe.error(f"{relativa}: 'refs' debe ser lista")
            else:
                for valor in datos["refs"]:
                    if not isinstance(valor, str):
                        informe.error(
                            f"{relativa}: 'refs' contiene un valor no textual: {valor!r}"
                        )
                        continue
                    referencias.append((relativa, "refs", valor))
        if identificador is not None and identificador == id_proyecto:
            informe.aviso(f"{relativa}: el id de la nota coincide con el del proyecto")

    tipos_destino = {identificador: "escena" for identificador in ids_escenas}
    tipos_destino.update(tipos_nota)
    if id_proyecto is not None:
        tipos_destino[id_proyecto] = "proyecto"

    for relativa, campo, valor in referencias:
        if not es_ulid(valor):
            informe.error(f"{relativa}: referencia no ULID en '{campo}': {valor!r}")
            continue
        tipo_destino = tipos_destino.get(valor)
        if tipo_destino is None:
            informe.error(f"{relativa}: '{campo}' apunta a un id inexistente: {valor}")
            continue
        if campo in ("pov", "personajes") and tipo_destino != "personajes":
            informe.error(
                f"{relativa}: '{campo}' apunta a un id que no es personaje: "
                f"{valor} ({tipo_destino})"
            )
        if campo == "tramas" and tipo_destino != "tramas":
            informe.error(
                f"{relativa}: 'tramas' apunta a un id que no es trama: "
                f"{valor} ({tipo_destino})"
            )

    if not escenas:
        informe.error("el proyecto no tiene escenas")

    informe.resumen = {"escenas": len(escenas), "notas": len(notas), "ids": len(ids)}
    if not imprimir:
        return informe

    print(f"Fixture: {raiz}")
    print(f"  escenas: {len(escenas)} · notas: {len(notas)} · ids: {len(ids)}")
    for aviso in informe.avisos:
        print(f"  aviso: {aviso}")
    for error in informe.errores:
        print(f"  ERROR: {error}")
    print(
        "  resultado: "
        + ("OK" if not informe.errores else f"{len(informe.errores)} error(es)")
        + (f", {len(informe.avisos)} aviso(s)" if informe.avisos else "")
    )
    return informe


def leer_documento_plano(ruta: Path) -> tuple[dict[str, Any], str]:
    texto = _decodificar(ruta.read_bytes(), ruta)
    texto = texto.replace("\r\n", "\n").replace("\r", "\n")
    datos: dict[str, Any] = {}
    for numero, linea in enumerate(texto.splitlines(), start=1):
        if not linea.strip():
            continue
        if linea.startswith((" ", "\t")):
            raise ErrorEsquema(f"{ruta}:{numero}: indentación fuera del subconjunto YAML v1")
        if linea.lstrip().startswith("#"):
            raise ErrorEsquema(f"{ruta}:{numero}: comentarios no admitidos")
        clave, separador, valor = linea.partition(":")
        if not separador:
            raise ErrorEsquema(f"{ruta}:{numero}: falta ':' tras la clave")
        clave = clave.strip()
        if not RE_CLAVE.match(clave):
            raise ErrorEsquema(f"{ruta}:{numero}: clave no válida: {clave!r}")
        if clave in datos:
            raise ErrorEsquema(f"{ruta}:{numero}: clave duplicada: {clave}")
        datos[clave] = parsear_escalar(valor, f"{ruta}:{numero}")
    return datos, ""


def _revisar_bytes(crudo: bytes, etiqueta: str, informe: Informe) -> None:
    if crudo.startswith(b"\xef\xbb\xbf"):
        informe.aviso(f"{etiqueta}: BOM UTF-8 (se tolera al leer, no al escribir)")
    if b"\r\n" in crudo:
        informe.aviso(f"{etiqueta}: finales de línea CRLF")
    if b"\r" in crudo.replace(b"\r\n", b""):
        informe.error(f"{etiqueta}: retorno de carro aislado; se exige LF")
    for numero, linea in enumerate(crudo.split(b"\n"), start=1):
        if linea.rstrip(b"\r").endswith((b" ", b"\t")):
            informe.aviso(f"{etiqueta}:{numero}: espacios al final de línea")


def _leer_fixture(
    ruta: Path, relativa: Path, informe: Informe
) -> dict[str, Any]:
    crudo = ruta.read_bytes()
    _revisar_bytes(crudo, str(relativa), informe)
    try:
        return leer_documento(ruta, str(relativa))[0]
    except ErrorEsquema as error:
        informe.error(str(error))
        return {}


def _validar_id(
    datos: dict[str, Any], relativa: Path, ids: dict[str, str], informe: Informe
) -> str | None:
    if "id" not in datos:
        informe.error(f"{relativa}: falta 'id' (iría a cuarentena)")
        return None
    identificador = datos["id"]
    if not isinstance(identificador, str) or not es_ulid(identificador):
        informe.error(f"{relativa}: 'id' no es un ULID válido: {identificador!r}")
        return None
    if identificador in ids:
        informe.error(
            f"{relativa}: 'id' duplicado con {ids[identificador]} ({identificador})"
        )
        return None
    ids[identificador] = str(relativa)
    return identificador


def _validar_etiquetas(datos: dict[str, Any], relativa: Path, informe: Informe) -> None:
    if "etiquetas" not in datos:
        return
    etiquetas = datos["etiquetas"]
    if not isinstance(etiquetas, list):
        informe.error(f"{relativa}: 'etiquetas' debe ser lista")
        return
    for etiqueta in etiquetas:
        if not isinstance(etiqueta, str) or not etiqueta.strip():
            informe.error(f"{relativa}: etiqueta no textual o vacía: {etiqueta!r}")


def _validar_orden(
    datos: dict[str, Any],
    carpeta: Path,
    relativa: Path,
    ordenes: dict[Path, dict[int, str]],
    informe: Informe,
) -> None:
    if "orden" not in datos:
        informe.error(f"{relativa}: falta 'orden'")
        return
    valor = datos["orden"]
    if not isinstance(valor, int) or isinstance(valor, bool) or not 1 <= valor <= 99999:
        informe.error(f"{relativa}: 'orden' fuera de rango: {valor!r}")
        return
    del_capitulo = ordenes.setdefault(carpeta, {})
    if valor in del_capitulo:
        informe.error(
            f"{relativa}: 'orden' {valor} repetido en la carpeta "
            f"(ya usado por {del_capitulo[valor]})"
        )
        return
    del_capitulo[valor] = str(relativa)


# --------------------------------------------------------------------------- #
# Autopruebas y coherencia de la documentación
# --------------------------------------------------------------------------- #

RE_BLOQUE = re.compile(r"```(yaml|markdown|json|text)\n(.*?)```", re.DOTALL)
RE_ULID_CANDIDATO = re.compile(r"\b[0-9][0-9A-HJKMNP-TV-Z]{19,31}\b")
RE_ID_ABREVIADO = re.compile(r"\b[0-9A-HJKMNP-TV-Z]{6,}…")
RE_HASH = re.compile(r"^blake3:[0-9a-f]{64}$")


def _revisar_id(valor: Any, etiqueta: str, problemas: list[str]) -> None:
    if not isinstance(valor, str) or not es_ulid(valor):
        problemas.append(f"{etiqueta}: no es un ULID de 26 caracteres: {valor!r}")


def _revisar_ejemplo_frontmatter(datos: dict[str, Any], etiqueta: str, problemas: list[str]) -> None:
    for clave in ("id", "pov"):
        if clave in datos:
            _revisar_id(datos[clave], f"{etiqueta}:{clave}", problemas)
    for clave in ("tramas", "personajes", "refs"):
        if clave not in datos:
            continue
        if not isinstance(datos[clave], list):
            problemas.append(f"{etiqueta}:{clave}: debe ser lista")
            continue
        for valor in datos[clave]:
            _revisar_id(valor, f"{etiqueta}:{clave}", problemas)
    if "titulo" in datos and not isinstance(datos["titulo"], str):
        problemas.append(f"{etiqueta}:titulo: debe ser texto")
    if "orden" in datos and (not isinstance(datos["orden"], int) or isinstance(datos["orden"], bool)):
        problemas.append(f"{etiqueta}:orden: debe ser entero")
    if "fija" in datos and not isinstance(datos["fija"], bool):
        problemas.append(f"{etiqueta}:fija: debe ser booleano")


def _revisar_ejemplo_json(datos: Any, etiqueta: str, problemas: list[str]) -> None:
    if not isinstance(datos, dict):
        problemas.append(f"{etiqueta}: el bloque JSON no es un objeto")
        return
    if "op_id" in datos:
        _revisar_id(datos["op_id"], f"{etiqueta}:op_id", problemas)
    for clave in ("base_rev", "target_rev"):
        if clave in datos and (not isinstance(datos[clave], int) or isinstance(datos[clave], bool)):
            problemas.append(f"{etiqueta}:{clave}: debe ser entero")
    efectos = datos.get("efectos", [])
    if not isinstance(efectos, list):
        problemas.append(f"{etiqueta}:efectos: debe ser lista")
        return
    for efecto in efectos:
        if not isinstance(efecto, dict):
            problemas.append(f"{etiqueta}:efectos: cada efecto debe ser objeto")
            continue
        for clave in ("hash_nuevo", "hash_anterior"):
            if clave in efecto and efecto[clave] is not None:
                if not isinstance(efecto[clave], str) or not RE_HASH.match(efecto[clave]):
                    problemas.append(
                        f"{etiqueta}:{clave}: se espera blake3:<64 hex en minúsculas>"
                    )


def verificar_docs(ruta: Path) -> int:
    texto = ruta.read_text(encoding="utf-8")
    problemas: list[str] = []
    for numero, linea in enumerate(texto.splitlines(), start=1):
        for candidato in RE_ULID_CANDIDATO.findall(linea):
            if candidato == ALFABETO_ULID:
                continue
            if not es_ulid(candidato):
                problemas.append(f"{ruta}:{numero}: ULID no válido: {candidato}")
        if RE_ID_ABREVIADO.search(linea):
            problemas.append(f"{ruta}:{numero}: ID abreviado con elipsis; usa el ULID completo")
    for tipo, contenido in RE_BLOQUE.findall(texto):
        if tipo == "yaml":
            for numero, linea in enumerate(contenido.splitlines(), start=1):
                if not linea.strip():
                    continue
                clave, separador, valor = linea.partition(":")
                if not separador:
                    problemas.append(f"{ruta}: bloque yaml, línea {numero}: falta ':'")
                    continue
                try:
                    escalar = parsear_escalar(valor, f"{ruta}: bloque yaml, línea {numero}")
                except ErrorEsquema as error:
                    problemas.append(str(error))
                    continue
                if clave.strip() == "id":
                    _revisar_id(escalar, f"{ruta}: bloque yaml, línea {numero}", problemas)
        elif tipo == "markdown":
            try:
                datos, _ = parsear_documento(contenido, f"{ruta}: bloque markdown")
            except ErrorEsquema as error:
                problemas.append(str(error))
                continue
            _revisar_ejemplo_frontmatter(datos, f"{ruta}: bloque markdown", problemas)
        elif tipo == "json":
            try:
                datos = json.loads(contenido)
            except json.JSONDecodeError as error:
                problemas.append(f"{ruta}: bloque json inválido: {error}")
                continue
            _revisar_ejemplo_json(datos, f"{ruta}: bloque json", problemas)
    for problema in problemas:
        print(f"  ERROR: {problema}")
    print(
        f"Documentación: {ruta} · "
        + ("OK" if not problemas else f"{len(problemas)} problema(s)")
    )
    return 1 if problemas else 0


def autoprueba() -> int:
    fallos = 0
    with tempfile.TemporaryDirectory() as temporal:
        base = Path(temporal) / "base"
        with contextlib.redirect_stdout(io.StringIO()):
            crear_proyecto(base, 3, 1, None, True)
        proyecto = base / "proyecto.yml"
        id_proyecto = leer_documento_plano(proyecto)[0]["id"]
        escenas = sorted((base / "manuscrito").rglob("*.md"))
        primera, segunda = escenas[0], escenas[1]
        orden_primera = re.search(r"(?m)^orden: (\d+)$", primera.read_text(encoding="utf-8"))
        trama = sorted((base / "notas" / "tramas").glob("*.md"))[0]
        id_trama = leer_documento(trama)[0]["id"]
        id_escena = leer_documento(primera)[0]["id"]

        if orden_primera is None:
            print("FALLO [autoprueba] no se encontró 'orden' en la primera escena")
            return 1

        casos: list[tuple[str, Path, Any, str]] = [
            (
                "pov apunta al proyecto",
                primera,
                lambda t: re.sub(r"(?m)^pov: .*$", f"pov: {id_proyecto}", t, count=1),
                "no es personaje",
            ),
            (
                "pov apunta a una trama",
                primera,
                lambda t: re.sub(r"(?m)^pov: .*$", f"pov: {id_trama}", t, count=1),
                "no es personaje",
            ),
            (
                "titulo booleano",
                primera,
                lambda t: re.sub(r"(?m)^titulo: .*$", "titulo: true", t, count=1),
                "debe ser texto",
            ),
            (
                "titulo vacío",
                primera,
                lambda t: re.sub(r"(?m)^titulo: .*$", 'titulo: ""', t, count=1),
                "está vacío",
            ),
            (
                "guardar_prompts de texto",
                proyecto,
                lambda t: t.replace("guardar_prompts: false", 'guardar_prompts: "false"'),
                "debe ser booleano",
            ),
            (
                "comillas interiores sin escapar",
                primera,
                lambda t: re.sub(r'(?m)^titulo: "(.*)"$', r'titulo: "El "\1""', t, count=1),
                "comilla interior",
            ),
            (
                "comillas simples",
                primera,
                lambda t: re.sub(r'(?m)^titulo: "(.*)"$', r"titulo: '\1'", t, count=1),
                "comillas simples",
            ),
            (
                "retorno de carro aislado",
                primera,
                lambda t: t.replace("\n", "\r", 1),
                "retorno de carro aislado",
            ),
            (
                "ULID de 24 caracteres",
                primera,
                lambda t: re.sub(r"(?m)^id: .*$", "id: 01JB7K2M9Q4T8V6X3R5Z1N0C", t, count=1),
                "no es un ULID",
            ),
            (
                "referencia inexistente",
                trama,
                lambda t: re.sub(
                    r"(?m)^refs: .*$",
                    "refs: [01KDVDNJS8TEP0Y8EXPSPATJH6]",
                    t,
                    count=1,
                ),
                "id inexistente",
            ),
            (
                "orden duplicado",
                segunda,
                lambda t: re.sub(
                    r"(?m)^orden: \d+$", f"orden: {orden_primera.group(1)}", t, count=1
                ),
                "repetido",
            ),
            (
                "texto plano con #",
                primera,
                lambda t: re.sub(
                    r'(?m)^titulo: "(.*)"$', "titulo: Escena # 1", t, count=1
                ),
                "usa comillas dobles",
            ),
            (
                "comentario de línea completa",
                primera,
                lambda t: t.replace("---\n", "---\n# comentario\n", 1),
                "comentarios no admitidos",
            ),
        ]

        for nombre, archivo, mutar, esperado in casos:
            with tempfile.TemporaryDirectory() as copia_temporal:
                copia = Path(copia_temporal) / "proyecto"
                shutil.copytree(base, copia)
                objetivo = copia / archivo.relative_to(base)
                original = objetivo.read_text(encoding="utf-8")
                mutado = mutar(original)
                if mutado == original:
                    print(f"FALLO [autoprueba] {nombre}: la mutación no cambió nada")
                    fallos += 1
                    continue
                objetivo.write_text(mutado, encoding="utf-8", newline="")
                with contextlib.redirect_stdout(io.StringIO()):
                    informe = verificar(copia)
                if not any(esperado in error for error in informe.errores):
                    print(
                        f"FALLO [autoprueba] {nombre}: se esperaba un error con "
                        f"{esperado!r}; obtenidos: {informe.errores}"
                    )
                    fallos += 1

        casos_positivos: list[tuple[str, Path, Any]] = [
            (
                "nota referencia una escena",
                trama,
                lambda t: re.sub(r"(?m)^refs: .*$", f"refs: [{id_escena}]", t, count=1),
            ),
            (
                "línea vacía dentro del frontmatter",
                primera,
                lambda t: t.replace("\ntitulo:", "\n\ntitulo:", 1),
            ),
            (
                "título con # entre comillas",
                primera,
                lambda t: re.sub(
                    r'(?m)^titulo: "(.*)"$', 'titulo: "Escena # 1"', t, count=1
                ),
            ),
        ]

        for nombre, archivo, mutar in casos_positivos:
            with tempfile.TemporaryDirectory() as copia_temporal:
                copia = Path(copia_temporal) / "proyecto"
                shutil.copytree(base, copia)
                objetivo = copia / archivo.relative_to(base)
                original = objetivo.read_text(encoding="utf-8")
                mutado = mutar(original)
                if mutado == original:
                    print(f"FALLO [autoprueba] {nombre}: la mutación no cambió nada")
                    fallos += 1
                    continue
                objetivo.write_text(mutado, encoding="utf-8", newline="")
                with contextlib.redirect_stdout(io.StringIO()):
                    informe = verificar(copia)
                if informe.errores:
                    print(
                        f"FALLO [autoprueba] {nombre}: no debía haber errores; "
                        f"obtenidos: {informe.errores}"
                    )
                    fallos += 1

        with contextlib.redirect_stdout(io.StringIO()):
            limpio = verificar(base)
        if limpio.errores:
            print(f"FALLO [autoprueba] el fixture base no es válido: {limpio.errores}")
            fallos += 1

        with tempfile.TemporaryDirectory() as copia_temporal:
            copia = Path(copia_temporal) / "proyecto"
            shutil.copytree(base, copia)
            objetivo = copia / primera.relative_to(base)
            original = objetivo.read_text(encoding="utf-8")
            objetivo.write_bytes(original.replace("\n", "\r\n").encode("utf-8"))
            with contextlib.redirect_stdout(io.StringIO()):
                crlf = verificar(copia)
            if crlf.errores or not any("CRLF" in aviso for aviso in crlf.avisos):
                print("FALLO [autoprueba] CRLF debe ser aviso, no error")
                fallos += 1

        with tempfile.TemporaryDirectory() as copia_temporal:
            copia = Path(copia_temporal) / "proyecto"
            shutil.copytree(base, copia)
            objetivo = copia / primera.relative_to(base)
            original = objetivo.read_bytes()
            objetivo.write_bytes(b"\xef\xbb\xbf" + original)
            with contextlib.redirect_stdout(io.StringIO()):
                bom = verificar(copia)
            if bom.errores or not any("BOM" in aviso for aviso in bom.avisos):
                print("FALLO [autoprueba] el BOM debe ser aviso, no error")
                fallos += 1

    if fallos:
        print(f"Autopruebas: {fallos} fallo(s)")
    else:
        print("Autopruebas: OK (casos negativos, positivos y de tolerancia)")
    return 1 if fallos else 0


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser(
        description="Genera y verifica fixtures de writer.",
        epilog=__doc__.split("Uso:")[0].strip(),
    )
    modo = parser.add_mutually_exclusive_group(required=True)
    modo.add_argument("--destino", type=Path, help="carpeta a generar")
    modo.add_argument("--verificar", type=Path, metavar="DIR", help="carpeta a verificar")
    modo.add_argument(
        "--autoprueba",
        action="store_true",
        help="ejecuta los casos negativos y de tolerancia del verificador",
    )
    modo.add_argument(
        "--verificar-docs",
        type=Path,
        metavar="DOC",
        help="comprueba la coherencia de los ejemplos del documento",
    )
    parser.add_argument("--escenas", type=int, default=3, help="número de escenas (por defecto 3)")
    parser.add_argument("--capitulos", type=int, default=None, help="número de capítulos")
    parser.add_argument("--semilla", type=int, default=1, help="semilla determinista (por defecto 1)")
    parser.add_argument("--forzar", action="store_true", help="sobrescribe el destino")
    parser.add_argument(
        "--json",
        action="store_true",
        help="con --verificar, emite el informe en JSON y no imprime el resumen humano",
    )

    opciones = parser.parse_args(argv)
    if opciones.autoprueba:
        return autoprueba()
    if opciones.verificar_docs is not None:
        if not opciones.verificar_docs.is_file():
            print(f"no existe el documento: {opciones.verificar_docs}", file=sys.stderr)
            return 2
        return verificar_docs(opciones.verificar_docs)
    if opciones.destino is not None:
        if opciones.escenas < 1:
            parser.error("--escenas debe ser >= 1")
        try:
            crear_proyecto(
                opciones.destino,
                opciones.escenas,
                opciones.semilla,
                opciones.capitulos,
                opciones.forzar,
            )
        except OSError as error:
            print(f"error de sistema de archivos: {error}", file=sys.stderr)
            return 1
        return 0

    raiz = opciones.verificar
    if not raiz.is_dir():
        print(f"no existe la carpeta: {raiz}", file=sys.stderr)
        return 2
    informe = verificar(raiz, imprimir=not opciones.json)
    if opciones.json:
        print(
            json.dumps(
                {
                    "fixture": str(raiz),
                    "resumen": informe.resumen,
                    "errores": informe.errores,
                    "avisos": informe.avisos,
                },
                ensure_ascii=False,
                indent=2,
            )
        )
    return 1 if informe.errores else 0


if __name__ == "__main__":
    sys.exit(main())
