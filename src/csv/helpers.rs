use crate::domain::{error::AppError, ntg_column::NtgColumnType};

pub fn normalize_original_header(raw: &str) -> String {
    // trim strip boom and normalize case
    raw.trim().trim_start_matches('\u{feff}').to_lowercase()
}

pub fn normalize_ntg_header(raw: &str, index: usize) -> String {
    let ntg_header = raw.trim().to_lowercase();
    if ntg_header.split("-").nth(1).is_some_and(|n| n.eq("999")) {
        format!("ntg-999-{index}")
    } else {
        ntg_header
    }
}

pub fn parse_ctype_clength(raw: &str) -> Result<(NtgColumnType, Option<i32>), AppError> {
    let raw = raw.trim();
    let (ctype_str, clength_str) = match raw.find(|c: char| c.is_ascii_digit()) {
        Some(i) => raw.split_at(i),
        None => (raw, ""),
    };

    let ctype = NtgColumnType::from_str(ctype_str).ok_or_else(|| AppError::CsvMalformed {
        detail: format!("unknown content type {ctype_str:?}"),
    })?;

    let clength = if clength_str.is_empty() {
        None
    } else {
        Some(
            clength_str
                .trim()
                .parse::<i32>()
                .map_err(|_| AppError::CsvMalformed {
                    detail: format!("invalid content length {clength_str:?}"),
                })?,
        )
    };

    Ok((ctype, clength))
}
