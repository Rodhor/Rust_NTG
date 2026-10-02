use csv::StringRecord;

use crate::csv::helpers::{normalize_ntg_header, normalize_original_header, parse_ctype_clength};
use crate::domain::{error::AppError, ntg_column::NtgColumnType, ntg_metadata::NtgMetadata};

fn load_csv(path: &str) -> Result<Vec<NtgMetadata>, AppError> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut records = reader.records();

    let meta: Vec<StringRecord> = records.by_ref().take(4).collect::<Result<_, _>>()?;

    let [names, required, ctype_clength, ntg_header] = meta.as_slice() else {
        return Err(AppError::CsvMalformed {
            detail: format!("expected 4 metadata rows, found {}", meta.len()),
        });
    };

    let mut out = Vec::with_capacity(names.len());

    for i in 0..names.len() {
        let original_header = normalize_original_header(names.get(i).unwrap_or_default());
        let (ctype, clength) = parse_ctype_clength(ctype_clength.get(i).unwrap_or_default())
            .unwrap_or((NtgColumnType::An, None));

        let required = required
            .get(i)
            .is_some_and(|f| f.trim().eq_ignore_ascii_case("m"));
        let column_index = Some(i as i32);
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
