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
                metadata JSONB,
                namespace TEXT DEFAULT 'project',
                project_id TEXT DEFAULT 'global'
            )"
        ).execute(&self.pool).await?;

        // Check if namespace column exists, if not add it (for existing databases)
        sqlx::query(
            "DO $$ 
             BEGIN 
                IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='embeddings' AND column_name='namespace') THEN 
                    ALTER TABLE embeddings ADD COLUMN namespace TEXT DEFAULT 'project';
                END IF; 
             END $$;"
        ).execute(&self.pool).await?;

        // Check if project_id column exists, if not add it (for existing databases)
        sqlx::query(
            "DO $$ 
             BEGIN 
                IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='embeddings' AND column_name='project_id') THEN 
                    ALTER TABLE embeddings ADD COLUMN project_id TEXT DEFAULT 'global';
                END IF; 
             END $$;"
        ).execute(&self.pool).await?;

        // Add index for better performance
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_embeddings_project_namespace ON embeddings(project_id, namespace)"
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                state JSONB,
                project_id TEXT
            )"
        ).execute(&self.pool).await?;

        // Check if project_id column exists in sessions, if not add it
        sqlx::query(
            "DO $$ 
             BEGIN 
                IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='sessions' AND column_name='project_id') THEN 
                    ALTER TABLE sessions ADD COLUMN project_id TEXT;
                END IF; 
             END $$;"
        ).execute(&self.pool).await?;

        // Create a table for project metadata cache
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project_metadata (
                project_id TEXT PRIMARY KEY,
                metadata JSONB,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        ).execute(&self.pool).await?;

        Ok(())
    }
}

#[async_trait]
impl VectorDbProvider for PostgresProvider {
    async fn store(&self, id: &str, vector: Vec<f32>, metadata: serde_json::Value, namespace: &str) -> Result<()> {
        self.store_with_project(id, vector, metadata, namespace, "global").await
    }

    async fn search(&self, vector: Vec<f32>, limit: usize, namespace: &str) -> Result<Vec<serde_json::Value>> {
        self.search_with_project(vector, limit, namespace, "global").await
    }

    async fn store_with_project(&self, id: &str, vector: Vec<f32>, metadata: serde_json::Value, namespace: &str, project_id: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO embeddings (id, vector, metadata, namespace, project_id) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id) DO UPDATE SET vector = $2, metadata = $3, namespace = $4, project_id = $5",
        )
        .bind(id)
        .bind(vector)
        .bind(metadata)
        .bind(namespace)
        .bind(project_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn search_with_project(&self, vector: Vec<f32>, limit: usize, namespace: &str, project_id: &str) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_as::<_, (serde_json::Value,)>(
            "SELECT metadata FROM embeddings WHERE namespace = $3 AND project_id = $4 ORDER BY vector <=> $1 LIMIT $2",
        )
        .bind(vector)
        .bind(limit as i64)
        .bind(namespace)
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }
}

impl PostgresProvider {
    /// Store project metadata in the cache
    pub async fn store_project_metadata(&self, project_id: &str, metadata: serde_json::Value) -> Result<()> {
        sqlx::query(
            "INSERT INTO project_metadata (project_id, metadata, updated_at) VALUES ($1, $2, CURRENT_TIMESTAMP)
             ON CONFLICT (project_id) DO UPDATE SET metadata = $2, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(project_id)
        .bind(metadata)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Retrieve project metadata from the cache
    pub async fn get_project_metadata(&self, project_id: &str) -> Result<Option<serde_json::Value>> {
        let row = sqlx::query_as::<_, (serde_json::Value,)>(
            "SELECT metadata FROM project_metadata WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0))
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
