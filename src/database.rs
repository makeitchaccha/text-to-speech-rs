use std::sync::Arc;
use text_to_speech_rs::profile::repository::ProfileRepository;

pub enum WrappedPool {
    Sqlite(sqlx::Pool<sqlx::Sqlite>),
    Postgres(sqlx::Pool<sqlx::Postgres>),
}

impl WrappedPool {
    pub async fn migrate_up(&self) -> anyhow::Result<()> {
        match &self {
            WrappedPool::Sqlite(pool) => {
                sqlx::migrate!("./migrations/sqlite").run(pool).await?;
                Ok(())
            },
            WrappedPool::Postgres(pool) => {
                sqlx::migrate!("./migrations/postgres").run(pool).await?;
                Ok(())
            }
        }
    }

    pub fn profile_repository(&self) -> Arc<dyn ProfileRepository> {
        match &self {
            WrappedPool::Sqlite(pool) => {
                #[cfg(feature = "sqlite")]
                {
                    use text_to_speech_rs::profile::repository::sqlite::SQLiteProfileRepository;
                    Arc::new(SQLiteProfileRepository::new(pool.clone()))
                }
                #[cfg(not(feature = "sqlite"))]
                unreachable!("sqlite feature must be enabled to create this pool")
            },
            WrappedPool::Postgres(pool) => {
                #[cfg(feature = "postgres")]
                {
                    use text_to_speech_rs::profile::repository::postgres::PostgresRepository;
                    Arc::new(PostgresRepository::new(pool.clone()))
                }
                #[cfg(not(feature = "postgres"))]
                unreachable!("postgres feature must be enabled to create this pool")
            }
        }
    }
}