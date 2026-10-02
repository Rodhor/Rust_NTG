pub mod connection;
pub mod queries;

// Swap guard to make database changes cause a minimal rewrite
pub type DbPool = sqlx::PgPool;
pub type DbConnection = sqlx::PgConnection;
