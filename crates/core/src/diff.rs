//! Diferencias de líneas para control de cambios y comparación de versiones.
//!
//! El algoritmo es local y no modifica documentos. Primero recorta el prefijo y
//! el sufijo comunes; después calcula una subsecuencia común máxima sobre el
//! tramo central. Para documentos desproporcionados evita una matriz gigante y
//! degrada a un bloque de borrado + inserción, que sigue siendo correcto aunque
//! menos granular.

use serde::Serialize;

/// Tipo de una línea en una diferencia.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TipoLinea {
    /// Igual en ambos lados.
    Igual,
    /// Solo existe en la propuesta.
    Insertada,
    /// Solo existe en la base.
    Borrada,
}

impl TipoLinea {
    /// Etiqueta estable para DTOs y pruebas.
    pub fn como_str(self) -> &'static str {
        match self {
            Self::Igual => "igual",
            Self::Insertada => "insertada",
            Self::Borrada => "borrada",
        }
    }
}

/// Una línea de la diferencia.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LineaDiff {
    /// Tipo.
    pub tipo: TipoLinea,
    /// Número de línea en la base (1-based), si aplica.
    pub numero_base: Option<usize>,
    /// Número de línea en la propuesta (1-based), si aplica.
    pub numero_propuesta: Option<usize>,
    /// Texto de la línea, sin salto.
    pub texto: String,
}

/// Resultado de comparar dos textos.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Diferencias {
    /// Líneas en orden de recorrido.
    pub lineas: Vec<LineaDiff>,
    /// Líneas insertadas.
    pub insertadas: usize,
    /// Líneas borradas.
    pub borradas: usize,
}

impl Diferencias {
    /// `true` si ambos textos son iguales.
    pub fn vacia(&self) -> bool {
        self.insertadas == 0 && self.borradas == 0
    }
}

/// Compara dos textos por líneas.
pub fn comparar(base: &str, propuesta: &str) -> Diferencias {
    let base = dividir_lineas(base);
    let propuesta = dividir_lineas(propuesta);
    let mut resultado = Diferencias::default();

    let mut prefijo = 0;
    while prefijo < base.len() && prefijo < propuesta.len() && base[prefijo] == propuesta[prefijo] {
        prefijo += 1;
    }

    let mut sufijo = 0;
    while sufijo < base.len().saturating_sub(prefijo)
        && sufijo < propuesta.len().saturating_sub(prefijo)
        && base[base.len() - 1 - sufijo] == propuesta[propuesta.len() - 1 - sufijo]
    {
        sufijo += 1;
    }

    for (indice, texto) in base.iter().take(prefijo).enumerate() {
        resultado.lineas.push(LineaDiff {
            tipo: TipoLinea::Igual,
            numero_base: Some(indice + 1),
            numero_propuesta: Some(indice + 1),
            texto: texto.clone(),
        });
    }

    let base_media = &base[prefijo..base.len() - sufijo];
    let propuesta_media = &propuesta[prefijo..propuesta.len() - sufijo];
    let mut indice_base = prefijo;
    let mut indice_propuesta = prefijo;

    if base_media.len().saturating_mul(propuesta_media.len()) > 2_000_000 {
        for linea in base_media {
            resultado.lineas.push(LineaDiff {
                tipo: TipoLinea::Borrada,
                numero_base: Some(indice_base + 1),
                numero_propuesta: None,
                texto: linea.to_string(),
            });
            indice_base += 1;
        }
        for linea in propuesta_media {
            resultado.lineas.push(LineaDiff {
                tipo: TipoLinea::Insertada,
                numero_base: None,
                numero_propuesta: Some(indice_propuesta + 1),
                texto: linea.to_string(),
            });
            indice_propuesta += 1;
        }
    } else {
        for operacion in operaciones_lcs(base_media, propuesta_media) {
            match operacion {
                Operacion::Igual(linea) => {
                    resultado.lineas.push(LineaDiff {
                        tipo: TipoLinea::Igual,
                        numero_base: Some(indice_base + 1),
                        numero_propuesta: Some(indice_propuesta + 1),
                        texto: linea.to_string(),
                    });
                    indice_base += 1;
                    indice_propuesta += 1;
                }
                Operacion::Borrada(linea) => {
                    resultado.lineas.push(LineaDiff {
                        tipo: TipoLinea::Borrada,
                        numero_base: Some(indice_base + 1),
                        numero_propuesta: None,
                        texto: linea.to_string(),
                    });
                    indice_base += 1;
                }
                Operacion::Insertada(linea) => {
                    resultado.lineas.push(LineaDiff {
                        tipo: TipoLinea::Insertada,
                        numero_base: None,
                        numero_propuesta: Some(indice_propuesta + 1),
                        texto: linea.to_string(),
                    });
                    indice_propuesta += 1;
                }
            }
        }
    }

    for indice in 0..sufijo {
        let base_indice = base.len() - sufijo + indice;
        let propuesta_indice = propuesta.len() - sufijo + indice;
        resultado.lineas.push(LineaDiff {
            tipo: TipoLinea::Igual,
            numero_base: Some(base_indice + 1),
            numero_propuesta: Some(propuesta_indice + 1),
            texto: base[base_indice].to_string(),
        });
    }

    resultado.insertadas = resultado
        .lineas
        .iter()
        .filter(|linea| linea.tipo == TipoLinea::Insertada)
        .count();
    resultado.borradas = resultado
        .lineas
        .iter()
        .filter(|linea| linea.tipo == TipoLinea::Borrada)
        .count();
    resultado
}

enum Operacion<'a> {
    Igual(&'a str),
    Borrada(&'a str),
    Insertada(&'a str),
}

fn operaciones_lcs<'a>(base: &'a [String], propuesta: &'a [String]) -> Vec<Operacion<'a>> {
    let filas = base.len();
    let columnas = propuesta.len();
    let mut tabla = vec![vec![0usize; columnas + 1]; filas + 1];
    for fila in (0..filas).rev() {
        for columna in (0..columnas).rev() {
            tabla[fila][columna] = if base[fila] == propuesta[columna] {
                tabla[fila + 1][columna + 1] + 1
            } else {
                tabla[fila + 1][columna].max(tabla[fila][columna + 1])
            };
        }
    }

    let mut operaciones = Vec::new();
    let mut fila = 0;
    let mut columna = 0;
    while fila < filas && columna < columnas {
        if base[fila] == propuesta[columna] {
            operaciones.push(Operacion::Igual(&base[fila]));
            fila += 1;
            columna += 1;
        } else if tabla[fila + 1][columna] >= tabla[fila][columna + 1] {
            operaciones.push(Operacion::Borrada(&base[fila]));
            fila += 1;
        } else {
            operaciones.push(Operacion::Insertada(&propuesta[columna]));
            columna += 1;
        }
    }
    while fila < filas {
        operaciones.push(Operacion::Borrada(&base[fila]));
        fila += 1;
    }
    while columna < columnas {
        operaciones.push(Operacion::Insertada(&propuesta[columna]));
        columna += 1;
    }
    operaciones
}

fn dividir_lineas(texto: &str) -> Vec<String> {
    let normalizado = texto.replace("\r\n", "\n").replace('\r', "\n");
    let sin_final = normalizado.strip_suffix('\n').unwrap_or(&normalizado);
    if sin_final.is_empty() {
        Vec::new()
    } else {
        sin_final.split('\n').map(str::to_string).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textos_iguales_no_tienen_cambios() {
        let diff = comparar("uno\ndos\n", "uno\ndos");
        assert!(diff.vacia());
        assert_eq!(diff.lineas.len(), 2);
        assert!(diff
            .lineas
            .iter()
            .all(|linea| linea.tipo == TipoLinea::Igual));
    }

    #[test]
    fn detecta_insercion_y_borrado() {
        let diff = comparar("uno\ndos\ntres", "uno\ntres\ncuatro");
        assert_eq!(diff.borradas, 1);
        assert_eq!(diff.insertadas, 1);
        assert!(diff
            .lineas
            .iter()
            .any(|linea| linea.tipo == TipoLinea::Borrada && linea.texto == "dos"));
        assert!(diff
            .lineas
            .iter()
            .any(|linea| linea.tipo == TipoLinea::Insertada && linea.texto == "cuatro"));
    }

    #[test]
    fn numera_ambos_lados() {
        let diff = comparar("a\nb", "a\nx\nb");
        let insertada = diff
            .lineas
            .iter()
            .find(|linea| linea.tipo == TipoLinea::Insertada)
            .expect("inserción");
        assert_eq!(insertada.numero_base, None);
        assert_eq!(insertada.numero_propuesta, Some(2));
        let final_igual = diff.lineas.last().expect("última");
        assert_eq!(final_igual.numero_base, Some(2));
        assert_eq!(final_igual.numero_propuesta, Some(3));
    }

    #[test]
    fn maneja_vacios_y_finales_de_linea() {
        assert!(comparar("", "").vacia());
        assert_eq!(comparar("", "uno").insertadas, 1);
        assert_eq!(comparar("uno", "").borradas, 1);
        assert!(comparar("uno\r\ndos\r\n", "uno\ndos").vacia());
    }
}
