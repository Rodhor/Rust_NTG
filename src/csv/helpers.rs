use crate::domain::ntg_column::NtgColumnType;

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

pub fn parse_ctype_clength(raw: &str) -> Option<(NtgColumnType, Option<i32>)> {
    let split_index = raw.find(|c: char| c.is_ascii_digit()).unwrap_or(0);
    if split_index == 0 {
        return None;
    }
    let parts: (&str, &str) = raw.split_at(split_index);
    let ctype = NtgColumnType::from_str(parts.0)?;
    let clength = parts.1.parse().ok();
    Some((ctype, clength))
}
