use crate::providers::VectorDbProvider;
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
