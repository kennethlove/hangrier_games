#![allow(dead_code)]

use api::AppState;
use axum::Router;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::opt::Config;
use surrealdb::opt::auth::Root;
use surrealdb_migrations::MigrationRunner;

/// Test configuration for SurrealDB
pub struct TestDb {
    pub db: Arc<Surreal<Any>>,
    pub namespace: String,
    pub database: String,
}

impl TestDb {
    /// Create a new test database connection (in-memory)
    pub async fn new() -> Self {
        // Use in-memory database for tests (no external dependencies).
        // Provision root credentials at engine init so the subsequent signin
        // (and downstream JWT-based authenticate calls) succeed against a
        // fresh `mem://` instance.
        let config = Config::new().user(Root {
            username: "root",
            password: "root",
        });
        let db = Arc::new(
            surrealdb::engine::any::connect(("mem://", config))
                .await
                .expect("Failed to create in-memory database"),
        );

        // Sign in as root
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .expect("Failed to authenticate to test database");

        // Use test namespace and database
        let test_db_name = format!(
            "test_{}",
            uuid::Uuid::new_v4().to_string().replace('-', "_")
        );
        let test_ns = "hangry-games-test".to_string();
        db.use_ns(&test_ns)
            .use_db(&test_db_name)
            .await
            .expect("Failed to use test database");

        // Run migrations.
        //
        // The runner reads `schemas/` and `migrations/` from a folder it
        // resolves via either `SURREAL_MIG_PATH`, a `.surrealdb` ini file,
        // or its CWD. Both env-var and "real" workspace-root paths fall
        // over under parallel tests:
        //
        //   * `std::env::set_var` is not thread-safe versus `getenv` calls
        //     happening on other threads — observed as `Failed to apply
        //     migrations: EOF while parsing` (PR #230's `OnceLock` setter
        //     doesn't help).
        //   * Pointing every test at the real workspace tree races even
        //     worse: `MigrationRunner::up()` *rewrites*
        //     `migrations/definitions/*.json` mid-run, so concurrent
        //     readers in sibling test threads/processes observe a
        //     mid-write empty file and serde panics with EOF.
        //   * A single per-process tempdir was tried previously but
        //     in-process parallelism (libtest spawns N test threads)
        //     still races on the same shared `migrations/definitions/`
        //     tree — same EOF panic in CI.
        //
        // Fix: copy `schemas/` + `migrations/` into a private *per-test*
        // tempdir (cheap — these are small files), point a `.surrealdb`
        // ini at that copy, and run migrations from there. Each test
        // owns its own tree so concurrent rewrites are impossible.
        // Belt-and-suspenders: a global mutex serializes the actual
        // `up()` invocation against the shared template directory we
        // copy from, since `read_dir` over a directory being written by
        // the migration tool can also EOF.
        let config_path = build_isolated_migration_root();
        let _guard = MIGRATION_LOCK.lock().await;
        MigrationRunner::new(&db)
            .use_config_file(&config_path)
            .up()
            .await
            .expect("Failed to apply migrations");
        drop(_guard);

        TestDb {
            db,
            namespace: test_ns,
            database: test_db_name,
        }
    }

    /// Get the AppState for testing
    pub fn app_state(&self) -> AppState {
        use api::storage::LocalStorage;
        use api::websocket::GameBroadcaster;

        AppState {
            db: self.db.clone(),
            storage: Arc::new(LocalStorage::new("test_uploads", "/uploads")),
            broadcaster: Arc::new(GameBroadcaster::default()),
            namespace: self.namespace.clone(),
            database: self.database.clone(),
        }
    }

    /// Clean up the test database
    pub async fn cleanup(&self) {
        // Note: In a real test setup, you might want to drop the entire database
        // For now, we'll just clear the tables
        let _ = self.db.query("REMOVE DATABASE $this").await;
    }
}

/// Build a fresh per-test migration root by copying `schemas/` and
/// `migrations/` from a process-cached source tree (`SOURCE_ROOT`) into
/// a unique tempdir, then writing a `.surrealdb` ini that points at the
/// copy. Each test gets its own tree so the migration runner's mid-run
/// rewrites of `migrations/definitions/*.json` cannot race against any
/// other test thread. See the comment in [`TestDb::new`] for full
/// rationale.
fn build_isolated_migration_root() -> std::path::PathBuf {
    let src = source_migration_root();
    let dst = std::env::temp_dir().join(format!(
        "hg-mig-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ));
    std::fs::create_dir_all(&dst).expect("create migration tempdir");
    for sub in ["schemas", "migrations"] {
        copy_dir_recursive(&src.join(sub), &dst.join(sub))
            .unwrap_or_else(|e| panic!("copy {sub} into tempdir: {e}"));
    }
    let cfg = dst.join(".surrealdb");
    std::fs::write(&cfg, format!("[core]\npath={}\n", dst.display()))
        .expect("write .surrealdb config");
    cfg
}

/// Process-wide source-of-truth for migration files. We do NOT point
/// tests at the workspace tree directly because `MigrationRunner::up()`
/// rewrites files in `migrations/definitions/`. Instead, copy the
/// workspace tree into a per-process tempdir once, treat that as the
/// read-only source, and snapshot from it on every test.
fn source_migration_root() -> std::path::PathBuf {
    static SOURCE: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    SOURCE
        .get_or_init(|| {
            let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("api crate must have a parent (workspace root)");
            let dst = std::env::temp_dir().join(format!(
                "hg-mig-src-{}-{}",
                std::process::id(),
                uuid::Uuid::new_v4().simple()
            ));
            std::fs::create_dir_all(&dst).expect("create migration source tempdir");
            for sub in ["schemas", "migrations"] {
                copy_dir_recursive(&workspace_root.join(sub), &dst.join(sub))
                    .unwrap_or_else(|e| panic!("seed {sub} source tempdir: {e}"));
            }
            dst
        })
        .clone()
}

/// Serializes `MigrationRunner::up()` calls. Belt-and-suspenders on top
/// of per-test tempdirs: while each test's *destination* tree is
/// private, libtest's many threads can still all enter `up()` at the
/// same time, and the runner has historically been fragile under heavy
/// concurrent invocation. The lock is cheap (held only for the
/// migration call) and removes the last source of intermittent EOF
/// panics in CI.
static MIGRATION_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else if ty.is_file() {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Create a test router with the API routes
pub fn create_test_router(state: AppState) -> Router {
    use api::auth::AUTH_ROUTER;
    use api::games::GAMES_ROUTER;
    use api::users::{USERS_PROTECTED_ROUTER, USERS_PUBLIC_ROUTER};
    use api::websocket::websocket_handler;
    use axum::middleware;
    use axum::routing::get;

    let api_routes = Router::new()
        .nest(
            "/games",
            GAMES_ROUTER
                .clone()
                .layer(middleware::from_fn_with_state(state.clone(), surreal_jwt)),
        )
        .nest("/users", USERS_PUBLIC_ROUTER.clone())
        .nest(
            "/users",
            USERS_PROTECTED_ROUTER
                .clone()
                .layer(middleware::from_fn_with_state(state.clone(), surreal_jwt)),
        )
        .nest("/auth", AUTH_ROUTER.clone());

    Router::new()
        .nest("/api", api_routes)
        .route("/ws", get(websocket_handler))
        .route(
            "/health",
            axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }),
        )
        .with_state(state)
}

/// JWT authentication middleware for tests
async fn surreal_jwt(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::http::header::AUTHORIZATION;
    use axum::response::IntoResponse;
    use base64_url::decode;
    use surrealdb::opt::auth::Jwt;
    use time::OffsetDateTime;

    let token = match request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
    {
        Some(token) => token,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let token_parts: Vec<&str> = token.split('.').collect();
    if token_parts.len() != 3 {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let payload_base64 = token_parts[1].trim_start_matches("=");
    let payload_bytes = decode(payload_base64).map_err(|_| ()).unwrap_or_default();
    let payload_str = String::from_utf8(payload_bytes).unwrap_or_default();
    let payload: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();

    let exp = payload["exp"].as_u64().unwrap_or_default();
    let now = OffsetDateTime::now_utc().unix_timestamp() as u64;
    if exp < now {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let jwt = Jwt::from(token);
    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    if user_db.authenticate(jwt).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut request = request;
    request.extensions_mut().insert(api::AuthDb(user_db));
    next.run(request).await
}

/// Test user credentials
pub struct TestUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl TestUser {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
            email: format!("{}@test.com", username),
            password: "TestPass123!".to_string(),
            access_token: None,
            refresh_token: None,
        }
    }

    pub fn with_tokens(mut self, access_token: String, refresh_token: String) -> Self {
        self.access_token = Some(access_token);
        self.refresh_token = Some(refresh_token);
        self
    }

    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token.as_ref().unwrap())
    }
}
