use csv::{StringRecord, StringRecordIter};

use crate::csv::helpers::{normalize_ntg_header, normalize_original_header, parse_ctype_clength};
use crate::domain::{error::AppError, ntg_column::NtgColumnType, ntg_metadata::NtgMetadata};

pub fn load_csv(path: &str) -> Result<(Vec<NtgMetadata>, Vec<StringRecord>), AppError> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut records = reader.records();

    let header_data: Vec<StringRecord> = records.by_ref().take(4).collect::<Result<_, _>>()?;
    let headers = parse_csv_headers(header_data)?;
    let remaining: Vec<StringRecord> = records.collect::<Result<_, _>>()?;
    Ok((headers, remaining))
}

fn parse_csv_headers(headers: Vec<StringRecord>) -> Result<Vec<NtgMetadata>, AppError> {
    let [names, required, ctype_clength, ntg_header] = headers.as_slice() else {
        return Err(AppError::CsvMalformed {
            detail: format!("expected 4 metadata rows, found {}", headers.len()),
        });
    };

    let mut out = Vec::with_capacity(names.len());

    for i in 0..names.len() {
        let original_header = normalize_original_header(names.get(i).unwrap_or_default());
        let (ctype, clength) =
            parse_ctype_clength(ctype_clength.get(i).ok_or_else(|| AppError::CsvMalformed {
                detail: format!("no ctype/clength for column {i}"),
            })?)?;

        let required = required
            .get(i)
            .is_some_and(|f| f.trim().eq_ignore_ascii_case("m"));
        let column_index = i as i32;
        let ntg_header = ntg_header.get(i).ok_or_else(|| AppError::CsvMalformed {
            detail: format!("no ntg header for column {i}"),
        })?;
        let ntg_header = normalize_ntg_header(ntg_header, i);

        out.push(NtgMetadata {
            ntg_header: ntg_header,
            original_header: original_header,
            content_type: ctype,
            content_length: clength,
            required: required,
            column_index: column_index,
        });
    }

    Ok(out)
}
