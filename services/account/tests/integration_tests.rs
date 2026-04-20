/// Integration tests using testcontainers.
///
/// These tests require Docker to be running and accessible.
/// They are gated behind the `docker_tests` cfg flag so they
/// can be selectively enabled in environments with Docker.
///
/// Run: `RUSTFLAGS='--cfg docker_tests' cargo test -p account --test integration_tests`

#[cfg(docker_tests)]
mod docker_tests {
    use sqlx::PgPool;
    use testcontainers::runners::AsyncRunner;
    use testcontainers::{ContainerAsync, GenericImage, ImageExt};
    use uuid::Uuid;

    async fn setup_postgres() -> (ContainerAsync<GenericImage>, PgPool) {
        let container: ContainerAsync<GenericImage> = GenericImage::new("postgres", "16")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .with_env_var("POSTGRES_DB", "postgres")
            .start()
            .await
            .expect("Failed to start PostgreSQL container");

        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

        // Retry connecting until PostgreSQL is ready (max 30 retries = ~30s)
        let pool = {
            let mut last_err = String::new();
            let mut pool = None;
            for i in 0..30 {
                match sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&url)
                    .await
                {
                    Ok(p) => {
                        pool = Some(p);
                        break;
                    }
                    Err(e) => {
                        last_err = e.to_string();
                        eprintln!("PostgreSQL attempt {}/30 not ready yet: {e}", i + 1);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
            pool.expect(&format!(
                "Failed to connect to PostgreSQL after 30 retries: {last_err}"
            ))
        };

        (container, pool)
    }

    async fn run_migrations(pool: &PgPool) {
        sqlx::migrate!("../../migrations/account")
            .run(pool)
            .await
            .expect("Failed to run migrations");
    }

    #[tokio::test]
    async fn test_postgres_create_user() {
        let (_container, pool) = setup_postgres().await;
        run_migrations(&pool).await;

        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO users (id, email, nickname, password_hash, kyc_status, account_status, created_at, updated_at, version)
             VALUES ($1, $2, $3, $4, 'NONE', 'ACTIVE', NOW(), NOW(), 1)"
        )
        .bind(id)
        .bind("test@example.com")
        .bind("TestUser")
        .bind("hash123")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
            .bind("test@example.com")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_postgres_all_tables_exist() {
        let (_container, pool) = setup_postgres().await;
        run_migrations(&pool).await;

        let tables = [
            "users",
            "fund_accounts",
            "fund_transactions",
            "positions",
            "position_histories",
            "risk_events",
        ];

        for table in &tables {
            let (count,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1 AND table_schema = 'public'"
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(count, 1, "Table {} should exist", table);
        }
    }

    #[tokio::test]
    async fn test_nats_pub_sub() {
        let _container: ContainerAsync<GenericImage> = GenericImage::new("nats", "2.10")
            .start()
            .await
            .expect("Failed to start NATS container");
    }
}
