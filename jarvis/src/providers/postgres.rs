use crate::providers::{VectorDbProvider, PersistenceProvider};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Pool, Postgres};

pub struct PostgresProvider {
    pool: Pool<Postgres>,
}

impl PostgresProvider {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn setup(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS embeddings (
                id TEXT PRIMARY KEY,
                vector vector(384),
                metadata JSONB
            )"
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                state JSONB
            )"
        ).execute(&self.pool).await?;

        Ok(())
    }
}

#[async_trait]
impl VectorDbProvider for PostgresProvider {
    async fn store(&self, id: &str, vector: Vec<f32>, metadata: serde_json::Value) -> Result<()> {
        sqlx::query(
            "INSERT INTO embeddings (id, vector, metadata) VALUES ($1, $2, $3)
             ON CONFLICT (id) DO UPDATE SET vector = $2, metadata = $3",
        )
        .bind(id)
        .bind(vector)
        .bind(metadata)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_as::<_, (serde_json::Value,)>(
            "SELECT metadata FROM embeddings ORDER BY vector <=> $1 LIMIT $2",
        )
        .bind(vector)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }
}

#[async_trait]
impl PersistenceProvider for PostgresProvider {
    async fn save_state(&self, session_id: &str, state: serde_json::Value) -> Result<()> {
        sqlx::query(
            "INSERT INTO sessions (id, state) VALUES ($1, $2)
             ON CONFLICT (id) DO UPDATE SET state = $2",
        )
        .bind(session_id)
        .bind(state)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load_state(&self, session_id: &str) -> Result<Option<serde_json::Value>> {
        let row = sqlx::query_as::<_, (serde_json::Value,)>(
            "SELECT state FROM sessions WHERE id = $1",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0))
    }
}
