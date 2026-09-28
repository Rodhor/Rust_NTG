pub mod connection;

// Swap guard to make database changes cause a minimal rewrite
pub type DbPool = sqlx::PgPool;
