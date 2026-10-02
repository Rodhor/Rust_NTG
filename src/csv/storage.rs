use crate::{
    csv::reader,
    db::{
        DbPool,
        queries::ntg_header::{self, NewNtgHeader},
    },
    domain::error::AppError,
};

pub async fn store_csv_data(pool: &DbPool, local_path: &str) -> Result<usize, AppError> {
    let (headers, _) = reader::load_csv(local_path)?;
    let rows: Vec<NewNtgHeader> = headers.into_iter().map(NewNtgHeader::from).collect();

    let written = ntg_header::upsert_bulk(pool, rows).await?;
    Ok(written.len())
}
