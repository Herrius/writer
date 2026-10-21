//! Fechas RFC 3339 UTC con milisegundos, sin dependencias externas.
//!
//! El esquema exige `YYYY-MM-DDTHH:MM:SS.mmmZ`. Solo se formatean y comparan
//! fechas; no hay zonas horarias ni calendarios locales.

use crate::error::{Error, Result};

/// Devuelve el instante actual en milisegundos desde la época Unix.
pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duracion) => i64::try_from(duracion.as_millis()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

/// Formatea milisegundos desde la época como RFC 3339 UTC.
pub fn format_rfc3339(ms: i64) -> String {
    let (dias, resto) = div_mod(ms, 86_400_000);
    let (anio, mes, dia) = civil_from_days(dias);
    let hora = resto / 3_600_000;
    let minuto = (resto % 3_600_000) / 60_000;
    let segundo = (resto % 60_000) / 1_000;
    let milisegundo = resto % 1_000;
    format!("{anio:04}-{mes:02}-{dia:02}T{hora:02}:{minuto:02}:{segundo:02}.{milisegundo:03}Z")
}

/// Formatea milisegundos como marca compacta para nombres de archivo.
pub fn timestamp_slug(ms: i64) -> String {
    let texto = format_rfc3339(ms);
    texto.replace([':', '-', '.'], "")
}

/// Instante actual como RFC 3339 UTC.
pub fn ahora() -> String {
    format_rfc3339(now_ms())
}

/// Convierte una fecha RFC 3339 UTC con `Z` a milisegundos.
pub fn parse_rfc3339(texto: &str) -> Result<i64> {
    let bytes = texto.as_bytes();
    if bytes.len() < 20 || !texto.ends_with('Z') {
        return Err(Error::Schema(format!("fecha RFC 3339 no válida: {texto}")));
    }
    let base = &texto[..texto.len() - 1];
    let (fecha, hora) = base
        .split_once('T')
        .ok_or_else(|| Error::Schema(format!("fecha RFC 3339 no válida: {texto}")))?;
    let mut partes_fecha = fecha.split('-');
    let anio: i64 = parse_num(partes_fecha.next(), texto)?;
    let mes: u32 = parse_num(partes_fecha.next(), texto)?;
    let dia: u32 = parse_num(partes_fecha.next(), texto)?;
    let mut partes_hora = hora.split(':');
    let hora_u: u32 = parse_num(partes_hora.next(), texto)?;
    let minuto: u32 = parse_num(partes_hora.next(), texto)?;
    let resto = partes_hora
        .next()
        .ok_or_else(|| Error::Schema(format!("fecha RFC 3339 no válida: {texto}")))?;
    let (segundo, milisegundo): (u32, u32) = match resto.split_once('.') {
        Some((segundo, fraccion)) => {
            if fraccion.len() != 3 {
                return Err(Error::Schema(format!(
                    "se exigen milisegundos exactos: {texto}"
                )));
            }
            (
                parse_num(Some(segundo), texto)?,
                parse_num(Some(fraccion), texto)?,
            )
        }
        None => (parse_num(Some(resto), texto)?, 0),
    };
    if !(1..=12).contains(&mes)
        || !(1..=31).contains(&dia)
        || hora_u > 23
        || minuto > 59
        || segundo > 60
    {
        return Err(Error::Schema(format!(
            "fecha RFC 3339 fuera de rango: {texto}"
        )));
    }
    let dias = days_from_civil(anio, mes, dia);
    Ok(dias * 86_400_000
        + i64::from(hora_u) * 3_600_000
        + i64::from(minuto) * 60_000
        + i64::from(segundo) * 1_000
        + i64::from(milisegundo))
}

fn parse_num<T: std::str::FromStr>(valor: Option<&str>, original: &str) -> Result<T> {
    valor
        .ok_or_else(|| Error::Schema(format!("fecha RFC 3339 no válida: {original}")))?
        .parse()
        .map_err(|_| Error::Schema(format!("fecha RFC 3339 no válida: {original}")))
}

fn div_mod(valor: i64, divisor: i64) -> (i64, i64) {
    let cociente = valor.div_euclid(divisor);
    let resto = valor.rem_euclid(divisor);
    (cociente, resto)
}

fn days_from_civil(anio: i64, mes: u32, dia: u32) -> i64 {
    let anio = if mes <= 2 { anio - 1 } else { anio };
    let era = if anio >= 0 { anio } else { anio - 399 } / 400;
    let yoe = anio - era * 400;
    let mes = i64::from(mes);
    let dia = i64::from(dia);
    let doy = (153 * (if mes > 2 { mes - 3 } else { mes + 9 }) + 2) / 5 + dia - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(dias: i64) -> (i64, u32, u32) {
    let z = dias + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ida_y_vuelta_rfc3339() {
        for ms in [0, 1_767_225_600_000, 1_767_225_600_123, 1_800_000_000_000] {
            let texto = format_rfc3339(ms);
            assert_eq!(parse_rfc3339(&texto).expect("parsear"), ms);
        }
    }

    #[test]
    fn fecha_conocida() {
        assert_eq!(format_rfc3339(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(
            format_rfc3339(1_767_225_600_000),
            "2026-01-01T00:00:00.000Z"
        );
    }

    #[test]
    fn slug_compacto() {
        assert_eq!(timestamp_slug(1_767_225_600_000), "20260101T000000000Z");
    }
}
