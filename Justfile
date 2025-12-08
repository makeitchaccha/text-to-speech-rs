set dotenv-load


db_url_postgres := "postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}"
db_url_sqlite := "sqlite://${SQLITE_FILE}"

@ensure-sqlx:
    which sqlx > /dev/null || (echo "Error: sqlx-cli not found. Install it with 'cargo install sqlx-cli --features postgres,sqlite'" && exit 1)

dbs-up:
    @echo "🚨 Starting Docker containers (Postgres)..."
    docker compose up -d postgres
    @echo "Waiting for Postgres..."
    sleep 3
    touch $SQLITE_FILE

dbs-down:
    @echo "🛑 Stopping containers and removing local DB file..."
    docker compose down
    rm -f ${SQLITE_FILE}

migrate: ensure-sqlx
    @echo "📦 Applying migrations..."
    # SQLite
    touch $SQLITE_FILE
    sqlx database create --database-url {{db_url_sqlite}}
    sqlx migrate run --source migrations/sqlite --database-url {{db_url_sqlite}}
    # Postgres
    sqlx database create --database-url {{db_url_postgres}}
    sqlx migrate run --source migrations/postgres --database-url {{db_url_postgres}}

prepare: dbs-up migrate
    @echo "✨ Generating offline SQLx metadata (Merging PG and SQLite)..."
    @echo " -> [1/3] Preparing Postgres metadata..."
    rm -rf .sqlx
    mkdir -p .sqlx
    cargo sqlx prepare \
        --database-url {{db_url_postgres}} \
        -- --features postgres --no-default-features
    mkdir -p .sqlx_tmp
    cp -rf .sqlx/* .sqlx_tmp/

    @echo " -> [2/3] Preparing SQLite metadata..."
    cargo sqlx prepare \
        --database-url {{db_url_sqlite}} \
        -- --features sqlite --no-default-features

    @echo " -> [3/3] Merging metadata..."
    mv .sqlx_tmp/*.json -t .sqlx
    rmdir .sqlx_tmp
    @echo "✅ Cache generation successful!"

reset: dbs-down
    @echo "🧹 Cleaning up..."
    rm -f $SQLITE_FILE sqlx-data.json
    just prepare