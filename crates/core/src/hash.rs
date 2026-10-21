//! Hashes BLAKE3 con el formato del esquema: `blake3:<hex minúsculas>`.

use crate::error::{Error, Result};

/// Calcula el hash de unos bytes y lo devuelve con prefijo.
pub fn hash(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

/// Calcula el hash sin prefijo (nombre de blob y checksum del log).
pub fn hash_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Comprueba que un texto tiene forma de hash del esquema.
pub fn es_hash(texto: &str) -> bool {
    match texto.strip_prefix("blake3:") {
        Some(hex) => {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        }
        None => false,
    }
}

/// Extrae el hexadecimal de un hash con prefijo.
pub fn hex(texto: &str) -> Result<&str> {
    texto
        .strip_prefix("blake3:")
        .filter(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
        .ok_or_else(|| Error::Format(format!("hash no válido: {texto}")))
}

/// Comprueba que unos bytes producen el hash indicado.
pub fn coincide(bytes: &[u8], esperado: &str) -> bool {
    hash(bytes).eq_ignore_ascii_case(esperado)
}

/// Calcula el hash de un archivo, o `None` si no existe.
pub fn hash_archivo(ruta: &std::path::Path) -> Result<Option<String>> {
    match std::fs::read(ruta) {
        Ok(bytes) => Ok(Some(hash(&bytes))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::Io(error).with_path(ruta)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formato_estable() {
        let valor = hash(b"hola\n");
        assert!(es_hash(&valor));
        assert_eq!(valor.len(), "blake3:".len() + 64);
        assert_eq!(hash(b"hola\n"), valor);
        assert_ne!(hash(b"hola\n"), hash(b"hola"));
    }

    #[test]
    fn rechaza_mayusculas_y_prefijo_incorrecto() {
        assert!(!es_hash("SHA256:abc"));
        assert!(!es_hash("blake3:ABC"));
        assert!(!es_hash("blake3:123"));
    }

    #[test]
    fn coincide_con_bytes() {
        let bytes = b"prueba";
        assert!(coincide(bytes, &hash(bytes)));
        assert!(!coincide(bytes, &hash(b"otra")));
    }
}
