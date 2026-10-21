//! ULID: 26 caracteres Crockford base32, ordenables por tiempo.
//!
//! El esquema exige mayúsculas, alfabeto Crockford y primera letra `0`–`7`.
//! No se aceptan ULID de 24 caracteres ni variantes con `I`, `L`, `O` o `U`.

use crate::error::{Error, Result};
use crate::time;

/// Alfabeto Crockford base32 usado por ULID.
pub const ALFABETO: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Comprueba si un texto es un ULID válido según el esquema v1.
pub fn es_valido(texto: &str) -> bool {
    let bytes = texto.as_bytes();
    if bytes.len() != 26 || !(b'0'..=b'7').contains(&bytes[0]) {
        return false;
    }
    bytes[1..].iter().all(|byte| indice(*byte).is_some())
}

/// Genera un ULID nuevo con el reloj del sistema y entropía del SO.
pub fn nuevo() -> Result<String> {
    let mut entropia = [0_u8; 10];
    getrandom::fill(&mut entropia)
        .map_err(|error| Error::Other(format!("no se pudo obtener entropía: {error}")))?;
    Ok(desde_partes(time::now_ms(), entropia))
}

/// Construye un ULID a partir de milisegundos y 80 bits de entropía.
///
/// Es público para pruebas deterministas; la CLI usa [`nuevo`].
pub fn desde_partes(momento_ms: i64, entropia: [u8; 10]) -> String {
    let momento = u128::try_from(momento_ms.max(0)).unwrap_or(0) & ((1_u128 << 48) - 1);
    let mut bytes = [0_u8; 16];
    bytes[6..].copy_from_slice(&entropia);
    let valor = (momento << 80) | u128::from_be_bytes(bytes);
    let mut salida = String::with_capacity(26);
    for posicion in (0..26).rev() {
        let indice = ((valor >> (5 * posicion)) & 31) as usize;
        salida.push(ALFABETO[indice] as char);
    }
    salida
}

/// Extrae los milisegundos codificados en un ULID válido.
pub fn momento_ms(texto: &str) -> Option<i64> {
    if !es_valido(texto) {
        return None;
    }
    let valor = decodificar(texto)?;
    i64::try_from(valor >> 80).ok()
}

fn decodificar(texto: &str) -> Option<u128> {
    let mut valor = 0_u128;
    for byte in texto.bytes() {
        valor = (valor << 5) | u128::from(indice(byte)?);
    }
    Some(valor)
}

fn indice(byte: u8) -> Option<u8> {
    match byte {
        b'0' => Some(0),
        b'1' => Some(1),
        b'2' => Some(2),
        b'3' => Some(3),
        b'4' => Some(4),
        b'5' => Some(5),
        b'6' => Some(6),
        b'7' => Some(7),
        b'8' => Some(8),
        b'9' => Some(9),
        b'A' => Some(10),
        b'B' => Some(11),
        b'C' => Some(12),
        b'D' => Some(13),
        b'E' => Some(14),
        b'F' => Some(15),
        b'G' => Some(16),
        b'H' => Some(17),
        b'J' => Some(18),
        b'K' => Some(19),
        b'M' => Some(20),
        b'N' => Some(21),
        b'P' => Some(22),
        b'Q' => Some(23),
        b'R' => Some(24),
        b'S' => Some(25),
        b'T' => Some(26),
        b'V' => Some(27),
        b'W' => Some(28),
        b'X' => Some(29),
        b'Y' => Some(30),
        b'Z' => Some(31),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acepta_y_rechaza() {
        let valido = desde_partes(1_767_225_600_000, [7; 10]);
        assert_eq!(valido.len(), 26);
        assert!(es_valido(&valido));
        assert!(es_valido("01KDVDNAZ84TFF59TDWH9EDD1R"));
        assert!(!es_valido("01KDVDNAZ84TFF59TDWH9EDD1"));
        assert!(!es_valido("81KDVDNAZ84TFF59TDWH9EDD1R"));
        assert!(!es_valido("01KDVDNAZ84TFF59TDWH9EDD1I"));
        assert!(!es_valido("01kdvdnaz84tff59tdwh9edd1r"));
    }

    #[test]
    fn momento_coincide() {
        let ms = 1_767_225_600_123;
        let ulid = desde_partes(ms, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(momento_ms(&ulid), Some(ms));
    }

    #[test]
    fn orden_por_tiempo() {
        let antes = desde_partes(1_000, [0; 10]);
        let despues = desde_partes(2_000, [0; 10]);
        assert!(antes < despues);
    }
}
