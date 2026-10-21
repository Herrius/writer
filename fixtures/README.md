# Fixtures

Proyectos de escritura de prueba, generados con `scripts/generar-fixtures.py`.

| Carpeta | Versionada | Contenido |
|---|---|---|
| `proyecto-pequeno/` | sí | 1 libro, 1 capítulo, 3 escenas y notas de todos los tipos. Semilla 1, IDs deterministas: sirve para golden tests. |
| `proyecto-1000/` | no | 1 libro, 10 capítulos, 1.000 escenas. Se genera bajo demanda para medir apertura y búsqueda. |
| `proyecto-roto/` | sí | 12 defectos deliberados del esquema y 4 avisos. Es la prueba de regresión negativa de `verify`; no regenerar ni "arreglar". |

Regenerar el pequeño (debe salir idéntico, byte a byte):

```bash
python scripts/generar-fixtures.py --destino fixtures/proyecto-pequeno --escenas 3 --semilla 1 --forzar
```

Comprobar cualquiera de los dos:

```bash
python scripts/generar-fixtures.py --verificar fixtures/proyecto-pequeno
cargo run --bin writer -- verify fixtures/proyecto-pequeno
```

El roto se comprueba contra su contrato de hallazgos:

```bash
python scripts/generar-fixtures.py --verificar fixtures/proyecto-roto --json
python scripts/ci.py producto
```

Los fixtures **no** incluyen `.escritor/`: son proyectos limpios tal como los deja
un usuario antes de abrirlos con el motor. El índice, los blobs y el log se crean
al usarlos.
