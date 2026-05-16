# Codebase Index: hangrier_games

> Generated: 2026-05-16 05:05:22 UTC | Files: 317 | Lines: 82970
> Languages: CSS (1), JSON (9), Markdown (77), Rust (217), TOML (8), XML (4), YAML (1)

## Directory Structure

```
hangrier_games/
  AGENTS.md
  CLAUDE.md
  Cargo.toml
  GAME_REVIEW.md
  README.md
  TASKS_8-12_SUMMARY.md
  TASK_12_FRONTEND_NOTES.md
  TASK_7_IMPLEMENTATION.md
  VALIDATION_SUMMARY.md
  announcers/
    Cargo.toml
    codemap.md
    src/
      codemap.md
      lib.rs
      main.rs
  api/
    Cargo.toml
    codemap.md
    src/
      auth.rs
      cleanup.rs
      codemap.md
      cookies.rs
      games.rs
      lib.rs
      main.rs
      storage.rs
      tributes.rs
      users.rs
      websocket.rs
    tests/
      IMPLEMENTATION_SUMMARY.md
      README.md
      WEBSOCKET_TESTING.md
      auth_tests.rs
      common/
        mod.rs
      game_customization_test.rs
      games_tests.rs
      simulation_tests.rs
      tributes_tests.rs
      websocket_tests.rs
  codemap.md
  docker-compose.yaml
  docs/
    superpowers/
      plans/
        2026-04-17-event-severity-integration.md
        2026-04-17-terrain-biome-system.md
        2026-04-18-unify-event-systems-33r.md
        2026-04-25-tribute-alliances-implementation.md
        2026-04-26-combat-wire-redesign.md
        2026-04-26-game-timeline-pr1-backend.md
        2026-04-26-game-timeline-pr2-frontend.md
        2026-05-03-break-mid-swing-penalty.md
        2026-05-03-gamemaker-event-system-pr1-backend.md
        2026-05-03-gamemaker-event-system-pr2-frontend.md
        2026-05-03-shelter-hunger-thirst-pr1-backend.md
        2026-05-03-shelter-hunger-thirst-pr2-frontend.md
        2026-05-03-stamina-combat-resource-pr1-backend.md
        2026-05-03-stamina-combat-resource-pr2-frontend.md
        2026-05-04-addiction-pr1.md
        2026-05-04-afflictions-pr1.md
        2026-05-04-design-system-v1.md
        2026-05-04-sponsorship-pr1.md
        2026-05-04-trapped-afflictions-pr1.md
        2026-05-04-trauma-pr1.md
      specs/
        2026-04-17-event-severity-integration.md
        2026-04-17-terrain-biome-system-design.md
        2026-04-25-tribute-alliances-design.md
        2026-04-26-game-event-enum.md
        2026-04-26-game-timeline-redesign.md
        2026-05-01-hex-arena-map-design.md
        2026-05-02-progressive-display-design.md
        2026-05-02-spectator-skin-layout-design.md
        2026-05-02-spectator-skin-visuals-design.md
        2026-05-02-tribute-emotions-design.md
        2026-05-02-weather-system-design.md
        2026-05-03-break-mid-swing-design.md
        2026-05-03-fixations-design.md
        2026-05-03-four-phase-day-design.md
        2026-05-03-gamemaker-event-system-design.md
        2026-05-03-health-conditions-design.md
        2026-05-03-phobias-design.md
        2026-05-03-shelter-hunger-thirst-design.md
        2026-05-03-stamina-combat-resource-design.md
        2026-05-04-addiction-design.md
        2026-05-04-design-system-v1-design.md
        2026-05-04-sponsorship-design.md
        2026-05-04-trapped-afflictions-design.md
        2026-05-04-trauma-design.md
  game/
    Cargo.toml
    benches/
      game_cycle_bench.rs
    codemap.md
    src/
      areas/
        codemap.md
        events.rs
        forage.rs
        hex.rs
        mod.rs
        path.rs
        shelter.rs
        water.rs
        weather.rs
      codemap.md
      config.rs
      districts.rs
      events.rs
      games.rs
      items/
        codemap.md
        mod.rs
        name_generator.rs
      lib.rs
      messages.rs
      output.rs
      pathfinding.rs
      phases/
        environment.rs
        mod.rs
      terrain/
        assignment.rs
        config.rs
        descriptors.rs
        mod.rs
        types.rs
      threats/
        animals.rs
        codemap.md
        mod.rs
      tributes/
        actions.rs
        alliances.rs
        brains.rs
        codemap.md
        combat.rs
        combat_beat.rs
        combat_tuning.rs
        events.rs
        inventory.rs
        lifecycle.rs
        mod.rs
        movement.rs
        stamina_band.rs
        statuses.rs
        survival.rs
        traits.rs
      witty_phrase_generator/
        codemap.md
        mod.rs
    tests/
      ai_terrain_behavior_test.rs
      district_affinity_test.rs
      event_game_loop_test.rs
      event_integration_test.rs
      event_severity_test.rs
      event_unification_area_events_test.rs
      event_unification_combat_test.rs
      event_unification_movement_test.rs
      item_distribution_test.rs
      narrative_test.rs
      stamina_combat_integration.rs
      stamina_edge_cases_test.rs
      survival_integration.rs
      terrain_assignment_test.rs
      terrain_compatibility_test.rs
      terrain_config_test.rs
      terrain_specific_events_test.rs
  migrations/
    codemap.md
    definitions/
      20260419_133608_ItemDurability.json
      20260427_120000_GameEventPayload.json
      20260427_180000_GameMessagePayloadV2.json
      20260501_120000_TributeAlliesString.json
      20260501_180000_DisplayGameWinnerTributeRef.json
      20260503_120000_TributeSurvivalFields.json
      _initial.json
      codemap.md
  rustfmt.toml
  schemas/
    codemap.md
  shared/
    Cargo.toml
    codemap.md
    src/
      codemap.md
      combat_beat.rs
      lib.rs
      messages.rs
  src/
    lib.rs
  web/
    Cargo.toml
    Dioxus.toml
    assets/
      icons.svg
      images/
        map.svg
        waves.svg
      package-lock.json
      package.json
      src/
        main.css
    build.rs
    codemap.md
    src/
      api_url.rs
      cache.rs
      codemap.md
      components/
        accounts.rs
        app.rs
        area_detail.rs
        button.rs
        codemap.md
        create_game.rs
        credits.rs
        filter_chips.rs
        game_areas.rs
        game_delete.rs
        game_detail.rs
        game_edit.rs
        game_period_page.rs
        game_tributes.rs
        games.rs
        games_list.rs
        home.rs
        icons/
          codemap.md
          delete.rs
          edit.rs
          eye_closed.rs
          eye_open.rs
          game_icons_net/
            broken_bone.rs
            burned.rs
            codemap.md
            dead.rs
            dehydrated.rs
            drowning.rs
            electrocuted.rs
            falling_rocks.rs
            fishing_net.rs
            fist.rs
            fizzing_flask.rs
            frozen_body.rs
            harpoon_trident.rs
            health_potion.rs
            hearts.rs
            heat_haze.rs
            high_shot.rs
            hypodermic_test.rs
            infection.rs
            mauled.rs
            mod.rs
            plain_dagger.rs
            pointy_sword.rs
            poison_bottle.rs
            powder.rs
            recently_dead.rs
            shield.rs
            spear_hook.rs
            spiked_mace.rs
            spinning_top.rs
            spray.rs
            starving.rs
            switchblade.rs
            trail_mix.rs
            vomiting.rs
            wood_axe.rs
            wounded.rs
          loading.rs
          lock_closed.rs
          lock_open.rs
          map_pin.rs
          mockingjay.rs
          mockingjay_arrow.rs
          mockingjay_flight.rs
          mod.rs
          svg_icon.rs
          uturn.rs
        icons_page.rs
        info_detail.rs
        input.rs
        item_detail.rs
        item_icon.rs
        loading_modal.rs
        map.rs
        map_affordance_overlay.rs
        mod.rs
        modal.rs
        navbar.rs
        period_card.rs
        period_grid.rs
        period_grid_empty.rs
        recap_card.rs
        server_version.rs
        timeline/
          cards/
            alliance_card.rs
            combat_card.rs
            combat_swing_card.rs
            cycle_card.rs
            death_card.rs
            item_card.rs
            mod.rs
            movement_card.rs
            sleep_card.rs
            stamina_card.rs
            state_card.rs
            survival_card.rs
            wake_card.rs
          event_card.rs
          filters.rs
          mod.rs
          timeline.rs
        tribute_delete.rs
        tribute_detail.rs
        tribute_edit.rs
        tribute_filter_chips.rs
        tribute_state_strip.rs
        tribute_status_icon.rs
        tribute_survival_section.rs
        ui/
          button.rs
          event_card.rs
          live_pill.rs
          mod.rs
          scoreboard.rs
          section_label.rs
          sidebar_hud.rs
          ticker.rs
          topbar.rs
          tribute_row.rs
      hooks/
        mod.rs
        use_game_websocket.rs
        use_timeline_summary.rs
      http.rs
      lib.rs
      main.rs
      routes.rs
      storage.rs
      theme.rs
    tests/
      button_test.rs
      map_test.rs
      mod.rs
      modal_test.rs
      tribute_status_icon_test.rs
    web/
      assets/
        icons.svg
```

---

## Public API Surface

**AGENTS.md**
- `# Agent Instructions`
- `# OR: cargo run --package api`
- `# Requires: SurrealDB running on SURREAL_HOST, .env file present`
- `# OR: cd web && dx serve`
- `# Install dx first: just setup-dx`
- `# Requires: APP_API_HOST in .env, Tailwind CSS built`
- `# OR: cd web/assets && npm install && npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css`
- `# OR: cargo test --package game`
- `# WARNING: Tests may be slow; workspace-wide `cargo test` can hang`
- `# OR: cargo fmt`
- `# 1. Make commits on a descriptive bookmark (not main)`
- `# 2. Push the bookmark to origin`
- `# 3. Open a PR with gh`

**CLAUDE.md**
- `# hangrier_games`

**Cargo.toml**
- `[workspace]`
- `[profile]`
- `[profile.dev]`
- `[profile.dev.package."*"]`
- `[profile.wasm-dev]`
- `[profile.server-dev]`
- `[profile.android-dev]`

**GAME_REVIEW.md**
- `# Hangrier Games - Comprehensive Game Development Review`

**README.md**
- `# Hangrier Games`
- `# Clone the repository`
- `# Install all dependencies (Dioxus CLI, Node packages, Ollama model)`
- `# Build Tailwind CSS`
- `# Start the full development environment (SurrealDB + API + web frontend)`
- `# Install dependencies`
- `# Create Ollama model (optional)`
- `# Build Tailwind CSS`
- `# Start services (in separate terminals)`
- `# Terminal 1: SurrealDB`
- `# Terminal 2: API server`
- `# Terminal 3: Web frontend`
- `# Terminal 1: Start backend services`
- `# Terminal 2: Start frontend with hot reload`
- `# Rebuild CSS after changing Tailwind classes`
- `# Run game crate tests only (recommended)`
- `# Run all workspace tests (WARNING: may be slow)`
- `# Build Docker images`
- `# Start all services`
- `# View logs`
- `# Stop services`
- `# Run game crate tests only (recommended)`
- `# Run all workspace tests (WARNING: may hang)`

**TASKS_8-12_SUMMARY.md**
- `# Terrain/Biome System Implementation - Tasks 8-12 Summary`
- `# Run all tests`
- `# Check compilation`
- `# View commits`

**TASK_12_FRONTEND_NOTES.md**
- `# Frontend Terrain UI Changes`

**TASK_7_IMPLEMENTATION.md**
- `# Task 7: AI Behavior Modifications - Implementation Summary`

**VALIDATION_SUMMARY.md**
- `# Input Validation Implementation Summary`
- `# Invalid username (too short)`
- `# Expected: 400 with "Username must be between 3 and 50 characters"`
- `# Invalid password (too short)`
- `# Expected: 400 with "Password must be between 8 and 128 characters"`
- `# Invalid game name (empty)`
- `# Expected: 400 with "Game name cannot be empty"`
- `# Invalid UUID`
- `# Expected: 400 with "invalid_uuid"`

**announcers/Cargo.toml**
- `[package]`
- `[dependencies]`

**announcers/codemap.md**
- `# announcers/`

**announcers/src/codemap.md**
- `# announcers/src/`

**announcers/src/lib.rs**
- `pub enum AnnouncerError`
- `pub static MODEL: &str = "announcers"`
- `pub static ANNOUNCER_PROMPT: &str = r#" You are writing a sports broadcast team covering the newest Hunger Games. Provide the spoken script for Verity and Rex directly with no summaries or conclusions. Now, here is this cycle's log entry: "#`
- `pub fn prompt(log: &str) -> String`
- `pub async fn summarize(log: &str) -> Result<String, AnnouncerError>`
- `pub async fn summarize_stream( log: &str, ) -> Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>`

**api/Cargo.toml**
- `[package]`
- `[dependencies]`
- `[dev-dependencies]`
- `[target.'cfg(windows)'.dependencies]`
- `[target.'cfg(unix)'.dependencies]`

**api/codemap.md**
- `# api/`

**api/src/auth.rs**
- `pub const JWT_SECRET: &str = "6dxLjU0m8ZmAzaLEk_qAeMpeD5ZAjGYlCjlvDi5DcgdJLATIHuCReUu7CbGyCDhRSp3btd7Ezob7RPYe6fUtsA"`
- `pub static AUTH_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(||`
- `pub struct RefreshToken`
- `pub struct RefreshTokenRequest`
- `pub struct TokenResponse`
- `pub async fn store_refresh_token( db: &surrealdb::Surreal<surrealdb::engine::any::Any>, refresh_token: &RefreshToken, ) -> Result<(), AppError>`
- `pub async fn get_refresh_token( db: &surrealdb::Surreal<surrealdb::engine::any::Any>, token: &str, ) -> Result<RefreshToken, AppError>`
- `pub async fn revoke_refresh_token( db: &surrealdb::Surreal<surrealdb::engine::any::Any>, token: &str, ) -> Result<(), AppError>`

**api/src/cleanup.rs**
- `pub async fn cleanup_refresh_tokens(state: &AppState) -> Result<usize, AppError>`
- `pub async fn start_cleanup_scheduler(state: AppState) -> Result<JobScheduler, AppError>`

**api/src/codemap.md**
- `# api/src/`

**api/src/cookies.rs**
- `pub const SESSION_COOKIE: &str = "hg_session"`
- `pub const REFRESH_COOKIE: &str = "hg_refresh"`
- `pub fn set_session_cookie(response: &mut Response, jwt: &str)`
- `pub fn set_refresh_cookie(response: &mut Response, token: &str)`
- `pub fn clear_auth_cookies(response: &mut Response)`
- `pub fn read_cookie<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> Option<&'a str>`

**api/src/games.rs**
- `pub struct PaginatedTributes`
- `pub struct PaginationParams`
- `pub static GAMES_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(||`
- `pub struct GameAreaEdge`
- `pub async fn create_game( Extension(AuthDb(db)): Extension<AuthDb>, Json(payload): Json<CreateGame>, ) -> Result<Response, AppError>`
- `pub async fn create_area( game_identifier: &str, area: Area, num_items: u32, db: &Surreal<Any>, ) -> Result<(), AppError>`
- `pub async fn add_item_to_area( game_area_edge: &GameAreaEdge, terrain: Option<BaseTerrain>, db: &Surreal<Any>, ) -> Result<(), AppError>`
- `pub async fn game_delete( game_identifier: Path<Uuid>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<StatusCode, AppError>`
- `pub async fn game_list( Extension(AuthDb(db)): Extension<AuthDb>, Query(params): Query<PaginationParams>, ) -> Result<Json<PaginatedGames>, AppError>`
- `pub async fn game_detail( game_identifier: Path<Uuid>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<DisplayGame>, AppError>`
- `pub async fn game_update( Path(game_identifier): Path<Uuid>, Extension(AuthDb(db)): Extension<AuthDb>, Json(payload): Json<EditGame>, ) -> Result<Json<Game>, AppError>`
- `pub async fn game_areas( Path(identifier): Path<Uuid>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<Vec<AreaDetails>>, AppError>`
- `pub async fn area_detail( Path((game_identifier, area_identifier)): Path<(Uuid, Uuid)>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<AreaDetails>, AppError>`
- `pub async fn item_detail( Path((game_identifier, item_identifier)): Path<(Uuid, Uuid)>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<Item>, AppError>`
- `pub async fn game_tributes( Path(identifier): Path<Uuid>, Extension(AuthDb(db)): Extension<AuthDb>, Query(params): Query<PaginationParams>, ) -> Result<Json<PaginatedTributes>, AppError>`
- `pub async fn next_step( Path(identifier): Path<Uuid>, state: State<AppState>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<Option<Game>>, AppError>`
- `pub async fn game_display( game_identifier: Path<Uuid>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<DisplayGame>, AppError>`

**api/src/lib.rs**
- `pub mod auth`
- `pub mod cleanup`
- `pub mod cookies`
- `pub mod games`
- `pub mod storage`
- `pub mod tributes`
- `pub mod users`
- `pub mod websocket`
- `pub struct AppState`
- `pub struct AuthDb(pub Surreal<Any>)`
- `pub enum AppError`
- `pub async fn verify_record_persisted( db: &Surreal<Any>, rid: &RecordId, site: &'static str, ) -> Result<(), AppError>`

**api/src/main.rs**
- `pub static DATABASE: LazyLock<Arc<Surreal<Any>>> = LazyLock::new(|| Arc::new(Surreal::init()))`

**api/src/storage.rs**
- `pub struct UploadConstraints`
- `pub trait StorageBackend: Send + Sync`
- `pub struct LocalStorage`
- `pub fn validate_upload( data: &[u8], filename: &str, constraints: &UploadConstraints, ) -> Result<(), AppError>`

**api/src/tributes.rs**
- `pub static TRIBUTES_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(||`
- `pub struct TributeItemEdge`
- `pub async fn create_tribute( tribute: Option<Tribute>, game_identifier: &str, db: &Surreal<Any>, district: u32, ) -> Result<Tribute, AppError>`
- `pub async fn tribute_delete( Path((_, tribute_identifier)): Path<(String, String)>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<StatusCode, AppError>`
- `pub async fn tribute_update( Path((_game_identifier, _tribute_identifier)): Path<(Uuid, Uuid)>, Extension(AuthDb(db)): Extension<AuthDb>, Json(payload): Json<EditTribute>, ) -> Result<StatusCode, AppError>`
- `pub async fn tribute_detail( Path((_, tribute_identifier)): Path<(Uuid, Uuid)>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<Tribute>, AppError>`
- `pub async fn tribute_log( Path((_, identifier)): Path<(Uuid, Uuid)>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<Vec<GameMessage>>, AppError>`
- `pub async fn upload_avatar( Path((_, tribute_identifier)): Path<(Uuid, Uuid)>, State(state): State<AppState>, Extension(AuthDb(db)): Extension<AuthDb>, mut multipart: Multipart, ) -> Result<Json<serde_json::Value>, AppError>`

**api/src/users.rs**
- `pub static USERS_PUBLIC_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(||`
- `pub static USERS_PROTECTED_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| Router::new().route("/session", get(session)))`

**api/src/websocket.rs**
- `pub struct GameBroadcaster`
- `pub async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response`
- `pub fn broadcast_game_message(broadcaster: &GameBroadcaster, game_id: &str, message: GameMessage)`
- `pub fn broadcast_game_started(broadcaster: &GameBroadcaster, game_id: &str, day: u32)`
- `pub fn broadcast_game_finished( broadcaster: &GameBroadcaster, game_id: &str, winner: Option<String>, )`

**api/tests/IMPLEMENTATION_SUMMARY.md**
- `# API Integration Tests - Implementation Summary`
- `# All tests`
- `# Specific test file`
- `# Single test`

**api/tests/README.md**
- `# API Integration Tests`

**api/tests/WEBSOCKET_TESTING.md**
- `# WebSocket Integration Tests`
- `# Run all WebSocket tests`
- `# Run specific test`
- `# Run with output`

**api/tests/common/mod.rs**
- `pub struct TestDb`
- `pub fn create_test_router(state: AppState) -> Router`
- `pub struct TestUser`

**codemap.md**
- `# Repository Atlas: Hangrier Games`
- `# Clone repository`
- `# Install all dependencies (Dioxus CLI, Node packages, Ollama model)`
- `# Create .env file (if not present)`
- `# Start full dev environment (DB + API + web)`
- `# Make changes to code, hot reload happens automatically`
- `# Run tests before committing`
- `# Format code`
- `# Full quality gate before PR`
- `# In one terminal: Start SurrealDB + API`
- `# In another terminal: Start frontend with hot reload`
- `# Rebuild Tailwind CSS after changes to classes`
- `# Start SurrealDB with trace logging`
- `# Access SurrealDB console`
- `# Migrations run automatically at API startup`
- `# Schema files: schemas/*.surql`
- `# Initial state: migrations/definitions/_initial.json`
- `# Create custom Ollama model for commentary`
- `# Verify model exists`
- `# Build everything (optimized release builds)`
- `# Output locations:`
- `# - API: target/release/api`
- `# - Web: web/dist/ (WASM + JS glue + assets)`
- `# Run with Docker Compose`

**docker-compose.yaml**
- `version:`
- `services:`
- `networks:`

**docs/superpowers/plans/2026-04-17-event-severity-integration.md**
- `# Event Severity Integration Implementation Plan`

**docs/superpowers/plans/2026-04-17-terrain-biome-system.md**
- `# Terrain/Biome System Implementation Plan`

**docs/superpowers/plans/2026-04-18-unify-event-systems-33r.md**
- `# Unify Event Systems Implementation Plan`
- `# PR1 — Combat Narration Restoration`
- `# PR2 — Movement and Turn-Phase Narration Restoration`
- `# PR3 — Survival Enrichment and Area-Event Narration`

**docs/superpowers/plans/2026-04-25-tribute-alliances-implementation.md**
- `# Tribute Alliances Implementation Plan`

**docs/superpowers/plans/2026-04-26-combat-wire-redesign.md**
- `# Combat Wire Redesign Implementation Plan`

**docs/superpowers/plans/2026-04-26-game-timeline-pr1-backend.md**
- `# Game Timeline PR1 — Backend Schema + Combat Refactor + Frontend Stub`

**docs/superpowers/plans/2026-04-26-game-timeline-pr2-frontend.md**
- `# Game Timeline PR2 — Frontend Implementation Plan`

**docs/superpowers/plans/2026-05-03-break-mid-swing-penalty.md**
- `# Break-Mid-Swing Penalty Implementation Plan`

**docs/superpowers/plans/2026-05-03-gamemaker-event-system-pr1-backend.md**
- `# Gamemaker Event System PR1 — Backend Implementation Plan`

**docs/superpowers/plans/2026-05-03-gamemaker-event-system-pr2-frontend.md**
- `# Gamemaker Event System — Plan 2: Frontend Implementation`

**docs/superpowers/plans/2026-05-03-shelter-hunger-thirst-pr1-backend.md**
- `# Shelter + Hunger/Thirst — Plan 1: Backend Implementation`

**docs/superpowers/plans/2026-05-03-shelter-hunger-thirst-pr2-frontend.md**
- `# Shelter + Hunger/Thirst — Plan 2: Frontend Implementation`

**docs/superpowers/plans/2026-05-03-stamina-combat-resource-pr1-backend.md**
- `# Stamina-as-Combat-Resource PR1 — Backend Implementation Plan`

**docs/superpowers/plans/2026-05-03-stamina-combat-resource-pr2-frontend.md**
- `# Stamina-as-Combat-Resource PR2 — Frontend Implementation Plan`

**docs/superpowers/plans/2026-05-04-addiction-pr1.md**
- `# Addiction PR1 — Types, Storage, `try_acquire_addiction` Implementation Plan`

**docs/superpowers/plans/2026-05-04-afflictions-pr1.md**
- `# Afflictions PR1 Implementation Plan`

**docs/superpowers/plans/2026-05-04-design-system-v1.md**
- `# Hangrier Games Design System v1 — Implementation Plan`

**docs/superpowers/plans/2026-05-04-sponsorship-pr1.md**
- `# Sponsorship System — PR1 Implementation Plan`

**docs/superpowers/plans/2026-05-04-trapped-afflictions-pr1.md**
- `# Trapped Afflictions PR1 Implementation Plan`

**docs/superpowers/plans/2026-05-04-trauma-pr1.md**
- `# Trauma PR1 — Types, Storage, `try_acquire_trauma` Implementation Plan`
- `# If anything was modified by `cargo fmt`:`

**docs/superpowers/specs/2026-04-17-event-severity-integration.md**
- `# Event Severity Integration Design`

**docs/superpowers/specs/2026-04-17-terrain-biome-system-design.md**
- `# Terrain/Biome System & Game Customization Design`

**docs/superpowers/specs/2026-04-25-tribute-alliances-design.md**
- `# Tribute Alliances — Design Spec`

**docs/superpowers/specs/2026-04-26-game-event-enum.md**
- `# GameEvent enum — structured replacement for GameOutput`

**docs/superpowers/specs/2026-04-26-game-timeline-redesign.md**
- `# Game Timeline Redesign`

**docs/superpowers/specs/2026-05-01-hex-arena-map-design.md**
- `# Hex-tile arena map (v1)`

**docs/superpowers/specs/2026-05-02-progressive-display-design.md**
- `# Progressive Display — Design`

**docs/superpowers/specs/2026-05-02-spectator-skin-layout-design.md**
- `# Spectator Skin — Layout, Behavior, and Chrome`

**docs/superpowers/specs/2026-05-02-spectator-skin-visuals-design.md**
- `# Spectator Skin — Visual Identity`

**docs/superpowers/specs/2026-05-02-tribute-emotions-design.md**
- `# Tribute Emotions & Outlook — Design`

**docs/superpowers/specs/2026-05-02-weather-system-design.md**
- `# Weather System — Design`

**docs/superpowers/specs/2026-05-03-break-mid-swing-design.md**
- `# Break-mid-swing penalty`

**docs/superpowers/specs/2026-05-03-fixations-design.md**
- `# Fixations — v1 Design`

**docs/superpowers/specs/2026-05-03-four-phase-day-design.md**
- `# Four-Phase Day — v1 Design`

**docs/superpowers/specs/2026-05-03-gamemaker-event-system-design.md**
- `# Gamemaker Event System — Design`

**docs/superpowers/specs/2026-05-03-health-conditions-design.md**
- `# Health Conditions (Afflictions) — v1 Design`

**docs/superpowers/specs/2026-05-03-phobias-design.md**
- `# Phobias — v1 Design`

**docs/superpowers/specs/2026-05-03-shelter-hunger-thirst-design.md**
- `# Shelter + Hunger/Thirst — Design`

**docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md**
- `# Stamina-as-Combat-Resource — Design`

**docs/superpowers/specs/2026-05-04-addiction-design.md**
- `# Addiction affliction system — design`

**docs/superpowers/specs/2026-05-04-design-system-v1-design.md**
- `# Hangrier Games — Design System (v1)`

**docs/superpowers/specs/2026-05-04-sponsorship-design.md**
- `# Sponsorship System v1 — Design Spec`

**docs/superpowers/specs/2026-05-04-trapped-afflictions-design.md**
- `# Trapped Afflictions — Design Spec`

**docs/superpowers/specs/2026-05-04-trauma-design.md**
- `# Trauma affliction system — design`

**game/Cargo.toml**
- `[package]`
- `[dependencies]`
- `[dev-dependencies]`
- `[[bench]]`

**game/codemap.md**
- `# game/`

**game/src/areas/codemap.md**
- `# game/src/areas/`

**game/src/areas/events.rs**
- `pub enum AreaEvent`
- `pub enum EventSeverity`
- `pub struct SurvivalResult`

**game/src/areas/forage.rs**
- `pub fn forage_richness(terrain: BaseTerrain) -> u8`

**game/src/areas/hex.rs**
- `pub struct Axial`
- `pub fn default_layout() -> [(Area, Axial); 7]`
- `pub struct SubAxial`
- `pub const SUB_SLOTS: [SubAxial`
- `pub const SUB_SIZE_RATIO: f64 = 1.0 / 3.0`

**game/src/areas/mod.rs**
- `pub mod events`
- `pub mod forage`
- `pub mod hex`
- `pub mod path`
- `pub mod shelter`
- `pub mod water`
- `pub mod weather`
- `pub enum Area`
- `pub struct DestinationInfo`
- `pub struct AreaDetails`

**game/src/areas/path.rs**
- `pub const CLOSED_PENALTY: u32 = 1000`
- `pub struct AreaGraph<'a>`
- `pub fn plan_path( areas: &[AreaDetails], closed: &[Area], tribute: &Tribute, start: Area, goal: Area, ) -> Option<(Vec<Area>, u32)>`

**game/src/areas/shelter.rs**
- `pub fn shelter_quality(terrain: BaseTerrain, weather: &Weather) -> u8`

**game/src/areas/water.rs**
- `pub fn water_source(terrain: BaseTerrain, weather: &Weather) -> u8`

**game/src/areas/weather.rs**
- `pub enum Weather`
- `pub fn current_weather() -> Weather`

**game/src/codemap.md**
- `# game/src/`

**game/src/config.rs**
- `pub struct GameConfig`

**game/src/districts.rs**
- `pub struct DistrictProfile`
- `pub const DISTRICT_PROFILES: [DistrictProfile; 12] = [ DistrictProfile`
- `pub fn assign_terrain_affinity(district: u8, rng: &mut impl Rng) -> Vec<BaseTerrain>`

**game/src/events.rs**
- `pub enum GameEvent`

**game/src/games.rs**
- `pub enum GameError`
- `pub struct TickCounter`
- `pub struct Game`

**game/src/items/codemap.md**
- `# game/src/items/`

**game/src/items/mod.rs**
- `pub enum ItemRarity`
- `pub enum WearOutcome`
- `pub enum ItemError`
- `pub trait OwnsItems`
- `pub struct Item`
- `pub enum ItemType`
- `pub enum Attribute`
- `pub trait ConsumableAttribute`

**game/src/items/name_generator.rs**
- `pub fn generate_shield_name() -> String`
- `pub fn generate_weapon_name() -> String`

**game/src/lib.rs**
- `pub mod areas`
- `pub mod config`
- `pub mod districts`
- `pub mod events`
- `pub mod games`
- `pub mod items`
- `pub mod messages`
- `pub mod output`
- `pub mod pathfinding`
- `pub mod phases`
- `pub mod terrain`
- `pub mod threats`
- `pub mod tributes`

**game/src/messages.rs**
- `pub struct TaggedEvent`
- `pub fn movement_narrative(terrain: BaseTerrain, tribute_name: &str) -> String`
- `pub fn hiding_spot_narrative(terrain: BaseTerrain, tribute_name: &str) -> String`
- `pub fn stamina_narrative(terrain: BaseTerrain, current_stamina: u32) -> String`

**game/src/output.rs**
- `pub enum GameOutput<'a>`

**game/src/pathfinding.rs**
- `pub trait Graph`
- `pub fn astar<G: Graph>( graph: &G, start: G::Node, goal: G::Node, ) -> Option<(Vec<G::Node>, G::Cost)>`

**game/src/phases/environment.rs**
- `pub enum LightLevel`
- `pub struct AfflictionDraft`
- `pub struct AreaPhaseConditions`
- `pub fn derive_light_level(phase: Phase, biome: BaseTerrain, weather: Weather) -> LightLevel`
- `pub fn roll_environmental_afflictions( phase: Phase, biome: BaseTerrain, weather: Weather, sheltered: bool, rng: &mut impl Rng, ) -> Vec<AfflictionDraft>`

**game/src/phases/mod.rs**
- `pub mod environment`

**game/src/terrain/assignment.rs**
- `pub fn enforce_balance_constraint(terrains: &mut [TerrainType], rng: &mut impl Rng)`

**game/src/terrain/config.rs**
- `pub enum Visibility`
- `pub enum Harshness`
- `pub struct ItemWeights`

**game/src/terrain/mod.rs**
- `pub mod assignment`
- `pub mod config`
- `pub mod descriptors`
- `pub mod types`

**game/src/terrain/types.rs**
- `pub enum BaseTerrain`
- `pub enum TerrainDescriptor`
- `pub struct TerrainType`

**game/src/threats/animals.rs**
- `pub enum Animal`

**game/src/threats/codemap.md**
- `# game/src/threats/`

**game/src/tributes/actions.rs**
- `pub struct TributeAction`
- `pub enum Action`
- `pub enum AttackResult`
- `pub enum AttackOutcome`

**game/src/tributes/alliances.rs**
- `pub const MAX_ALLIES: usize = 5`
- `pub const BASE_ALLIANCE_CHANCE: f64 = 0.20`
- `pub const TREACHEROUS_BETRAYAL_INTERVAL: u8 = 5`
- `pub enum AllianceEvent`
- `pub fn passes_gate(self_traits: &[Trait], target_traits: &[Trait]) -> bool`
- `pub fn roll_chance( self_traits: &[Trait], target_traits: &[Trait], same_district: bool, self_allies_len: usize, target_allies_len: usize, ) -> f64`
- `pub enum DecidingFactor`
- `pub fn deciding_factor( self_traits: &[Trait], target_traits: &[Trait], same_district: bool, ) -> Option<DecidingFactor>`
- `pub fn sanity_break_roll( current_sanity: u32, low_sanity_limit: u32, rng: &mut impl rand::Rng, ) -> bool`
- `pub fn trust_shock_roll( current_sanity: u32, low_sanity_limit: u32, rng: &mut impl rand::Rng, ) -> bool`
- `pub fn try_form_alliance( self_traits: &[Trait], target_traits: &[Trait], same_district: bool, self_allies_len: usize, target_allies_len: usize, rng: &mut impl rand::Rng, ) -> bool`

**game/src/tributes/brains.rs**
- `pub enum PsychoticBreakType`
- `pub struct PersonalityThresholds`
- `pub struct Brain`
- `pub fn survival_override( tribute: &Tribute, terrain: BaseTerrain, weather: &crate::areas::weather::Weather, in_combat: bool, ) -> Option<Action>`
- `pub fn stamina_override( tribute: &Tribute, nearby: &[Tribute], sheltered: bool, tuning: &crate::tributes::combat_tuning::CombatTuning, ) -> Option<Action>`
- `pub fn target_attack_score( actor: &Tribute, target: &Tribute, tuning: &crate::tributes::combat_tuning::CombatTuning, ) -> i32`
- `pub fn action_score( actor: &Tribute, action: &Action, _nearby: &[Tribute], tuning: &crate::tributes::combat_tuning::CombatTuning, ) -> i32`

**game/src/tributes/codemap.md**
- `# game/src/tributes/`

**game/src/tributes/combat.rs**
- `pub struct AttackContestOutcome`
- `pub fn attack_contest( attacker: &mut Tribute, target: &mut Tribute, rng: &mut impl Rng, events: &mut Vec<TaggedEvent>, tuning: &crate::tributes::combat_tuning::CombatTuning, ) -> AttackContestOutcome`
- `pub fn update_stats(attacker: &mut Tribute, defender: &mut Tribute, result: AttackResult)`

**game/src/tributes/combat_beat.rs**
- `pub trait CombatBeatExt`

**game/src/tributes/combat_tuning.rs**
- `pub struct CombatTuning`

**game/src/tributes/events.rs**
- `pub enum TributeEvent`

**game/src/tributes/mod.rs**
- `pub mod actions`
- `pub mod alliances`
- `pub mod brains`
- `pub mod combat`
- `pub mod combat_beat`
- `pub mod combat_tuning`
- `pub mod events`
- `pub mod inventory`
- `pub mod lifecycle`
- `pub mod movement`
- `pub mod stamina_band`
- `pub mod statuses`
- `pub mod survival`
- `pub mod traits`
- `pub struct ActionSuggestion`
- `pub struct EnvironmentContext<'a>`
- `pub struct EncounterContext`
- `pub struct Tribute`
- `pub fn calculate_stamina_cost( action: &Action, terrain: &crate::terrain::TerrainType, tribute: &Tribute, ) -> u32`
- `pub struct Statistics`
- `pub struct Attributes`

**game/src/tributes/movement.rs**
- `pub enum TravelResult`

**game/src/tributes/stamina_band.rs**
- `pub fn stamina_band(stamina: u32, max_stamina: u32, tuning: &CombatTuning) -> StaminaBand`

**game/src/tributes/statuses.rs**
- `pub enum TributeStatus`

**game/src/tributes/survival.rs**
- `pub fn hunger_band(value: u8) -> HungerBand`
- `pub fn thirst_band(value: u8) -> ThirstBand`
- `pub fn hunger_band_is_public(band: HungerBand) -> bool`
- `pub fn thirst_band_is_public(band: ThirstBand) -> bool`
- `pub fn tick_survival(tribute: &mut Tribute, weather: &Weather, sheltered: bool)`
- `pub fn apply_starvation_drain(tribute: &mut Tribute) -> u32`
- `pub fn apply_dehydration_drain(tribute: &mut Tribute) -> u32`
- `pub fn eat_food(tribute: &mut Tribute, amount: u8)`
- `pub fn drink_water(tribute: &mut Tribute, amount: u8)`

**game/src/tributes/traits.rs**
- `pub enum Trait`
- `pub const REFUSERS: &[Trait] = &[Trait::Paranoid, Trait::LoneWolf]`
- `pub fn geometric_mean_affinity(traits: &[Trait]) -> f64`
- `pub const CONFLICTS: &[(Trait, Trait)] = &[ (Trait::Friendly, Trait::Paranoid), (Trait::Loyal, Trait::Treacherous), (Trait::Loyal, Trait::LoneWolf), (Trait::Aggressive, Trait::Cautious), (Trait::Aggressive, Trait::Defensive), (Trait::Reckless, Trait::Cautious), (Trait::Resilient, Trait::Fragile), (Trait::Cunning, Trait::Dim), ]`
- `pub fn conflicts_with(a: Trait, b: Trait) -> bool`
- `pub const DISTRICT_1_POOL: &[(Trait, u8)] = &[ (Trait::Loyal, 4), (Trait::Aggressive, 4), (Trait::Paranoid, 3), (Trait::Tough, 2), ]`
- `pub const DISTRICT_2_POOL: &[(Trait, u8)] = &[ (Trait::Aggressive, 4), (Trait::Defensive, 4), (Trait::Loyal, 3), (Trait::Tough, 2), ]`
- `pub const DISTRICT_3_POOL: &[(Trait, u8)] = &[ (Trait::Cunning, 4), (Trait::Cautious, 3), (Trait::Dim, 2), (Trait::Nearsighted, 2), (Trait::Asthmatic, 1), ]`
- `pub const DISTRICT_4_POOL: &[(Trait, u8)] = &[ (Trait::Resilient, 4), (Trait::Aggressive, 3), (Trait::Loyal, 3), (Trait::Tough, 2), ]`
- `pub const DISTRICT_5_POOL: &[(Trait, u8)] = &[ (Trait::Cunning, 4), (Trait::Cautious, 3), (Trait::Treacherous, 2), ]`
- `pub const DISTRICT_6_POOL: &[(Trait, u8)] = &[ (Trait::Fragile, 3), (Trait::Friendly, 3), (Trait::Asthmatic, 2), (Trait::Nearsighted, 2), ]`
- `pub const DISTRICT_7_POOL: &[(Trait, u8)] = &[ (Trait::Resilient, 4), (Trait::Defensive, 3), (Trait::Tough, 3), ]`
- `pub const DISTRICT_8_POOL: &[(Trait, u8)] = &[ (Trait::Fragile, 2), (Trait::Friendly, 4), (Trait::Loyal, 3), (Trait::Asthmatic, 2), ]`
- `pub const DISTRICT_9_POOL: &[(Trait, u8)] = &[ (Trait::Cautious, 3), (Trait::Friendly, 3), (Trait::Asthmatic, 2), ]`
- `pub const DISTRICT_10_POOL: &[(Trait, u8)] = &[ (Trait::Resilient, 4), (Trait::Defensive, 3), (Trait::Tough, 3), ]`
- `pub const DISTRICT_11_POOL: &[(Trait, u8)] = &[ (Trait::Loyal, 3), (Trait::Friendly, 4), (Trait::Resilient, 3), (Trait::Tough, 2), ]`
- `pub const DISTRICT_12_POOL: &[(Trait, u8)] = &[ (Trait::Resilient, 3), (Trait::LoneWolf, 3), (Trait::Cunning, 3), (Trait::Asthmatic, 2), ]`
- `pub fn pool_for(district: u8) -> &'static [(Trait, u8)]`
- `pub fn generate_traits(district: u8, rng: &mut impl Rng) -> Vec<Trait>`
- `pub struct ThresholdDelta`

**game/src/witty_phrase_generator/codemap.md**
- `# game/src/witty_phrase_generator/`

**game/src/witty_phrase_generator/mod.rs**
- `pub struct WPGen`

**migrations/codemap.md**
- `# migrations/`

**migrations/definitions/codemap.md**
- `# migrations/definitions/`

**schemas/codemap.md**
- `# schemas/`

**shared/Cargo.toml**
- `[package]`
- `[dependencies]`

**shared/codemap.md**
- `# shared/`

**shared/src/codemap.md**
- `# shared/src/`

**shared/src/combat_beat.rs**
- `pub enum WearOutcomeReport`
- `pub struct WearReport`
- `pub enum SwingOutcome`
- `pub struct StressReport`
- `pub struct CombatBeat`

**shared/src/lib.rs**
- `pub mod combat_beat`
- `pub mod messages`
- `pub enum WebSocketMessage`
- `pub enum ItemQuantity`
- `pub enum EventFrequency`
- `pub struct CreateGame`
- `pub type DeleteTribute = String`
- `pub struct DeleteGame(pub String, pub String)`
- `pub struct EditTribute`
- `pub struct EditGame`
- `pub struct GameArea`
- `pub struct RegistrationUser`
- `pub struct AuthenticatedUser`
- `pub enum GameStatus`
- `pub struct DisplayGame`
- `pub struct CreatedBy`
- `pub struct UserSession`
- `pub struct ListDisplayGame`
- `pub struct PaginationMetadata`
- `pub struct PaginatedGames`

**shared/src/messages.rs**
- `pub const CAUSE_STARVATION: &str = "starvation"`
- `pub const CAUSE_DEHYDRATION: &str = "dehydration"`
- `pub enum MessageSource`
- `pub enum Phase`
- `pub struct ParsePhaseError`
- `pub struct TributeRef`
- `pub struct AreaRef`
- `pub struct ItemRef`
- `pub enum AreaEventKind`
- `pub struct CombatEngagement`
- `pub enum CombatOutcome`
- `pub enum DrinkSource`
- `pub enum MessageKind`
- `pub enum StaminaBand`
- `pub enum HungerBand`
- `pub enum ThirstBand`
- `pub enum WakeReason`
- `pub enum InterruptionKind`
- `pub enum MessagePayload`
- `pub struct GameMessage`
- `pub struct PeriodSummary`
- `pub struct TimelineSummary`
- `pub fn summarize_periods(messages: &[GameMessage], current: (u32, Phase)) -> Vec<PeriodSummary>`

**web/Cargo.toml**
- `[package]`
- `[dependencies]`
- `[build-dependencies]`
- `[features]`
- `[dependencies.getrandom]`

**web/Dioxus.toml**
- `[application]`
- `[web.app]`
- `[web.watcher]`
- `[web.resource]`
- `[web.resource.dev]`
- `[[web.proxy]]`
- `[[web.proxy]]`

**web/assets/icons.svg**
- `<svg>`

**web/assets/images/map.svg**
- `<svg>`
- `<defs>`
- `<g>`

**web/assets/images/waves.svg**
- `<svg>`
- `<style>`
- `<defs>`
- `<path>`

**web/assets/package-lock.json**
- `"name": "assets"`
- `"lockfileVersion": 3`
- `"requires": true`
- `"packages": {`

**web/assets/package.json**
- `"dependencies": {`

**web/assets/src/main.css**
- `--color-bg: #19121A`
- `--color-surface: #241829`
- `--color-surface-2: #1F1521`
- `--color-border: #3A2440`
- `--color-text: #F2EBE2`
- `--color-text-muted: #A498A2`
- `--color-primary: #00E5FF`
- `--color-danger: #FF2E6E`
- `--color-gold: #E8B14B`
- `.light`
- `--color-bg: #FBF6E9`
- `--color-surface: #FFFCF2`
- `--color-surface-2: #F4EAC9`
- `--color-border: #E8DCB8`
- `--color-text: #1A1410`
- `--color-text-muted: #6B5938`
- `--color-primary: #007A99`
- `--color-danger: #C8003C`
- `--color-gold: #B8861B`
- `--color-bg: var(--color-bg)`
- `--color-surface: var(--color-surface)`
- `--color-surface-2: var(--color-surface-2)`
- `--color-border: var(--color-border)`
- `--color-text: var(--color-text)`
- `--color-text-muted: var(--color-text-muted)`
- `--color-primary: var(--color-primary)`
- `--color-danger: var(--color-danger)`
- `--color-gold: var(--color-gold)`
- `--font-display: "Bebas Neue", Impact, sans-serif`
- `--font-text: "Source Sans 3", system-ui, sans-serif`
- `--font-mono: "IBM Plex Mono", ui-monospace, monospace`
- `--radius-card: 0.625rem;   /* 10px */`
- `--radius-inner: 0.375rem;  /* 6px */`
- `.frame`
- `@keyframes spinner`
- `.spinner`

**web/codemap.md**
- `# web/`

**web/src/api_url.rs**
- `pub fn api_url(path: &str) -> String`

**web/src/codemap.md**
- `# web/src/`

**web/src/components/accounts.rs**
- `pub fn Accounts() -> Element`
- `pub fn AccountsPage() -> Element`

**web/src/components/app.rs**
- `pub fn App() -> Element`

**web/src/components/area_detail.rs**
- `pub fn AreaDetail(game_identifier: String, area_identifier: String) -> Element`

**web/src/components/button.rs**
- `pub struct ButtonProps`
- `pub fn Button(props: ButtonProps) -> Element`
- `pub fn ThemedButton(props: ButtonProps) -> Element`

**web/src/components/codemap.md**
- `# web/src/components/`

**web/src/components/create_game.rs**
- `pub fn CreateGameButton() -> Element`
- `pub fn CreateGameForm() -> Element`

**web/src/components/credits.rs**
- `pub fn Credits() -> Element`

**web/src/components/filter_chips.rs**
- `pub struct FilterChipsProps`
- `pub fn FilterChips(props: FilterChipsProps) -> Element`

**web/src/components/game_areas.rs**
- `pub fn GameAreaList(game: DisplayGame) -> Element`

**web/src/components/game_delete.rs**
- `pub fn GameDelete(game_identifier: String, game_name: String, icon_class: String) -> Element`
- `pub fn DeleteGameModal() -> Element`

**web/src/components/game_detail.rs**
- `pub fn GamePage(identifier: String) -> Element`

**web/src/components/game_edit.rs**
- `pub fn GameEdit(identifier: String, name: String, icon_class: String, private: bool) -> Element`
- `pub fn EditGameModal() -> Element`
- `pub fn EditGameForm() -> Element`

**web/src/components/game_period_page.rs**
- `pub fn GamePeriodPage( identifier: String, day: u32, phase: Phase, filter: String, tribute: String, ) -> Element`

**web/src/components/game_tributes.rs**
- `pub struct PaginatedTributesResponse`
- `pub fn GameTributes(game: DisplayGame) -> Element`
- `pub fn GameTributeListMember( tribute: Tribute, game_identifier: String, game_status: GameStatus, current_phase: Option<u32>, ) -> Element`

**web/src/components/games.rs**
- `pub fn Games() -> Element`

**web/src/components/games_list.rs**
- `pub struct PaginatedGamesResponse`
- `pub fn GamesList() -> Element`
- `pub fn GameListMember(game: ListDisplayGame) -> Element`

**web/src/components/home.rs**
- `pub fn Home() -> Element`

**web/src/components/icons/codemap.md**
- `# web/src/components/icons/`

**web/src/components/icons/delete.rs**
- `pub fn DeleteIcon(class: String) -> Element`

**web/src/components/icons/edit.rs**
- `pub fn EditIcon(class: String) -> Element`

**web/src/components/icons/eye_closed.rs**
- `pub fn EyeClosedIcon(class: String) -> Element`

**web/src/components/icons/eye_open.rs**
- `pub fn EyeOpenIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/broken_bone.rs**
- `pub fn BrokenBoneIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/burned.rs**
- `pub fn BurnedIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/codemap.md**
- `# web/src/components/icons/game_icons_net/`

**web/src/components/icons/game_icons_net/dead.rs**
- `pub fn DeadIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/dehydrated.rs**
- `pub fn DehydratedIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/drowning.rs**
- `pub fn DrowningIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/electrocuted.rs**
- `pub fn ElectrocutedIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/falling_rocks.rs**
- `pub fn FallingRocksIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/fishing_net.rs**
- `pub fn FishingNetIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/fist.rs**
- `pub fn FistIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/fizzing_flask.rs**
- `pub fn FizzingFlaskIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/frozen_body.rs**
- `pub fn FrozenBodyIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/harpoon_trident.rs**
- `pub fn HarpoonTridentIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/health_potion.rs**
- `pub fn HealthPotionIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/hearts.rs**
- `pub fn HeartsIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/heat_haze.rs**
- `pub fn HeatHazeIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/high_shot.rs**
- `pub fn HighShotIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/hypodermic_test.rs**
- `pub fn HypodermicTestIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/infection.rs**
- `pub fn InfectionIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/mauled.rs**
- `pub fn MauledIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/plain_dagger.rs**
- `pub fn PlainDaggerIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/pointy_sword.rs**
- `pub fn PointySwordIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/poison_bottle.rs**
- `pub fn PoisonBottleIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/powder.rs**
- `pub fn PowderIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/recently_dead.rs**
- `pub fn RecentlyDeadIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/shield.rs**
- `pub fn ShieldIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/spear_hook.rs**
- `pub fn SpearHookIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/spiked_mace.rs**
- `pub fn SpikedMaceIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/spinning_top.rs**
- `pub fn SpinningTopIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/spray.rs**
- `pub fn SprayIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/starving.rs**
- `pub fn StarvingIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/switchblade.rs**
- `pub fn SwitchbladeIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/trail_mix.rs**
- `pub fn TrailMixIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/vomiting.rs**
- `pub fn VomitingIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/wood_axe.rs**
- `pub fn WoodAxeIcon(class: String) -> Element`

**web/src/components/icons/game_icons_net/wounded.rs**
- `pub fn WoundedIcon(class: String) -> Element`

**web/src/components/icons/loading.rs**
- `pub fn LoadingIcon() -> Element`

**web/src/components/icons/lock_closed.rs**
- `pub fn LockClosedIcon(class: String) -> Element`

**web/src/components/icons/lock_open.rs**
- `pub fn LockOpenIcon(class: String) -> Element`

**web/src/components/icons/map_pin.rs**
- `pub fn MapPinIcon(class: String) -> Element`

**web/src/components/icons/mockingjay.rs**
- `pub fn Mockingjay(class: String) -> Element`

**web/src/components/icons/mockingjay_arrow.rs**
- `pub fn MockingjayArrow(class: String) -> Element`

**web/src/components/icons/mockingjay_flight.rs**
- `pub fn MockingjayFlight(class: String) -> Element`

**web/src/components/icons/mod.rs**
- `pub mod delete`
- `pub mod edit`
- `pub mod eye_closed`
- `pub mod eye_open`
- `pub mod game_icons_net`
- `pub mod loading`
- `pub mod lock_closed`
- `pub mod lock_open`
- `pub mod map_pin`
- `pub mod mockingjay`
- `pub mod mockingjay_arrow`
- `pub mod mockingjay_flight`
- `pub mod svg_icon`
- `pub mod uturn`

**web/src/components/icons/svg_icon.rs**
- `pub fn SvgIcon(name: String, class: String) -> Element`
- `pub fn SpriteSheetLoader() -> Element`
- `pub fn icon_name_for_item(item: &game::items::Item) -> String`

**web/src/components/icons/uturn.rs**
- `pub fn UTurnIcon(class: String) -> Element`

**web/src/components/icons_page.rs**
- `pub fn IconsPage() -> Element`

**web/src/components/info_detail.rs**
- `pub struct InfoDetailProps`
- `pub fn InfoDetail(props: InfoDetailProps) -> Element`

**web/src/components/input.rs**
- `pub struct InputProperties`
- `pub fn Input(props: InputProperties) -> Element`

**web/src/components/item_detail.rs**
- `pub fn ItemDetail(game_identifier: String, item_identifier: String) -> Element`

**web/src/components/item_icon.rs**
- `pub fn ItemIcon(item: Item, css_class: String) -> Element`

**web/src/components/loading_modal.rs**
- `pub fn LoadingModal() -> Element`

**web/src/components/map.rs**
- `pub fn Map(areas: Vec<AreaDetails>) -> Element`

**web/src/components/map_affordance_overlay.rs**
- `pub fn MapAffordanceOverlay(cx: f64, cy: f64, size: f64, area: AreaDetails) -> Element`

**web/src/components/mod.rs**
- `pub mod timeline`
- `pub mod game_tributes`
- `pub mod games_list`
- `pub mod icons`
- `pub mod modal`
- `pub mod ui`

**web/src/components/modal.rs**
- `pub struct Props`
- `pub fn Modal(modal_props: Props) -> Element`

**web/src/components/navbar.rs**
- `pub fn Navbar() -> Element`

**web/src/components/period_card.rs**
- `pub struct PeriodCardProps`
- `pub fn PeriodCard(props: PeriodCardProps) -> Element`

**web/src/components/period_grid.rs**
- `pub struct PeriodGridProps`
- `pub fn PeriodGrid(props: PeriodGridProps) -> Element`

**web/src/components/period_grid_empty.rs**
- `pub enum EmptyKind`
- `pub struct PeriodGridEmptyProps`
- `pub fn PeriodGridEmpty(props: PeriodGridEmptyProps) -> Element`

**web/src/components/recap_card.rs**
- `pub struct RecapCardProps`
- `pub fn RecapCard(props: RecapCardProps) -> Element`

**web/src/components/server_version.rs**
- `pub fn ServerVersion() -> Element`

**web/src/components/timeline/cards/alliance_card.rs**
- `pub struct AllianceCardProps`
- `pub fn AllianceCard(props: AllianceCardProps) -> Element`

**web/src/components/timeline/cards/combat_card.rs**
- `pub struct CombatCardProps`
- `pub fn CombatCard(props: CombatCardProps) -> Element`

**web/src/components/timeline/cards/combat_swing_card.rs**
- `pub struct CombatSwingCardProps`
- `pub fn CombatSwingCard(props: CombatSwingCardProps) -> Element`

**web/src/components/timeline/cards/cycle_card.rs**
- `pub struct CycleCardProps`
- `pub fn CycleCard(props: CycleCardProps) -> Element`

**web/src/components/timeline/cards/death_card.rs**
- `pub struct DeathCardProps`
- `pub fn DeathCard(props: DeathCardProps) -> Element`

**web/src/components/timeline/cards/item_card.rs**
- `pub struct ItemCardProps`
- `pub fn ItemCard(props: ItemCardProps) -> Element`

**web/src/components/timeline/cards/mod.rs**
- `pub mod alliance_card`
- `pub mod combat_card`
- `pub mod combat_swing_card`
- `pub mod cycle_card`
- `pub mod death_card`
- `pub mod item_card`
- `pub mod movement_card`
- `pub mod sleep_card`
- `pub mod stamina_card`
- `pub mod state_card`
- `pub mod survival_card`
- `pub mod wake_card`

**web/src/components/timeline/cards/movement_card.rs**
- `pub struct MovementCardProps`
- `pub fn MovementCard(props: MovementCardProps) -> Element`

**web/src/components/timeline/cards/sleep_card.rs**
- `pub struct SleepCardProps`
- `pub fn SleepCard(props: SleepCardProps) -> Element`

**web/src/components/timeline/cards/stamina_card.rs**
- `pub struct StaminaCardProps`
- `pub fn StaminaCard(props: StaminaCardProps) -> Element`

**web/src/components/timeline/cards/state_card.rs**
- `pub struct StateCardProps`
- `pub fn StateCard(props: StateCardProps) -> Element`

**web/src/components/timeline/cards/survival_card.rs**
- `pub struct SurvivalCardProps`
- `pub fn SurvivalCard(props: SurvivalCardProps) -> Element`

**web/src/components/timeline/cards/wake_card.rs**
- `pub struct WakeCardProps`
- `pub fn WakeCard(props: WakeCardProps) -> Element`

**web/src/components/timeline/event_card.rs**
- `pub struct EventCardProps`
- `pub fn EventCard(props: EventCardProps) -> Element`

**web/src/components/timeline/filters.rs**
- `pub enum FilterMode`
- `pub struct PeriodFilters`

**web/src/components/timeline/mod.rs**
- `pub mod cards`
- `pub mod event_card`
- `pub mod filters`
- `pub mod timeline`

**web/src/components/timeline/timeline.rs**
- `pub struct TimelineProps`
- `pub fn Timeline(props: TimelineProps) -> Element`

**web/src/components/tribute_delete.rs**
- `pub fn TributeDelete(tribute_name: String) -> Element`
- `pub fn DeleteTributeModal() -> Element`

**web/src/components/tribute_detail.rs**
- `pub fn TributeDetail(game_identifier: String, tribute_identifier: String) -> Element`

**web/src/components/tribute_edit.rs**
- `pub fn TributeEdit( identifier: String, name: String, avatar: String, game_identifier: String, ) -> Element`
- `pub fn EditTributeModal() -> Element`
- `pub fn EditTributeForm() -> Element`

**web/src/components/tribute_filter_chips.rs**
- `pub struct TributeFilterChipsProps`
- `pub fn TributeFilterChips(props: TributeFilterChipsProps) -> Element`

**web/src/components/tribute_state_strip.rs**
- `pub fn TributeStateStrip(tribute: Tribute, current_phase: Option<u32>) -> Element`

**web/src/components/tribute_status_icon.rs**
- `pub fn TributeStatusIcon(status: TributeStatus, css_class: String) -> Element`

**web/src/components/tribute_survival_section.rs**
- `pub fn TributeSurvivalSection(tribute: Tribute, current_phase: Option<u32>) -> Element`

**web/src/components/ui/button.rs**
- `pub enum ButtonVariant`
- `pub fn Button( #[props(default = ButtonVariant::Primary)] variant: ButtonVariant, #[props(default = false)] disabled: bool, #[props(default)] onclick: EventHandler<MouseEvent>, children: Element, ) -> Element`

**web/src/components/ui/event_card.rs**
- `pub struct EventCardProps`
- `pub fn EventCard(props: EventCardProps) -> Element`

**web/src/components/ui/live_pill.rs**
- `pub fn LivePill() -> Element`

**web/src/components/ui/mod.rs**
- `pub mod button`
- `pub mod event_card`
- `pub mod live_pill`
- `pub mod scoreboard`
- `pub mod section_label`
- `pub mod sidebar_hud`
- `pub mod ticker`
- `pub mod topbar`
- `pub mod tribute_row`

**web/src/components/ui/scoreboard.rs**
- `pub struct ScoreboardProps`
- `pub fn Scoreboard(props: ScoreboardProps) -> Element`

**web/src/components/ui/section_label.rs**
- `pub fn SectionLabel(children: Element) -> Element`

**web/src/components/ui/sidebar_hud.rs**
- `pub struct StatTileProps`
- `pub fn StatTile(props: StatTileProps) -> Element`
- `pub fn SidebarHud(header: String, children: Element) -> Element`

**web/src/components/ui/ticker.rs**
- `pub struct TickerItem`
- `pub fn Ticker(items: Vec<TickerItem>) -> Element`

**web/src/components/ui/topbar.rs**
- `pub fn TopBar(brand: String, children: Element) -> Element`

**web/src/components/ui/tribute_row.rs**
- `pub struct TributeRowProps`
- `pub fn TributeRow(props: TributeRowProps) -> Element`

**web/src/hooks/mod.rs**
- `pub mod use_game_websocket`
- `pub mod use_timeline_summary`

**web/src/hooks/use_game_websocket.rs**
- `pub enum ConnectionState`
- `pub fn use_game_websocket(game_id: String) -> (Signal<Vec<GameMessage>>, Signal<ConnectionState>)`

**web/src/http.rs**
- `pub trait WithCredentials`

**web/src/lib.rs**
- `pub mod api_url`
- `pub mod components`
- `pub mod hooks`
- `pub mod http`
- `pub mod theme`
- `pub enum LoadingState`

**web/src/routes.rs**
- `pub enum Routes`

**web/src/storage.rs**
- `pub fn use_persistent<T: Serialize + DeserializeOwned + Default + 'static>( // A unique key for the storage entry key: impl ToString, // A function that returns the initial value if the storage entry is empty init: impl FnOnce() -> T, ) -> UsePersistent<T>`
- `pub struct UsePersistent<T: 'static>`
- `pub struct AppState`

**web/src/theme.rs**
- `pub enum Theme`

**web/web/assets/icons.svg**
- `<svg>`

---

## AGENTS.md

**Language:** Markdown | **Size:** 8.3 KB | **Lines:** 224

**Declarations:**

---

## CLAUDE.md

**Language:** Markdown | **Size:** 3.8 KB | **Lines:** 63

**Declarations:**

---

## Cargo.toml

**Language:** TOML | **Size:** 296 B | **Lines:** 21

**Declarations:**

---

## GAME_REVIEW.md

**Language:** Markdown | **Size:** 18.0 KB | **Lines:** 613

**Declarations:**

---

## README.md

**Language:** Markdown | **Size:** 10.3 KB | **Lines:** 310

**Declarations:**

---

## TASKS_8-12_SUMMARY.md

**Language:** Markdown | **Size:** 8.5 KB | **Lines:** 266

**Declarations:**

---

## TASK_12_FRONTEND_NOTES.md

**Language:** Markdown | **Size:** 3.3 KB | **Lines:** 104

**Declarations:**

---

## TASK_7_IMPLEMENTATION.md

**Language:** Markdown | **Size:** 4.9 KB | **Lines:** 136

**Declarations:**

---

## VALIDATION_SUMMARY.md

**Language:** Markdown | **Size:** 4.9 KB | **Lines:** 128

**Declarations:**

---

## announcers/Cargo.toml

**Language:** TOML | **Size:** 295 B | **Lines:** 13

**Imports:**
- `async-stream`
- `futures`
- `ollama-rs`
- `thiserror`
- `tokio`
- `tokio-stream`

**Declarations:**

---

## announcers/codemap.md

**Language:** Markdown | **Size:** 1.9 KB | **Lines:** 51

**Declarations:**

---

## announcers/src/codemap.md

**Language:** Markdown | **Size:** 378 B | **Lines:** 19

**Declarations:**

---

## announcers/src/lib.rs

**Language:** Rust | **Size:** 2.3 KB | **Lines:** 85

**Imports:**
- `futures::StreamExt`
- `futures::stream::Stream`
- `ollama_rs::Ollama`
- `ollama_rs::generation::completion::request::GenerationRequest`
- `std::pin::Pin`
- `thiserror::Error`

**Declarations:**

---

## announcers/src/main.rs

**Language:** Rust | **Size:** 3.6 KB | **Lines:** 80

**Declarations:**

`async fn main()`

---

## api/Cargo.toml

**Language:** TOML | **Size:** 1.5 KB | **Lines:** 42

**Imports:**
- `announcers`
- `async-trait`
- `axum`
- `base64-url`
- `chrono`
- `futures`
- `game`
- `jsonwebtoken`
- `serde`
- `serde_json`
- *... and 18 more imports*

**Declarations:**

---

## api/codemap.md

**Language:** Markdown | **Size:** 367 B | **Lines:** 19

**Declarations:**

---

## api/src/auth.rs

**Language:** Rust | **Size:** 11.5 KB | **Lines:** 350

**Imports:**
- `crate::cookies::{
    REFRESH_COOKIE, clear_auth_cookies, read_cookie, set_refresh_cookie, set_session_cookie,
}`
- `crate::{AppError, AppState}`
- `axum::extract::State`
- `axum::http::{HeaderMap, StatusCode}`
- `axum::response::{IntoResponse, Response}`
- `axum::routing::post`
- `axum::{Json, Router}`
- `chrono::Utc`
- `jsonwebtoken::{Algorithm, EncodingKey, Header, encode}`
- `serde::{Deserialize, Serialize}`
- *... and 5 more imports*

**Declarations:**

**`impl RefreshToken`**
  `pub fn new(user_id: Thing, username: String) -> Self`

  `pub fn is_valid(&self) -> bool`


`fn generate_access_token( user_id: &Thing, username: &str, namespace: &str, database: &str, ) -> Result<String, AppError>`

`async fn refresh_token( State(state): State<AppState>, headers: HeaderMap, body: Option<Json<RefreshTokenRequest>>, ) -> Result<Response, AppError>`

`async fn logout( State(state): State<AppState>, headers: HeaderMap, body: Option<Json<RefreshTokenRequest>>, ) -> Result<Response, AppError>`

`mod tests`

---

## api/src/cleanup.rs

**Language:** Rust | **Size:** 2.6 KB | **Lines:** 78

**Imports:**
- `crate::{AppError, AppState}`
- `tokio_cron_scheduler::{Job, JobScheduler}`
- `tracing::{error, info}`

**Declarations:**

`mod tests`

---

## api/src/codemap.md

**Language:** Markdown | **Size:** 17.1 KB | **Lines:** 601

**Declarations:**

---

## api/src/cookies.rs

**Language:** Rust | **Size:** 3.0 KB | **Lines:** 85

**Imports:**
- `axum::http::HeaderValue`
- `axum::http::header::{COOKIE, SET_COOKIE}`
- `axum::response::Response`

**Declarations:**

`const SESSION_MAX_AGE: i64 = 3600`

`const REFRESH_MAX_AGE: i64 = 7 * 24 * 3600`

`fn is_secure() -> bool`

`fn build_cookie(name: &str, value: &str, path: &str, max_age: i64) -> String`

`fn build_clear_cookie(name: &str, path: &str) -> String`

`fn append_cookie(response: &mut Response, cookie: &str)`

---

## api/src/games.rs

**Language:** Rust | **Size:** 51.9 KB | **Lines:** 1481

**Imports:**
- `crate::tributes::{TRIBUTES_ROUTER, create_tribute}`
- `crate::{AppError, AppState, AuthDb}`
- `axum::Json`
- `axum::Router`
- `axum::extract::{Extension, Path, Query, State}`
- `axum::http::{HeaderValue, StatusCode, header::LOCATION}`
- `axum::response::{IntoResponse, Response}`
- `axum::routing::{get, put}`
- `chrono::{DateTime, Utc}`
- `game::areas::{Area, AreaDetails}`
- *... and 16 more imports*

**Declarations:**

`const MAX_MESSAGES: usize = 10000`

`fn default_limit() -> u32`

`fn default_offset() -> u32`

`async fn create_game_area(area: Area, db: &Surreal<Any>) -> Result<GameArea, AppError>`

`async fn create_game_area_edge( area: Area, game_identifier: Uuid, db: &Surreal<Any>, ) -> Result<GameAreaEdge, AppError>`

`async fn delete_pieces( pieces: HashMap<String, Vec<Thing>>, db: &Surreal<Any>, ) -> Result<(), AppError>`

`async fn get_game_status(db: &Surreal<Any>, identifier: &str) -> Result<GameStatus, AppError>`

`async fn update_game_status( db: &Surreal<Any>, record_id: &RecordId, status: GameStatus, ) -> Result<(), AppError>`

`async fn get_dead_tribute_count(db: &Surreal<Any>, identifier: &str) -> Result<u32, AppError>`

`async fn run_game_cycles( game: &mut Game, db: &Surreal<Any>, broadcaster: &crate::websocket::GameBroadcaster, ) -> Result<(), AppError>`

`async fn get_full_game(identifier: Uuid, db: &Surreal<Any>) -> Result<Json<Game>, AppError>`

`pub(crate) struct GameLog`
> Fields: `id: RecordId`, `identifier: String`, `source: MessageSource`, `game_day: u32`, `subject: String`, `timestamp: DateTime<Utc>`, `content: String`, `phase: shared::messages::Phase`, `tick: u32`, `emit_index: u32`, `payload: String`

**`impl From<GameLog> for GameMessage`**
  `fn from(row: GameLog) -> Self`


`async fn save_game( game: &mut Game, db: &Surreal<Any>, broadcaster: &crate::websocket::GameBroadcaster, ) -> Result<Json<Game>, AppError>`

`async fn save_area_items( items: &Vec<Item>, owner: RecordId, db: &Surreal<Any>, ) -> Result<(), AppError>`

`async fn save_tribute_items( items: &Vec<Item>, owner: RecordId, db: &Surreal<Any>, ) -> Result<(), AppError>`

`async fn game_day_logs( Path((game_identifier, day)): Path<(Uuid, u32)>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<Vec<GameMessage>>, AppError>`

`async fn tribute_logs( Path((game_identifier, day, tribute_identifier)): Path<(Uuid, u32, Uuid)>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<Vec<GameMessage>>, AppError>`

`async fn timeline_summary( Path(game_identifier): Path<Uuid>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<shared::messages::TimelineSummary>, AppError>`

`async fn publish_game( Path(game_identifier): Path<Uuid>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<serde_json::Value>, AppError>`

`async fn unpublish_game( Path(game_identifier): Path<Uuid>, Extension(AuthDb(db)): Extension<AuthDb>, ) -> Result<Json<serde_json::Value>, AppError>`

---

## api/src/lib.rs

**Language:** Rust | **Size:** 4.9 KB | **Lines:** 142

**Imports:**
- `axum::Json`
- `axum::http::StatusCode`
- `axum::response::{IntoResponse, Response}`
- `serde::Deserialize`
- `serde_json::json`
- `std::sync::Arc`
- `surrealdb::RecordId`
- `surrealdb::Surreal`
- `surrealdb::engine::any::Any`
- `thiserror::Error`
- *... and 1 more imports*

**Declarations:**

**`impl IntoResponse for AppError`**
  `fn into_response(self) -> Response`


`struct VerifyRow`
> Fields: `id: RecordId`

---

## api/src/main.rs

**Language:** Rust | **Size:** 16.0 KB | **Lines:** 464

**Imports:**
- `api::auth::AUTH_ROUTER`
- `api::cleanup::start_cleanup_scheduler`
- `api::games::GAMES_ROUTER`
- `api::users::{USERS_PROTECTED_ROUTER, USERS_PUBLIC_ROUTER}`
- `api::{AppState, AuthDb}`
- `axum::extract::{Request, State}`
- `axum::http::StatusCode`
- `axum::http::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_MAX_AGE, AUTHORIZATION, CACHE_CONTROL,
    CONTENT_TYPE, EXPIRES, HeaderName,
}`
- `axum::middleware::Next`
- `axum::response::{IntoResponse, Response}`
- *... and 23 more imports*

**Declarations:**

`fn initialize_logging()`

`async fn main() -> Result<(), Box<dyn std::error::Error>>`

`async fn surreal_jwt(State(state): State<AppState>, request: Request, next: Next) -> Response`

`async fn add_rate_limit_headers(request: Request, next: Next) -> Response`

`struct CompoundKeyExtractor`

**`impl KeyExtractor for CompoundKeyExtractor`**
  `fn extract<T>( &self, request: &axum::http::Request<T>, ) -> Result<Self::Key, tower_governor::GovernorError>`


---

## api/src/storage.rs

**Language:** Rust | **Size:** 6.5 KB | **Lines:** 224

**Imports:**
- `crate::AppError`
- `std::path::{Path, PathBuf}`
- `tokio::fs`
- `tokio::io::AsyncWriteExt`

**Declarations:**

**`impl Default for UploadConstraints`**
  `fn default() -> Self`


**`impl LocalStorage`**
  `pub fn new(base_path: impl AsRef<Path>, public_prefix: impl Into<String>) -> Self`

  `pub async fn init(&self) -> Result<(), AppError>`


**`impl StorageBackend for LocalStorage`**
  `async fn save(&self, path: &str, data: &[u8]) -> Result<String, AppError>`

  `async fn delete(&self, path: &str) -> Result<(), AppError>`

  `async fn exists(&self, path: &str) -> Result<bool, AppError>`

  `fn public_url(&self, path: &str) -> String`


`fn validate_image_format(data: &[u8], expected_ext: &str) -> Result<(), AppError>`

`mod tests`

---

## api/src/tributes.rs

**Language:** Rust | **Size:** 10.0 KB | **Lines:** 279

**Imports:**
- `crate::games::game_tributes`
- `crate::storage::{UploadConstraints, validate_upload}`
- `crate::{AppError, AppState, AuthDb}`
- `axum::extract::{Extension, Multipart, Path, State}`
- `axum::http::StatusCode`
- `axum::routing::{get, post}`
- `axum::{Json, Router}`
- `game::items::Item`
- `game::messages::GameMessage`
- `game::tributes::Tribute`
- *... and 7 more imports*

**Declarations:**

---

## api/src/users.rs

**Language:** Rust | **Size:** 9.5 KB | **Lines:** 244

**Imports:**
- `crate::auth::{JWT_SECRET, RefreshToken, TokenResponse, store_refresh_token}`
- `crate::cookies::{set_refresh_cookie, set_session_cookie}`
- `crate::{AppError, AppState, AuthDb}`
- `axum::extract::{Extension, State}`
- `axum::http::StatusCode`
- `axum::response::{IntoResponse, Response}`
- `axum::routing::{get, post}`
- `axum::{Json, Router}`
- `jsonwebtoken::{Algorithm, DecodingKey, Validation, decode}`
- `serde::{Deserialize, Serialize}`
- *... and 5 more imports*

**Declarations:**

`struct JwtClaims`
> Fields: `id: String`, `sub: Option<String>`

`fn extract_user_id_from_jwt(jwt: &str) -> Result<Thing, AppError>`

`async fn create_token_pair( db: &surrealdb::Surreal<surrealdb::engine::any::Any>, jwt: String, user_id: Thing, username: String, ) -> Result<TokenResponse, AppError>`

`async fn session(Extension(AuthDb(db)): Extension<AuthDb>) -> Result<Json<UserSession>, AppError>`

`async fn user_create( state: State<AppState>, Json(payload): Json<RegistrationUser>, ) -> Result<Response, AppError>`

`async fn user_authenticate( state: State<AppState>, Json(payload): Json<RegistrationUser>, ) -> Result<Response, AppError>`

`fn token_response(pair: TokenResponse) -> Response`

---

## api/src/websocket.rs

**Language:** Rust | **Size:** 6.6 KB | **Lines:** 199

**Imports:**
- `axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
}`
- `futures::{sink::SinkExt, stream::StreamExt}`
- `shared::WebSocketMessage`
- `shared::messages::{GameMessage, MessagePayload, MessageSource, Phase, TributeRef}`
- `std::sync::Arc`
- `tokio::sync::broadcast`
- `tracing::{debug, error, info, warn}`
- `crate::AppState`

**Declarations:**

**`impl GameBroadcaster`**
  `pub fn new(capacity: usize) -> Self`

  `pub fn broadcast(&self, msg: WebSocketMessage)`

  `pub fn subscribe(&self) -> broadcast::Receiver<WebSocketMessage>`


**`impl Default for GameBroadcaster`**
  `fn default() -> Self`


`async fn handle_socket(socket: WebSocket, broadcaster: Arc<GameBroadcaster>)`

---

## api/tests/IMPLEMENTATION_SUMMARY.md

**Language:** Markdown | **Size:** 5.3 KB | **Lines:** 173

**Declarations:**

---

## api/tests/README.md

**Language:** Markdown | **Size:** 4.3 KB | **Lines:** 180

**Declarations:**

---

## api/tests/WEBSOCKET_TESTING.md

**Language:** Markdown | **Size:** 2.7 KB | **Lines:** 84

**Declarations:**

---

## api/tests/auth_tests.rs

**Language:** Rust | **Size:** 7.6 KB | **Lines:** 275

**Imports:**
- `axum_test::TestServer`
- `common::{TestDb, TestUser, create_test_router}`
- `serde_json::json`

**Declarations:**

`mod common`

`async fn test_user_registration()`

`async fn test_user_authentication()`

`async fn test_authentication_wrong_password()`

`async fn test_token_refresh()`

`async fn test_logout()`

`async fn test_duplicate_username()`

`async fn test_session_endpoint()`

---

## api/tests/common/mod.rs

**Language:** Rust | **Size:** 11.4 KB | **Lines:** 319

**Imports:**
- `api::AppState`
- `axum::Router`
- `std::sync::Arc`
- `surrealdb::Surreal`
- `surrealdb::engine::any::Any`
- `surrealdb::opt::Config`
- `surrealdb::opt::auth::Root`
- `surrealdb_migrations::MigrationRunner`

**Declarations:**

**`impl TestDb`**
  `pub async fn new() -> Self`

  `pub fn app_state(&self) -> AppState`

  `pub async fn cleanup(&self)`


`fn build_isolated_migration_root() -> std::path::PathBuf`

`fn source_migration_root() -> std::path::PathBuf`

`static MIGRATION_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(())`

`fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()>`

`async fn surreal_jwt( axum::extract::State(state): axum::extract::State<AppState>, request: axum::extract::Request, next: axum::middleware::Next, ) -> axum::response::Response`

**`impl TestUser`**
  `pub fn new(username: &str) -> Self`

  `pub fn with_tokens(mut self, access_token: String, refresh_token: String) -> Self`

  `pub fn auth_header(&self) -> String`


---

## api/tests/game_customization_test.rs

**Language:** Rust | **Size:** 3.8 KB | **Lines:** 117

**Imports:**
- `shared::{EventFrequency, ItemQuantity}`

**Declarations:**

`fn test_item_quantity_base_counts()`

`fn test_item_quantity_default()`

`fn test_event_frequency_probabilities()`

`fn test_event_frequency_default()`

`fn test_item_quantity_serde()`

`fn test_event_frequency_serde()`

`fn test_all_item_quantities_serde()`

`fn test_all_event_frequencies_serde()`

`fn test_event_frequency_ordering()`

`fn test_item_quantity_ordering()`

---

## api/tests/games_tests.rs

**Language:** Rust | **Size:** 14.1 KB | **Lines:** 484

**Imports:**
- `axum_test::TestServer`
- `common::{TestDb, TestUser, create_test_router}`
- `serde_json::json`

**Declarations:**

`mod common`

`async fn create_authenticated_user(server: &TestServer, username: &str) -> TestUser`

`async fn test_create_game()`

`async fn test_list_games()`

`async fn test_get_game()`

`async fn test_update_game()`

`async fn test_delete_game()`

`async fn test_game_display()`

`async fn test_game_areas()`

`async fn test_publish_game()`

`async fn test_unpublish_game()`

`async fn test_unauthorized_game_access()`

`async fn timeline_summary_includes_current_period_even_when_empty()`

`async fn timeline_summary_returns_404_for_missing_game()`

---

## api/tests/simulation_tests.rs

**Language:** Rust | **Size:** 11.8 KB | **Lines:** 373

**Imports:**
- `axum_test::TestServer`
- `common::{TestDb, TestUser, create_test_router}`
- `serde_json::json`

**Declarations:**

`mod common`

`async fn create_authenticated_user(server: &TestServer, username: &str) -> TestUser`

`async fn create_game_with_tributes( server: &TestServer, user: &TestUser, num_tributes: usize, ) -> String`

`async fn test_advance_game()`

`async fn test_game_status_transitions()`

`async fn test_game_day_logs()`

`async fn test_tribute_day_logs()`

`async fn test_multiple_game_cycles()`

`async fn test_game_finishes_with_winner()`

`async fn test_advance_finished_game()`

`async fn test_game_state_persistence()`

---

## api/tests/tributes_tests.rs

**Language:** Rust | **Size:** 10.4 KB | **Lines:** 311

**Imports:**
- `axum_test::TestServer`
- `common::{TestDb, TestUser, create_test_router}`
- `serde_json::json`

**Declarations:**

`mod common`

`async fn create_authenticated_user(server: &TestServer, username: &str) -> TestUser`

`async fn create_test_game(server: &TestServer, user: &TestUser) -> String`

`async fn fetch_tributes( server: &TestServer, user: &TestUser, game_id: &str, ) -> Vec<serde_json::Value>`

`async fn first_tribute_id(server: &TestServer, user: &TestUser, game_id: &str) -> String`

`async fn test_game_auto_spawns_tributes()`

`async fn test_get_tribute()`

`async fn test_update_tribute()`

`async fn test_delete_tribute()`

`async fn test_auto_spawn_district_coverage()`

`async fn test_tribute_log()`

`async fn test_tribute_items()`

`async fn test_update_tribute_validation()`

---

## api/tests/websocket_tests.rs

**Language:** Rust | **Size:** 5.4 KB | **Lines:** 189

**Imports:**
- `common::TestDb`

**Declarations:**

`mod common`

`fn sample_message( payload: shared::messages::MessagePayload, content: &str, ) -> shared::messages::GameMessage`

`async fn test_game_broadcaster_basic()`

`async fn test_game_broadcaster_multi_subscriber()`

`async fn test_broadcast_helper_functions()`

`async fn test_database_setup()`

---

## codemap.md

**Language:** Markdown | **Size:** 21.0 KB | **Lines:** 436

**Declarations:**

---

## docker-compose.yaml

**Language:** YAML | **Size:** 1.7 KB | **Lines:** 59

**Declarations:**

---

## docs/superpowers/plans/2026-04-17-event-severity-integration.md

**Language:** Markdown | **Size:** 37.3 KB | **Lines:** 1241

**Declarations:**

---

## docs/superpowers/plans/2026-04-17-terrain-biome-system.md

**Language:** Markdown | **Size:** 22.7 KB | **Lines:** 719

**Declarations:**

---

## docs/superpowers/plans/2026-04-18-unify-event-systems-33r.md

**Language:** Markdown | **Size:** 33.7 KB | **Lines:** 977

**Declarations:**

---

## docs/superpowers/plans/2026-04-25-tribute-alliances-implementation.md

**Language:** Markdown | **Size:** 54.9 KB | **Lines:** 1490

**Declarations:**

---

## docs/superpowers/plans/2026-04-26-combat-wire-redesign.md

**Language:** Markdown | **Size:** 44.6 KB | **Lines:** 1186

**Declarations:**

---

## docs/superpowers/plans/2026-04-26-game-timeline-pr1-backend.md

**Language:** Markdown | **Size:** 49.5 KB | **Lines:** 1323

**Declarations:**

---

## docs/superpowers/plans/2026-04-26-game-timeline-pr2-frontend.md

**Language:** Markdown | **Size:** 43.4 KB | **Lines:** 1323

**Declarations:**

---

## docs/superpowers/plans/2026-05-03-break-mid-swing-penalty.md

**Language:** Markdown | **Size:** 35.0 KB | **Lines:** 928

**Declarations:**

---

## docs/superpowers/plans/2026-05-03-gamemaker-event-system-pr1-backend.md

**Language:** Markdown | **Size:** 89.6 KB | **Lines:** 2668

**Declarations:**

---

## docs/superpowers/plans/2026-05-03-gamemaker-event-system-pr2-frontend.md

**Language:** Markdown | **Size:** 36.2 KB | **Lines:** 957

**Declarations:**

---

## docs/superpowers/plans/2026-05-03-shelter-hunger-thirst-pr1-backend.md

**Language:** Markdown | **Size:** 69.9 KB | **Lines:** 2082

**Declarations:**

---

## docs/superpowers/plans/2026-05-03-shelter-hunger-thirst-pr2-frontend.md

**Language:** Markdown | **Size:** 31.7 KB | **Lines:** 848

**Declarations:**

---

## docs/superpowers/plans/2026-05-03-stamina-combat-resource-pr1-backend.md

**Language:** Markdown | **Size:** 67.2 KB | **Lines:** 1564

**Declarations:**

---

## docs/superpowers/plans/2026-05-03-stamina-combat-resource-pr2-frontend.md

**Language:** Markdown | **Size:** 23.7 KB | **Lines:** 539

**Declarations:**

---

## docs/superpowers/plans/2026-05-04-addiction-pr1.md

**Language:** Markdown | **Size:** 71.3 KB | **Lines:** 1742

**Declarations:**

---

## docs/superpowers/plans/2026-05-04-afflictions-pr1.md

**Language:** Markdown | **Size:** 64.3 KB | **Lines:** 1699

**Declarations:**

---

## docs/superpowers/plans/2026-05-04-design-system-v1.md

**Language:** Markdown | **Size:** 39.9 KB | **Lines:** 1163

**Declarations:**

---

## docs/superpowers/plans/2026-05-04-sponsorship-pr1.md

**Language:** Markdown | **Size:** 42.0 KB | **Lines:** 1240

**Declarations:**

---

## docs/superpowers/plans/2026-05-04-trapped-afflictions-pr1.md

**Language:** Markdown | **Size:** 62.1 KB | **Lines:** 1727

**Declarations:**

---

## docs/superpowers/plans/2026-05-04-trauma-pr1.md

**Language:** Markdown | **Size:** 66.9 KB | **Lines:** 1710

**Declarations:**

---

## docs/superpowers/specs/2026-04-17-event-severity-integration.md

**Language:** Markdown | **Size:** 12.2 KB | **Lines:** 304

**Declarations:**

---

## docs/superpowers/specs/2026-04-17-terrain-biome-system-design.md

**Language:** Markdown | **Size:** 43.0 KB | **Lines:** 1291

**Declarations:**

---

## docs/superpowers/specs/2026-04-25-tribute-alliances-design.md

**Language:** Markdown | **Size:** 33.1 KB | **Lines:** 737

**Declarations:**

---

## docs/superpowers/specs/2026-04-26-game-event-enum.md

**Language:** Markdown | **Size:** 11.5 KB | **Lines:** 149

**Declarations:**

---

## docs/superpowers/specs/2026-04-26-game-timeline-redesign.md

**Language:** Markdown | **Size:** 30.4 KB | **Lines:** 534

**Declarations:**

---

## docs/superpowers/specs/2026-05-01-hex-arena-map-design.md

**Language:** Markdown | **Size:** 5.8 KB | **Lines:** 168

**Declarations:**

---

## docs/superpowers/specs/2026-05-02-progressive-display-design.md

**Language:** Markdown | **Size:** 30.5 KB | **Lines:** 445

**Declarations:**

---

## docs/superpowers/specs/2026-05-02-spectator-skin-layout-design.md

**Language:** Markdown | **Size:** 17.0 KB | **Lines:** 260

**Declarations:**

---

## docs/superpowers/specs/2026-05-02-spectator-skin-visuals-design.md

**Language:** Markdown | **Size:** 23.0 KB | **Lines:** 342

**Declarations:**

---

## docs/superpowers/specs/2026-05-02-tribute-emotions-design.md

**Language:** Markdown | **Size:** 21.7 KB | **Lines:** 374

**Declarations:**

---

## docs/superpowers/specs/2026-05-02-weather-system-design.md

**Language:** Markdown | **Size:** 29.6 KB | **Lines:** 499

**Declarations:**

---

## docs/superpowers/specs/2026-05-03-break-mid-swing-design.md

**Language:** Markdown | **Size:** 9.7 KB | **Lines:** 189

**Declarations:**

---

## docs/superpowers/specs/2026-05-03-fixations-design.md

**Language:** Markdown | **Size:** 10.2 KB | **Lines:** 177

**Declarations:**

---

## docs/superpowers/specs/2026-05-03-four-phase-day-design.md

**Language:** Markdown | **Size:** 17.6 KB | **Lines:** 289

**Declarations:**

---

## docs/superpowers/specs/2026-05-03-gamemaker-event-system-design.md

**Language:** Markdown | **Size:** 25.9 KB | **Lines:** 554

**Declarations:**

---

## docs/superpowers/specs/2026-05-03-health-conditions-design.md

**Language:** Markdown | **Size:** 20.4 KB | **Lines:** 368

**Declarations:**

---

## docs/superpowers/specs/2026-05-03-phobias-design.md

**Language:** Markdown | **Size:** 24.1 KB | **Lines:** 387

**Declarations:**

---

## docs/superpowers/specs/2026-05-03-shelter-hunger-thirst-design.md

**Language:** Markdown | **Size:** 29.4 KB | **Lines:** 527

**Declarations:**

---

## docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md

**Language:** Markdown | **Size:** 19.3 KB | **Lines:** 325

**Declarations:**

---

## docs/superpowers/specs/2026-05-04-addiction-design.md

**Language:** Markdown | **Size:** 38.6 KB | **Lines:** 527

**Declarations:**

---

## docs/superpowers/specs/2026-05-04-design-system-v1-design.md

**Language:** Markdown | **Size:** 11.0 KB | **Lines:** 254

**Declarations:**

---

## docs/superpowers/specs/2026-05-04-sponsorship-design.md

**Language:** Markdown | **Size:** 13.4 KB | **Lines:** 295

**Declarations:**

---

## docs/superpowers/specs/2026-05-04-trapped-afflictions-design.md

**Language:** Markdown | **Size:** 27.3 KB | **Lines:** 668

**Declarations:**

---

## docs/superpowers/specs/2026-05-04-trauma-design.md

**Language:** Markdown | **Size:** 29.5 KB | **Lines:** 430

**Declarations:**

---

## game/Cargo.toml

**Language:** TOML | **Size:** 718 B | **Lines:** 30

**Imports:**
- `chrono`
- `fake`
- `indefinite`
- `once_cell`
- `rand`
- `serde`
- `serde_json`
- `shared`
- `strum`
- `strum_macros`
- *... and 7 more imports*

**Declarations:**

---

## game/benches/game_cycle_bench.rs

**Language:** Rust | **Size:** 1.7 KB | **Lines:** 63

**Imports:**
- `criterion::{Criterion, criterion_group, criterion_main}`
- `game::games::Game`
- `game::tributes::Tribute`
- `game::tributes::statuses::TributeStatus`
- `std::hint::black_box`

**Declarations:**

`fn create_test_game(tribute_count: usize) -> Game`

`fn bench_living_tributes_full(c: &mut Criterion)`

`fn bench_living_tributes_half(c: &mut Criterion)`

`fn bench_living_tributes_few(c: &mut Criterion)`

---

## game/codemap.md

**Language:** Markdown | **Size:** 368 B | **Lines:** 19

**Declarations:**

---

## game/src/areas/codemap.md

**Language:** Markdown | **Size:** 6.3 KB | **Lines:** 133

**Declarations:**

---

## game/src/areas/events.rs

**Language:** Rust | **Size:** 25.2 KB | **Lines:** 642

**Imports:**
- `crate::terrain::BaseTerrain`
- `rand::RngExt`
- `rand::prelude::*`
- `serde::{Deserialize, Serialize}`
- `std::fmt::Display`
- `std::str::FromStr`
- `strum::{EnumIter, IntoEnumIterator}`

**Declarations:**

**`impl FromStr for AreaEvent`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


**`impl Display for AreaEvent`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl AreaEvent`**
  `pub fn random(rng: &mut impl Rng) -> AreaEvent`

  `pub fn random_for_terrain(terrain: &BaseTerrain, rng: &mut impl Rng) -> AreaEvent`

  `pub fn severity_in_terrain(&self, terrain: &BaseTerrain) -> EventSeverity`

  `pub fn survival_check( &self, terrain: &BaseTerrain, has_affinity: bool, has_item_bonus: bool, is_desperate: bool, current_health: u32, instant_death_enabled: bool, severity_multiplier: f64, rng: &mut impl Rng, ) -> SurvivalResult`


`mod tests`

---

## game/src/areas/forage.rs

**Language:** Rust | **Size:** 1.3 KB | **Lines:** 39

**Imports:**
- `crate::terrain::types::BaseTerrain`

**Declarations:**

`mod tests`

---

## game/src/areas/hex.rs

**Language:** Rust | **Size:** 9.3 KB | **Lines:** 272

**Imports:**
- `crate::areas::Area`

**Declarations:**

**`impl Axial`**
  `pub const fn new(q: i32, r: i32) -> Self`

  `pub fn neighbors(self) -> [Axial; 6]`

  `pub fn distance(self, other: Axial) -> i32`

  `pub fn to_pixel(self, size: f64) -> (f64, f64)`


**`impl SubAxial`**
  `pub const fn new(q: i32, r: i32) -> Self`

  `pub fn to_pixel(self, sub_size: f64) -> (f64, f64)`


`mod tests`

---

## game/src/areas/mod.rs

**Language:** Rust | **Size:** 12.4 KB | **Lines:** 388

**Imports:**
- `crate::areas::events::AreaEvent`
- `crate::areas::hex::{SUB_SLOTS, SubAxial}`
- `crate::items::OwnsItems`
- `crate::items::{Item, ItemError}`
- `crate::terrain::{BaseTerrain, TerrainType}`
- `serde::{Deserialize, Serialize}`
- `std::collections::HashMap`
- `std::fmt::Display`
- `std::str::FromStr`
- `strum_macros::EnumIter`
- *... and 1 more imports*

**Declarations:**

**`impl Serialize for Area`**
  `fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error>`


**`impl<'de> Deserialize<'de> for Area`**
  `fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error>`


**`impl Display for Area`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl PartialEq<&Area> for Area`**
  `fn eq(&self, other: &&Area) -> bool`


**`impl FromStr for Area`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


**`impl Area`**
  `pub fn neighbors(&self) -> Vec<Area>`


`fn default_terrain() -> TerrainType`

**`impl Default for AreaDetails`**
  `fn default() -> Self`


**`impl OwnsItems for AreaDetails`**
  `fn add_item(&mut self, item: Item)`

  `fn has_item(&self, item: &Item) -> bool`

  `fn use_item(&mut self, item: &Item) -> Result<(), ItemError>`

  `fn remove_item(&mut self, item: &Item) -> Result<(), ItemError>`


**`impl AreaDetails`**
  `pub fn new(name: Option<String>, area: Area) -> Self`

  `pub fn new_with_terrain(name: Option<String>, area: Area, terrain: TerrainType) -> Self`

  `pub fn is_open(&self) -> bool`

  `pub fn assign_slot(&mut self, tribute_id: &str) -> SubAxial`

  `pub fn release_slot(&mut self, tribute_id: &str) -> Option<SubAxial>`


`mod tests`

---

## game/src/areas/path.rs

**Language:** Rust | **Size:** 7.5 KB | **Lines:** 209

**Imports:**
- `crate::areas::{Area, AreaDetails}`
- `crate::pathfinding::{Graph, astar}`
- `crate::terrain::Harshness`
- `crate::tributes::actions::Action`
- `crate::tributes::{Tribute, calculate_stamina_cost}`
- `std::collections::HashMap`
- `strum::IntoEnumIterator`

**Declarations:**

**`impl<'a> AreaGraph<'a>`**
  `pub fn new(areas: &'a [AreaDetails], closed: &[Area], tribute: &'a Tribute) -> Self`

  `fn edge_cost(&self, to: Area) -> u32`


**`impl<'a> Graph for AreaGraph<'a>`**
  `fn neighbors(&self, node: Area) -> Vec<(Area, u32)>`

  `fn heuristic(&self, _from: Area, _to: Area) -> u32`


`mod tests`

---

## game/src/areas/shelter.rs

**Language:** Rust | **Size:** 3.1 KB | **Lines:** 90

**Imports:**
- `crate::areas::weather::Weather`
- `crate::terrain::types::BaseTerrain`

**Declarations:**

`mod tests`

---

## game/src/areas/water.rs

**Language:** Rust | **Size:** 3.4 KB | **Lines:** 91

**Imports:**
- `crate::areas::weather::Weather`
- `crate::terrain::types::BaseTerrain`

**Declarations:**

`mod tests`

---

## game/src/areas/weather.rs

**Language:** Rust | **Size:** 828 B | **Lines:** 34

**Imports:**
- `serde::{Deserialize, Serialize}`

**Declarations:**

`mod tests`

---

## game/src/codemap.md

**Language:** Markdown | **Size:** 9.3 KB | **Lines:** 176

**Declarations:**

---

## game/src/config.rs

**Language:** Rust | **Size:** 5.2 KB | **Lines:** 157

**Imports:**
- `serde::{Deserialize, Serialize}`

**Declarations:**

**`impl Default for GameConfig`**
  `fn default() -> Self`


`mod tests`

---

## game/src/districts.rs

**Language:** Rust | **Size:** 6.8 KB | **Lines:** 209

**Imports:**
- `crate::terrain::BaseTerrain`
- `rand::Rng`
- `rand::RngExt`

**Declarations:**

`mod tests`

---

## game/src/events.rs

**Language:** Rust | **Size:** 51.7 KB | **Lines:** 1624

**Imports:**
- `std::fmt::{Display, Formatter}`
- `serde::{Deserialize, Serialize}`
- `uuid::Uuid`
- `crate::items::Item`
- `crate::threats::animals::Animal`
- `indefinite::{indefinite, indefinite_capitalized}`

**Declarations:**

**`impl Display for GameEvent`**
  `fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result`


`mod tests`

---

## game/src/games.rs

**Language:** Rust | **Size:** 133.5 KB | **Lines:** 3357

**Imports:**
- `crate::areas::events::AreaEvent`
- `crate::areas::{Area, AreaDetails}`
- `crate::items::Item`
- `crate::items::OwnsItems`
- `crate::tributes::actions::Action`
- `crate::tributes::events::TributeEvent`
- `crate::tributes::statuses::TributeStatus`
- `crate::tributes::{
    ActionSuggestion, EncounterContext, EnvironmentContext, Tribute, calculate_stamina_cost,
}`
- `rand::RngExt`
- `rand::prelude::*`
- *... and 8 more imports*

**Declarations:**

`const SLEEP_STAMINA_PER_PHASE: u32 = 25`

`const SLEEP_HP_PER_PHASE: u32 = 5`

`const SLEEP_HP_CAP: u32 = 100`

`fn area_event_to_kind(ev: &AreaEvent) -> shared::messages::AreaEventKind`

**`impl Display for GameError`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl std::error::Error for GameError`**

**`impl From<String> for GameError`**
  `fn from(error: String) -> Self`


`const LOW_TRIBUTE_THRESHOLD: u32 = 8`

`const FEAST_WEAPON_COUNT: u32 = 2`

`const FEAST_SHIELD_COUNT: u32 = 2`

`const FEAST_CONSUMABLE_COUNT: u32 = 4`

`const DAY_EVENT_FREQUENCY: f64 = 1.0 / 4.0`

`const NIGHT_EVENT_FREQUENCY: f64 = 1.0 / 8.0`

**`impl TickCounter`**
  `pub fn reset(&mut self)`

  `pub fn next(&mut self) -> u32`

  `pub fn boundary(&self) -> u32`


`fn default_phase() -> crate::messages::Phase`

**`impl PartialEq for Game`**
  `fn eq(&self, other: &Self) -> bool`


**`impl Default for Game`**
  `fn default() -> Game`


**`impl Display for Game`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


`type CollectedEvent = ( String, String, String, Option<crate::messages::MessagePayload>, Option<crate::events::GameEvent>, )`

`struct CycleContext`
> Fields: `is_day: bool`, `phase: crate::messages::Phase`, `current_day: u32`, `action_suggestion: Option<ActionSuggestion>`, `area_details_map: HashMap<Area, usize>`, `tributes_by_area: HashMap<Area, Vec<Tribute>>`, `enemy_density: HashMap<Area, u32>`, `combat_tuning_snapshot: crate::tributes::combat_tuning::CombatTuning`, `all_areas_snapshot: Vec<AreaDetails>`, `closed_areas: Vec<Area>`, `living_tributes_count: usize`

**`impl Game`**
  `pub fn new(name: &str) -> Self`

  `pub fn end(&mut self)`

  `pub fn start(&mut self) -> Result<(), GameError>`

  `pub fn living_tributes(&self) -> Vec<Tribute>`

  `pub fn living_tributes_count(&self) -> usize`

  `fn recently_dead_tributes(&self) -> Vec<Tribute>`

  `pub fn winner(&self) -> Option<Tribute>`

  `fn random_area(&mut self) -> Option<&mut AreaDetails>`

  `fn random_open_area(&self) -> Option<AreaDetails>`

  `fn open_areas(&self) -> Vec<AreaDetails>`

  `fn closed_areas(&self) -> Vec<AreaDetails>`

  `fn fallback_payload( source: &crate::messages::MessageSource, ) -> crate::messages::MessagePayload`

  `fn push_message( &mut self, source: crate::messages::MessageSource, subject: String, content: String, payload: crate::messages::MessagePayload, tick: u32, )`

  `pub fn log( &mut self, source: crate::messages::MessageSource, subject: String, content: String, )`

  `pub fn log_output<D: std::fmt::Display>( &mut self, source: crate::messages::MessageSource, subject: String, output: D, )`

  `pub fn log_output_kind<D: std::fmt::Display>( &mut self, source: crate::messages::MessageSource, subject: String, output: D, _kind: crate::messages::MessageKind, )`

  `pub fn log_event( &mut self, source: crate::messages::MessageSource, subject: String, event: crate::events::GameEvent, )`

  `pub fn log_event_kind( &mut self, source: crate::messages::MessageSource, subject: String, event: crate::events::GameEvent, _kind: crate::messages::MessageKind, )`

  `fn check_for_winner(&mut self) -> Result<(), GameError>`

  `fn prepare_cycle(&mut self, phase: crate::messages::Phase) -> Result<(), GameError>`

  `fn is_new_day_boundary(&self, phase: crate::messages::Phase) -> bool`

  `fn announce_cycle_start(&mut self, phase: crate::messages::Phase) -> Result<(), GameError>`

  `fn announce_cycle_end(&mut self, phase: crate::messages::Phase) -> Result<(), GameError>`

  `pub fn process_event_for_area( &mut self, area: &Area, event: &AreaEvent, rng: &mut impl Rng, ) -> Result<(), GameError>`

  `pub fn run_phase(&mut self, phase: crate::messages::Phase) -> Result<(), GameError>`

  `pub fn run_full_day(&mut self) -> Result<(), GameError>`

  `fn announce_area_events(&mut self) -> Result<(), GameError>`

  `fn ensure_open_area(&mut self)`

  `fn trigger_cycle_events( &mut self, phase: crate::messages::Phase, rng: &mut SmallRng, ) -> Result<(), GameError>`

  `fn constrain_areas(&mut self, rng: &mut SmallRng) -> Result<(), GameError>`

  `fn build_cycle_context( &self, phase: crate::messages::Phase, closed_areas: Vec<Area>, living_tributes: Vec<Tribute>, living_tributes_count: usize, ) -> CycleContext`

  `fn execute_cycle(&mut self, ctx: CycleContext, rng: &mut SmallRng) -> Result<(), GameError>`

  `fn flush_tribute_events(&mut self, collected_events: Vec<CollectedEvent>)`

  `fn run_tribute_cycle( &mut self, phase: crate::messages::Phase, rng: &mut SmallRng, closed_areas: Vec<Area>, living_tributes: Vec<Tribute>, living_tributes_count: usize, ) -> Result<(), GameError>`

  `fn do_a_cycle(&mut self, phase: crate::messages::Phase) -> Result<(), GameError>`

  `fn clean_up_recent_deaths(&mut self)`

  `fn get_area_details_mut(&mut self, area: Area) -> Option<&mut AreaDetails>`

  `pub fn process_alliance_events(&mut self, rng: &mut impl Rng)`


`mod tests`

---

## game/src/items/codemap.md

**Language:** Markdown | **Size:** 8.1 KB | **Lines:** 157

**Declarations:**

---

## game/src/items/mod.rs

**Language:** Rust | **Size:** 30.7 KB | **Lines:** 1005

**Imports:**
- `crate::items::name_generator::{generate_shield_name, generate_weapon_name}`
- `crate::terrain::BaseTerrain`
- `rand::RngExt`
- `rand::prelude::*`
- `serde::{Deserialize, Serialize}`
- `std::fmt::Display`
- `std::str::FromStr`
- `strum::{EnumIter, IntoEnumIterator}`
- `thiserror::Error`
- `uuid::Uuid`

**Declarations:**

`mod name_generator`

**`impl ItemRarity`**
  `pub fn random() -> ItemRarity`

  `pub fn effect_range(&self) -> (i32, i32)`

  `pub fn weapon_durability_range(&self) -> (u32, u32)`

  `pub fn shield_durability_range(&self) -> (u32, u32)`


**`impl Display for ItemRarity`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


`fn default_rarity() -> ItemRarity`

**`impl Display for Item`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl Default for Item`**
  `fn default() -> Self`


**`impl Item`**
  `pub fn new( name: &str, item_type: ItemType, rarity: ItemRarity, max_durability: u32, attribute: Attribute, effect: i32, ) -> Item`

  `pub fn wear(&mut self, wear_amount: u32) -> WearOutcome`

  `pub fn new_random(name: Option<&str>) -> Item`

  `pub fn new_random_with_terrain(terrain: BaseTerrain, name: Option<&str>) -> Item`

  `pub fn new_weapon(name: &str) -> Item`

  `pub fn new_random_weapon() -> Item`

  `pub fn new_consumable(name: &str) -> Item`

  `pub fn new_random_consumable() -> Item`

  `pub fn new_shield(name: &str) -> Item`

  `pub fn new_random_shield() -> Item`

  `pub fn new_food(name: Option<&str>, value: u8) -> Item`

  `pub fn new_water(name: Option<&str>, value: u8) -> Item`

  `pub fn is_weapon(&self) -> bool`

  `pub fn is_defensive(&self) -> bool`

  `pub fn is_consumable(&self) -> bool`


**`impl ItemType`**
  `pub fn random() -> ItemType`

  `pub fn is_food(&self) -> bool`

  `pub fn is_water(&self) -> bool`

  `pub fn food_value(&self) -> Option<u8>`

  `pub fn water_value(&self) -> Option<u8>`


**`impl Display for ItemType`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl FromStr for ItemType`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


**`impl Attribute`**
  `pub fn random() -> Attribute`


**`impl ConsumableAttribute for Attribute`**
  `fn consumable_name(&self) -> String`


**`impl Display for Attribute`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl FromStr for Attribute`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


`mod tests`

---

## game/src/items/name_generator.rs

**Language:** Rust | **Size:** 1.6 KB | **Lines:** 56

**Imports:**
- `rand::prelude::*`

**Declarations:**

`const SHIELD_ADJECTIVES: &[&str] = &[ "iron", "wooden", "brass", "bronze", "glass", "steel", "stone", ]`

`const WEAPON_NOUNS: &[&str] = &[ "sword", "spear", "dagger", "knife", "net", "trident", "bow", "mace", "axe", ]`

`const WEAPON_ADJECTIVES: &[&str] = &[ "sharp", "heavy", "long", "short", "glass", "iron", "wooden", "brass", "bronze", "glass", "steel", "stone", ]`

`mod tests`

---

## game/src/lib.rs

**Language:** Rust | **Size:** 343 B | **Lines:** 17

**Imports:**
- `pub use terrain::{BaseTerrain, TerrainDescriptor, TerrainType}`

**Declarations:**

`mod witty_phrase_generator`

---

## game/src/messages.rs

**Language:** Rust | **Size:** 11.6 KB | **Lines:** 351

**Imports:**
- `pub use shared::messages::{
    AreaEventKind, AreaRef, CombatEngagement, CombatOutcome, GameMessage, ItemRef, MessageKind,
    MessagePayload, MessageSource, ParsePhaseError, Phase, TributeRef,
}`
- `crate::terrain::{BaseTerrain, Harshness, Visibility}`

**Declarations:**

**`impl TaggedEvent`**
  `pub fn new(content: impl Into<String>, payload: MessagePayload) -> Self`


`fn terrain_name(terrain: BaseTerrain) -> &'static str`

`mod tests`

---

## game/src/output.rs

**Language:** Rust | **Size:** 18.7 KB | **Lines:** 468

**Imports:**
- `crate::items::Item`
- `crate::threats::animals::Animal`
- `indefinite::indefinite`
- `indefinite::indefinite_capitalized`
- `std::fmt::{Display, Formatter}`
- `std::str::FromStr`

**Declarations:**

**`impl<'a> Display for GameOutput<'a>`**
  `fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result`


`mod tests`

---

## game/src/pathfinding.rs

**Language:** Rust | **Size:** 5.3 KB | **Lines:** 178

**Imports:**
- `std::cmp::Ordering`
- `std::collections::{BinaryHeap, HashMap}`
- `std::hash::Hash`

**Declarations:**

`struct Frontier<N, C>`
> Fields: `f: C`, `node: N`

**`impl<N, C: Ord> Ord for Frontier<N, C>`**
  `fn cmp(&self, other: &Self) -> Ordering`


**`impl<N, C: Ord> PartialOrd for Frontier<N, C>`**
  `fn partial_cmp(&self, other: &Self) -> Option<Ordering>`


**`impl<N, C: Eq> Eq for Frontier<N, C>`**

**`impl<N, C: Eq> PartialEq for Frontier<N, C>`**
  `fn eq(&self, other: &Self) -> bool`


`fn reconstruct<N: Copy + Eq + Hash, C: Copy>( came_from: &HashMap<N, N>, end: N, cost: C, ) -> (Vec<N>, C)`

`mod tests`

---

## game/src/phases/environment.rs

**Language:** Rust | **Size:** 12.9 KB | **Lines:** 385

**Imports:**
- `crate::areas::weather::Weather`
- `crate::messages::Phase`
- `crate::terrain::types::BaseTerrain`
- `crate::tributes::statuses::TributeStatus`
- `rand::{Rng, RngExt}`
- `serde::{Deserialize, Serialize}`

**Declarations:**

**`impl AfflictionDraft`**
  `pub fn new(status: TributeStatus) -> Self`


`fn darken(l: LightLevel) -> LightLevel`

`fn candidate_probabilities( phase: Phase, biome: BaseTerrain, weather: Weather, ) -> Vec<(TributeStatus, f32)>`

`mod tests`

---

## game/src/phases/mod.rs

**Language:** Rust | **Size:** 348 B | **Lines:** 8

**Declarations:**

---

## game/src/terrain/assignment.rs

**Language:** Rust | **Size:** 6.4 KB | **Lines:** 192

**Imports:**
- `crate::terrain::config::Harshness`
- `crate::terrain::{BaseTerrain, TerrainDescriptor, TerrainType}`
- `rand::RngExt`
- `rand::prelude::*`
- `strum::IntoEnumIterator`

**Declarations:**

**`impl TerrainType`**
  `pub fn random(rng: &mut impl Rng) -> Self`

  `pub fn random_safe(rng: &mut impl Rng) -> Self`

  `pub fn random_moderate(rng: &mut impl Rng) -> Self`

  `fn compatible_descriptors_for(base: BaseTerrain, rng: &mut impl Rng) -> Vec<TerrainDescriptor>`


`mod tests`

---

## game/src/terrain/config.rs

**Language:** Rust | **Size:** 4.7 KB | **Lines:** 149

**Imports:**
- `serde::{Deserialize, Serialize}`
- `crate::terrain::BaseTerrain`

**Declarations:**

**`impl BaseTerrain`**
  `pub const fn movement_cost(&self) -> f32`

  `pub const fn visibility(&self) -> Visibility`

  `pub const fn harshness(&self) -> Harshness`

  `pub const fn item_spawn_modifier(&self) -> f32`

  `pub const fn item_weights(&self) -> ItemWeights`


---

## game/src/terrain/descriptors.rs

**Language:** Rust | **Size:** 78 B | **Lines:** 2

---

## game/src/terrain/mod.rs

**Language:** Rust | **Size:** 237 B | **Lines:** 8

**Imports:**
- `pub use assignment::enforce_balance_constraint`
- `pub use config::{Harshness, ItemWeights, Visibility}`
- `pub use types::{BaseTerrain, TerrainDescriptor, TerrainType}`

**Declarations:**

---

## game/src/terrain/types.rs

**Language:** Rust | **Size:** 3.4 KB | **Lines:** 121

**Imports:**
- `serde::{Deserialize, Serialize}`
- `strum::EnumIter`

**Declarations:**

**`impl TerrainType`**
  `pub fn new(base: BaseTerrain, descriptors: Vec<TerrainDescriptor>) -> Result<Self, String>`

  `fn is_compatible(base: &BaseTerrain, descriptor: &TerrainDescriptor) -> bool`


**`impl BaseTerrain`**
  `pub fn descriptive_name(&self) -> &'static str`


**`impl TerrainDescriptor`**
  `pub fn as_adjective(&self) -> &'static str`


---

## game/src/threats/animals.rs

**Language:** Rust | **Size:** 5.6 KB | **Lines:** 192

**Imports:**
- `rand::prelude::*`
- `serde::{Deserialize, Serialize}`
- `std::fmt::Display`
- `std::str::FromStr`
- `strum::{EnumIter, IntoEnumIterator}`

**Declarations:**

**`impl Animal`**
  `pub fn plural(&self) -> String`

  `pub fn random() -> Animal`

  `pub fn damage(&self) -> u32`


**`impl FromStr for Animal`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


**`impl Display for Animal`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


`mod tests`

---

## game/src/threats/codemap.md

**Language:** Markdown | **Size:** 5.7 KB | **Lines:** 135

**Declarations:**

---

## game/src/threats/mod.rs

**Language:** Rust | **Size:** 24 B | **Lines:** 1

**Declarations:**

`pub(crate) mod animals`

---

## game/src/tributes/actions.rs

**Language:** Rust | **Size:** 7.0 KB | **Lines:** 212

**Imports:**
- `crate::areas::Area`
- `crate::items::Item`
- `crate::tributes::Tribute`
- `serde::{Deserialize, Serialize}`
- `std::fmt::Display`
- `std::str::FromStr`

**Declarations:**

**`impl TributeAction`**
  `pub fn new(action: Action, target: Option<Tribute>) -> TributeAction`


**`impl Display for Action`**
  `fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result`


**`impl FromStr for Action`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


`mod tests`

`mod survival_action_tests`

---

## game/src/tributes/alliances.rs

**Language:** Rust | **Size:** 15.6 KB | **Lines:** 490

**Imports:**
- `rand::RngExt`
- `uuid::Uuid`
- `crate::tributes::traits::{REFUSERS, Trait, geometric_mean_affinity}`

**Declarations:**

**`impl DecidingFactor`**
  `pub fn label(&self) -> &'static str`


`mod tests`

---

## game/src/tributes/brains.rs

**Language:** Rust | **Size:** 69.1 KB | **Lines:** 1935

**Imports:**
- `crate::areas::{Area, AreaDetails}`
- `crate::terrain::{BaseTerrain, Harshness, TerrainType, Visibility}`
- `crate::tributes::Tribute`
- `crate::tributes::actions::Action`
- `crate::tributes::alliances::MAX_ALLIES`
- `crate::tributes::traits::{REFUSERS, ThresholdDelta, Trait, geometric_mean_affinity}`
- `rand::Rng`
- `rand::RngExt`
- `serde::{Deserialize, Serialize}`
- `std::collections::HashMap`

**Declarations:**

`const LOW_ENEMY_LIMIT: u32 = 6`

`const SLEEP_DOMINANT_THRESHOLD: u32 = 12`

`const SLEEP_WANT_THRESHOLD: u32 = 6`

`const SLEEP_EXHAUSTED_PCT: u32 = 25`

`const CROWD_PENALTY_PER_ENEMY: i32 = 8`

`const CROWD_PENALTY_MAX: i32 = 32`

**`impl PersonalityThresholds`**
  `pub fn from_traits(traits: &[Trait], rng: &mut impl Rng) -> Self`


`fn deserialize_optional_enum_lenient<'de, D, E>(deserializer: D) -> Result<Option<E>, D::Error> where D: serde::Deserializer<'de>, E: serde::de::DeserializeOwned,`

**`impl Default for Brain`**
  `fn default() -> Self`


**`impl Brain`**
  `pub fn from_traits(traits: &[Trait], rng: &mut impl Rng) -> Self`

  `pub fn check_psychotic_break(&mut self, current_sanity: u32, rng: &mut impl Rng)`

  `pub fn check_recovery(&mut self, current_sanity: u32)`


**`impl Brain`**
  `pub fn set_preferred_action(&mut self, action: Action, percentage: f64)`

  `pub fn clear_preferred_action(&mut self)`

  `pub fn act( &self, tribute: &Tribute, nearby_tributes: u32, available_destinations: &[crate::areas::DestinationInfo], all_areas: &[AreaDetails], closed_areas: &[Area], enemy_density: &HashMap<Area, u32>, rng: &mut impl Rng, ) -> Action`

  `pub fn choose_destination( &self, areas: &[AreaDetails], tribute: &Tribute, enemy_density: &HashMap<Area, u32>, ) -> Option<Area>`

  `pub fn decide_action_with_terrain( &self, tribute: &Tribute, nearby_tributes: u32, terrain: TerrainType, rng: &mut impl Rng, ) -> Action`

  `pub fn should_sleep( &self, tribute: &Tribute, nearby_tributes: u32, phase: shared::messages::Phase, _rng: &mut impl Rng, ) -> Option<Action>`

  `fn run_pre_decision_overrides( &self, tribute: &Tribute, nearby_tributes: u32, terrain: Option<BaseTerrain>, rng: &mut impl Rng, ) -> Option<Action>`

  `fn decide_action_few_enemies_with_terrain( &self, tribute: &Tribute, is_concealed: bool, ) -> Action`

  `fn decide_action_few_enemies_low_health_with_terrain( &self, tribute: &Tribute, is_concealed: bool, ) -> Action`

  `fn decide_action_many_enemies_with_terrain( &self, tribute: &Tribute, is_concealed: bool, ) -> Action`

  `fn wants_to_propose_alliance( &self, tribute: &Tribute, nearby_tributes: u32, rng: &mut impl Rng, ) -> bool`

  `fn decide_action_no_enemies(&self, tribute: &Tribute) -> Action`

  `fn decide_action_few_enemies_low_health(&self, tribute: &Tribute) -> Action`

  `fn decide_action_few_enemies(&self, tribute: &Tribute) -> Action`

  `fn decide_action_many_enemies(&self, tribute: &Tribute) -> Action`


`mod tests`

`mod survival_override_tests`

---

## game/src/tributes/codemap.md

**Language:** Markdown | **Size:** 14.3 KB | **Lines:** 370

**Declarations:**

---

## game/src/tributes/combat.rs

**Language:** Rust | **Size:** 75.2 KB | **Lines:** 2041

**Imports:**
- `crate::items::{Item, OwnsItems}`
- `crate::messages::{CombatEngagement, CombatOutcome, MessagePayload, TaggedEvent, TributeRef}`
- `crate::output::GameOutput`
- `crate::tributes::Tribute`
- `crate::tributes::actions::{AttackOutcome, AttackResult}`
- `rand::RngExt`
- `rand::prelude::*`
- `shared::combat_beat::{CombatBeat, StressReport, SwingOutcome}`
- `shared::messages::ItemRef`
- `std::cmp::Ordering`

**Declarations:**

`fn tref(t: &Tribute) -> TributeRef`

`fn iref(i: &Item) -> ItemRef`

`fn new_beat( attacker: &Tribute, target: &Tribute, outcome: SwingOutcome, tuning: &crate::tributes::combat_tuning::CombatTuning, ) -> CombatBeat`

**`impl Tribute`**
  `pub(crate) fn attacks( &mut self, target: &mut Tribute, rng: &mut impl Rng, events: &mut Vec<TaggedEvent>, phase: shared::messages::Phase, tuning: &crate::tributes::combat_tuning::CombatTuning, ) -> AttackOutcome`

  `pub(crate) fn apply_violence_stress( &mut self, events: &mut Vec<TaggedEvent>, tuning: &crate::tributes::combat_tuning::CombatTuning, ) -> u32`


`fn calculate_violence_stress( kills: u32, wins: u32, current_sanity: u32, tuning: &crate::tributes::combat_tuning::CombatTuning, ) -> u32`

`pub(crate) fn apply_combat_results( winner: &mut Tribute, loser: &mut Tribute, damage_to_loser: u32, log_event: GameOutput, events: &mut Vec<TaggedEvent>, tuning: &crate::tributes::combat_tuning::CombatTuning, ) -> u32`

`mod tests`

---

## game/src/tributes/combat_beat.rs

**Language:** Rust | **Size:** 19.9 KB | **Lines:** 568

**Imports:**
- `pub use shared::combat_beat::{
    CombatBeat, StressReport, SwingOutcome, WearOutcomeReport, WearReport,
}`
- `crate::output::GameOutput`

**Declarations:**

**`impl CombatBeatExt for CombatBeat`**
  `fn to_log_lines(&self) -> Vec<String>`


`mod tests`

---

## game/src/tributes/combat_tuning.rs

**Language:** Rust | **Size:** 4.0 KB | **Lines:** 118

**Imports:**
- `serde::{Deserialize, Serialize}`

**Declarations:**

**`impl Default for CombatTuning`**
  `fn default() -> Self`


`mod tests`

---

## game/src/tributes/events.rs

**Language:** Rust | **Size:** 5.5 KB | **Lines:** 159

**Imports:**
- `crate::threats::animals::Animal`
- `rand::RngExt`
- `rand::prelude::SmallRng`
- `rand::prelude::*`
- `serde::{Deserialize, Serialize}`
- `std::fmt::Display`
- `std::str::FromStr`

**Declarations:**

**`impl FromStr for TributeEvent`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


**`impl Display for TributeEvent`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl TributeEvent`**
  `pub fn random() -> TributeEvent`


`mod tests`

---

## game/src/tributes/inventory.rs

**Language:** Rust | **Size:** 8.5 KB | **Lines:** 267

**Imports:**
- `crate::areas::AreaDetails`
- `crate::items::{Attribute, Item, ItemError, OwnsItems}`
- `crate::tributes::Tribute`
- `rand::RngExt`
- `rand::prelude::*`
- `rand::rngs::SmallRng`

**Declarations:**

**`impl OwnsItems for Tribute`**
  `fn add_item(&mut self, item: Item)`

  `fn has_item(&self, item: &Item) -> bool`

  `fn use_item(&mut self, item: &Item) -> Result<(), ItemError>`

  `fn remove_item(&mut self, item: &Item) -> Result<(), ItemError>`


**`impl Tribute`**
  `pub(crate) fn receive_patron_gift(&mut self, mut rng: impl Rng) -> Option<Item>`

  `pub(crate) fn take_nearby_item(&mut self, area_details: &mut AreaDetails) -> Option<Item>`

  `pub(crate) fn try_use_consumable(&mut self, chosen_item: &Item) -> Result<(), ItemError>`

  `pub(crate) fn available_items(&self) -> Vec<Item>`

  `pub fn consumables(&self) -> Vec<Item>`

  `pub(crate) fn equipped_weapon_mut(&mut self) -> Option<&mut Item>`

  `pub(crate) fn equipped_shield_mut(&mut self) -> Option<&mut Item>`


`mod tests`

---

## game/src/tributes/lifecycle.rs

**Language:** Rust | **Size:** 21.1 KB | **Lines:** 624

**Imports:**
- `crate::areas::AreaDetails`
- `crate::areas::events::AreaEvent`
- `crate::messages::{MessagePayload, TaggedEvent, TributeRef}`
- `crate::output::GameOutput`
- `crate::tributes::Tribute`
- `crate::tributes::statuses::TributeStatus`
- `rand::RngExt`
- `rand::prelude::*`
- `rand::rngs::SmallRng`

**Declarations:**

`const MAX_HEALTH: u32 = 100`

`const MAX_SANITY: u32 = 100`

`const MAX_MOVEMENT: u32 = 100`

`const MAX_STRENGTH: u32 = 50`

`const MAX_BRAVERY: u32 = 100`

`const DEFAULT_HEAL: u32 = 5`

`const DEFAULT_MENTAL_HEAL: u32 = 5`

`const WOUNDED_DAMAGE: u32 = 1`

`const SICK_STRENGTH_REDUCTION: u32 = 1`

`const SICK_MOVEMENT_REDUCTION: u32 = 1`

`const ELECTROCUTED_DAMAGE: u32 = 20`

`const FROZEN_MOVEMENT_REDUCTION: u32 = 1`

`const OVERHEATED_MOVEMENT_REDUCTION: u32 = 1`

`const DEHYDRATED_STRENGTH_REDUCTION: u32 = 1`

`const STARVING_STRENGTH_REDUCTION: u32 = 1`

`const POISONED_MENTAL_DAMAGE: u32 = 5`

`const BROKEN_BONE_LEG_MOVEMENT_REDUCTION: u32 = 10`

`const BROKEN_BONE_ARM_STRENGTH_REDUCTION: u32 = 5`

`const BROKEN_BONE_SKULL_INTELLIGENCE_REDUCTION: u32 = 5`

`const BROKEN_BONE_RIB_DAMAGE: u32 = 5`

`const INFECTED_DAMAGE: u32 = 2`

`const INFECTED_MENTAL_DAMAGE: u32 = 5`

`const DROWNED_DAMAGE: u32 = 2`

`const DROWNED_MENTAL_DAMAGE: u32 = 2`

`const BURNED_DAMAGE: u32 = 5`

`const BURIED_DAMAGE: u32 = 5`

**`impl Tribute`**
  `pub fn dies(&mut self)`

  `pub fn is_alive(&self) -> bool`

  `pub(crate) fn hides(&mut self) -> bool`

  `pub fn is_visible(&self) -> bool`

  `pub(crate) fn misses_home(&mut self)`

  `pub(crate) fn takes_physical_damage(&mut self, damage: u32)`

  `pub(crate) fn takes_mental_damage(&mut self, damage: u32)`

  `pub(crate) fn reduce_strength(&mut self, amount: u32)`

  `pub(crate) fn increase_strength(&mut self, amount: u32)`

  `pub(crate) fn reduce_movement(&mut self, amount: u32)`

  `pub(crate) fn reduce_intelligence(&mut self, amount: u32)`

  `pub(crate) fn increase_bravery(&mut self, amount: u32)`

  `pub(crate) fn increase_movement(&mut self, amount: u32)`

  `pub(crate) fn heals(&mut self, health: u32)`

  `pub(crate) fn heals_mental_damage(&mut self, sanity: u32)`

  `pub(crate) fn short_rests(&mut self)`

  `pub(crate) fn long_rests(&mut self)`

  `pub fn recover_stamina( &mut self, action: &crate::tributes::actions::Action, sheltered: bool, hunger: crate::tributes::survival::HungerBand, thirst: crate::tributes::survival::ThirstBand, tuning: &crate::tributes::combat_tuning::CombatTuning, )`

  `pub fn set_status(&mut self, status: TributeStatus)`

  `pub(crate) fn apply_area_effects(&mut self, area_details: &AreaDetails)`

  `pub(crate) fn process_status( &mut self, area_details: &AreaDetails, rng: &mut impl Rng, events: &mut Vec<TaggedEvent>, )`


`mod tests`

---

## game/src/tributes/mod.rs

**Language:** Rust | **Size:** 62.5 KB | **Lines:** 1589

**Imports:**
- `pub use combat::{attack_contest, update_stats}`
- `pub use movement::TravelResult`
- `crate::areas::{Area, AreaDetails}`
- `crate::items::{Item, OwnsItems}`
- `crate::messages::{AreaRef, ItemRef, MessagePayload, TaggedEvent, TributeRef}`
- `crate::output::GameOutput`
- `crate::tributes::events::TributeEvent`
- `actions::{Action, AttackOutcome}`
- `brains::Brain`
- `fake::Fake`
- *... and 8 more imports*

**Declarations:**

`fn serialize_uuids_as_strings<S>(uuids: &[Uuid], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,`

`fn deserialize_uuids_lenient<'de, D>(deserializer: D) -> Result<Vec<Uuid>, D::Error> where D: Deserializer<'de>,`

`fn serialize_uuid_as_string<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,`

`fn deserialize_uuid_lenient<'de, D>(deserializer: D) -> Result<Uuid, D::Error> where D: Deserializer<'de>,`

`const SANITY_BREAK_LEVEL: u32 = 9`

**`impl Default for Tribute`**
  `fn default() -> Self`


**`impl Tribute`**
  `pub fn new(name: String, district: Option<u32>, avatar: Option<String>) -> Self`

  `pub fn random() -> Self`

  `pub fn avatar(&self) -> String`

  `pub fn process_turn_phase( &mut self, action_suggestion: Option<ActionSuggestion>, environment_details: &mut EnvironmentContext<'_>, encounter_context: EncounterContext, rng: &mut impl Rng, events: &mut Vec<TaggedEvent>, )`

  `fn pick_target( &self, mut targets: Vec<Tribute>, living_tributes_count: u32, events: &mut Vec<TaggedEvent>, ) -> Option<Tribute>`

  `pub fn wake_interrupted( &mut self, reason: shared::messages::InterruptionKind, phase: shared::messages::Phase, events: &mut Vec<TaggedEvent>, ) -> bool`

  `pub fn drain_alliance_events(&mut self) -> Vec<alliances::AllianceEvent>`

  `pub fn tick_alliance_timers(&mut self)`

  `pub fn consume_pending_trust_shock( &mut self, rng: &mut impl rand::Rng, events: &mut Vec<TaggedEvent>, )`


**`impl Default for Attributes`**
  `fn default() -> Self`


**`impl Attributes`**
  `pub fn new() -> Self`


`mod tests`

---

## game/src/tributes/movement.rs

**Language:** Rust | **Size:** 9.9 KB | **Lines:** 276

**Imports:**
- `crate::areas::Area`
- `crate::messages::{AreaRef, MessagePayload, TaggedEvent, TributeRef}`
- `crate::output::GameOutput`
- `crate::tributes::Tribute`
- `rand::prelude::*`
- `rand::rngs::SmallRng`

**Declarations:**

**`impl Tribute`**
  `pub(crate) fn travels( &self, closed_areas: &[Area], suggested_area: Option<Area>, events: &mut Vec<TaggedEvent>, ) -> TravelResult`


`mod tests`

---

## game/src/tributes/stamina_band.rs

**Language:** Rust | **Size:** 2.4 KB | **Lines:** 69

**Imports:**
- `crate::tributes::combat_tuning::CombatTuning`
- `shared::messages::StaminaBand`

**Declarations:**

`mod tests`

---

## game/src/tributes/statuses.rs

**Language:** Rust | **Size:** 5.6 KB | **Lines:** 153

**Imports:**
- `crate::threats::animals::Animal`
- `serde::{Deserialize, Serialize}`
- `std::fmt::Display`
- `std::str::FromStr`
- `strum::EnumIter`

**Declarations:**

**`impl FromStr for TributeStatus`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


**`impl Display for TributeStatus`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


`mod tests`

---

## game/src/tributes/survival.rs

**Language:** Rust | **Size:** 10.6 KB | **Lines:** 327

**Imports:**
- `crate::areas::weather::Weather`
- `crate::tributes::Tribute`
- `pub use shared::messages::{HungerBand, ThirstBand}`

**Declarations:**

`const HIGH_ATTR_THRESHOLD: u32 = 75`

`const LOW_ATTR_THRESHOLD: u32 = 25`

`mod tests`

---

## game/src/tributes/traits.rs

**Language:** Rust | **Size:** 14.3 KB | **Lines:** 459

**Imports:**
- `rand::Rng`
- `rand::RngExt`
- `serde::{Deserialize, Serialize}`

**Declarations:**

**`impl Trait`**
  `pub fn label(&self) -> &'static str`

  `pub fn alliance_affinity(&self) -> f64`


**`impl std::ops::Add for ThresholdDelta`**
  `fn add(self, rhs: Self) -> Self`


**`impl std::iter::Sum for ThresholdDelta`**
  `fn sum<I: Iterator<Item = ThresholdDelta>>(iter: I) -> Self`


**`impl Trait`**
  `pub fn threshold_modifiers(&self) -> ThresholdDelta`


`mod tests`

---

## game/src/witty_phrase_generator/codemap.md

**Language:** Markdown | **Size:** 8.4 KB | **Lines:** 173

**Declarations:**

---

## game/src/witty_phrase_generator/mod.rs

**Language:** Rust | **Size:** 8.0 KB | **Lines:** 260

**Imports:**
- `rand::RngExt`
- `rand::prelude::*`
- `rand::seq::SliceRandom`
- `std::cell::RefCell`

**Declarations:**

**`impl WPGen`**
  `pub fn new() -> WPGen`

  `fn create_format(words: usize) -> Vec<usize>`

  `fn generate_backtracking( &self, len_min: usize, len_max: usize, dep: usize, dict: &[Vec<&&'static str>; 4], format: &Vec<usize>, ) -> Option<Vec<&'static str>>`

  `pub fn generic( &self, words: usize, count: usize, len_min: Option<usize>, len_max: Option<usize>, word_len_max: Option<usize>, start_char: Option<char>, ) -> Option<Vec<Vec<&'static str>>>`

  `pub fn with_phrasewise_alliteration( &self, words: usize, count: usize, len_min: Option<usize>, len_max: Option<usize>, word_len_max: Option<usize>, ) -> Option<Vec<Vec<&'static str>>>`

  `pub fn with_words(&self, words: usize) -> Option<Vec<&'static str>>`


---

## game/tests/ai_terrain_behavior_test.rs

**Language:** Rust | **Size:** 9.4 KB | **Lines:** 262

**Imports:**
- `game::areas::{Area, AreaDetails}`
- `game::items::Item`
- `game::terrain::{BaseTerrain, TerrainType}`
- `game::tributes::Tribute`
- `game::tributes::brains::Brain`
- `rand::prelude::*`
- `rstest::{fixture, rstest}`

**Declarations:**

`fn small_rng() -> SmallRng`

`fn tribute_with_forest_affinity() -> Tribute`

`fn areas_with_terrain() -> Vec<AreaDetails>`

`fn test_destination_scoring_favors_affinity_terrain( tribute_with_forest_affinity: Tribute, areas_with_terrain: Vec<AreaDetails>, )`

`fn test_harsh_terrain_penalty_applied()`

`fn test_concealed_terrain_boosts_hiding(mut small_rng: SmallRng)`

`fn test_resource_scarce_terrain_boosts_search(mut small_rng: SmallRng)`

`fn test_desperate_tributes_flee_to_affinity_terrain()`

`fn test_concealed_visibility_bonus()`

`fn test_areas_with_items_bonus()`

`fn test_combined_scoring_factors()`

`fn test_desperate_modifier_strength()`

---

## game/tests/district_affinity_test.rs

**Language:** Rust | **Size:** 4.9 KB | **Lines:** 141

**Imports:**
- `game::districts::assign_terrain_affinity`
- `game::terrain::BaseTerrain`
- `game::tributes::Tribute`
- `rand::SeedableRng`
- `rand::rngs::SmallRng`
- `rstest::rstest`

**Declarations:**

`fn test_district_primary_affinity( #[case] district: u8, #[case] expected_primary: BaseTerrain, #[case] expected_bonus_pool: Vec<BaseTerrain>, )`

`fn test_affinity_count()`

`fn test_bonus_affinity_probability()`

`fn test_affinity_terrains_valid()`

---

## game/tests/event_game_loop_test.rs

**Language:** Rust | **Size:** 8.0 KB | **Lines:** 220

**Imports:**
- `game::areas::events::AreaEvent`
- `game::areas::{Area, AreaDetails}`
- `game::games::Game`
- `game::terrain::{BaseTerrain, TerrainType}`
- `game::tributes::Tribute`
- `rand::SeedableRng`
- `rand::rngs::SmallRng`

**Declarations:**

`fn test_event_survival_integration_with_game_loop()`

`fn test_trigger_cycle_events_calls_process_event()`

`fn test_terrain_affinity_improves_survival()`

---

## game/tests/event_integration_test.rs

**Language:** Rust | **Size:** 2.8 KB | **Lines:** 95

**Imports:**
- `game::areas::events::AreaEvent`
- `game::areas::{Area, AreaDetails}`
- `game::games::Game`
- `game::terrain::{BaseTerrain, TerrainType}`
- `game::tributes::Tribute`
- `rand::SeedableRng`
- `rand::rngs::SmallRng`

**Declarations:**

`fn test_wildfire_in_forest_kills_tributes()`

`fn test_wildfire_in_desert_minor_impact()`

---

## game/tests/event_severity_test.rs

**Language:** Rust | **Size:** 11.2 KB | **Lines:** 296

**Imports:**
- `game::areas::events::{AreaEvent, EventSeverity}`
- `game::terrain::BaseTerrain`
- `rand::SeedableRng`
- `rand::rngs::SmallRng`
- `rstest::rstest`

**Declarations:**

`fn test_severity_ordering()`

`fn test_wildfire_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_blizzard_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_sandstorm_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_flood_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_earthquake_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_avalanche_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_landslide_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_heatwave_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_drought_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_rockslide_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity)`

`fn test_survival_check_with_affinity()`

`fn test_survival_check_with_item_bonus()`

`fn test_survival_check_with_desperation()`

`fn test_survival_result_structure()`

`fn test_catastrophic_instant_death_probability()`

`fn test_desperation_rewards_distribution()`

---

## game/tests/event_unification_area_events_test.rs

**Language:** Rust | **Size:** 4.1 KB | **Lines:** 112

**Imports:**
- `game::areas::events::AreaEvent`
- `game::areas::{Area, AreaDetails}`
- `game::games::Game`
- `game::messages::{GameMessage, MessageSource}`
- `game::terrain::{BaseTerrain, TerrainType}`
- `game::tributes::Tribute`
- `rand::SeedableRng`
- `rand::rngs::SmallRng`

**Declarations:**

`fn area_event_survival_narration_reaches_game_messages()`

---

## game/tests/event_unification_combat_test.rs

**Language:** Rust | **Size:** 3.6 KB | **Lines:** 101

**Imports:**
- `game::areas::{Area, AreaDetails}`
- `game::games::Game`
- `game::messages::MessageSource`
- `game::terrain::{BaseTerrain, TerrainType}`
- `game::tributes::Tribute`

**Declarations:**

`fn combat_events_reach_game_messages_with_tribute_source()`

`fn is_tribute_sourced(m: &game::messages::GameMessage) -> bool`

---

## game/tests/event_unification_movement_test.rs

**Language:** Rust | **Size:** 3.1 KB | **Lines:** 84

**Imports:**
- `game::areas::{Area, AreaDetails}`
- `game::games::Game`
- `game::messages::{GameMessage, MessageSource}`
- `game::terrain::{BaseTerrain, TerrainType}`
- `game::tributes::Tribute`

**Declarations:**

`fn movement_and_turn_phase_events_reach_game_messages()`

---

## game/tests/item_distribution_test.rs

**Language:** Rust | **Size:** 8.4 KB | **Lines:** 278

**Imports:**
- `game::items::Item`
- `game::terrain::BaseTerrain`

**Declarations:**

`fn test_desert_favors_consumables()`

`fn test_urban_ruins_favors_weapons()`

`fn test_clearing_balanced_distribution()`

`fn test_mountains_favors_combat_gear()`

`fn test_tundra_distribution()`

`fn test_all_terrains_produce_valid_items()`

`fn test_item_weights_sum_to_one()`

`fn test_named_items_respect_terrain_weights()`

---

## game/tests/narrative_test.rs

**Language:** Rust | **Size:** 7.9 KB | **Lines:** 248

**Imports:**
- `game::messages::{hiding_spot_narrative, movement_narrative, stamina_narrative}`
- `game::terrain::BaseTerrain`

**Declarations:**

`fn test_desert_movement_narrative()`

`fn test_forest_movement_narrative()`

`fn test_mountains_movement_narrative()`

`fn test_forest_hiding_narrative()`

`fn test_urban_ruins_hiding_narrative()`

`fn test_desert_hiding_narrative()`

`fn test_tundra_hiding_narrative()`

`fn test_high_stamina_harsh_terrain()`

`fn test_low_stamina_harsh_terrain()`

`fn test_medium_stamina_moderate_terrain()`

`fn test_low_stamina_mild_terrain()`

`fn test_all_terrains_movement_narrative()`

`fn test_all_terrains_hiding_narrative()`

`fn test_concealed_terrains_better_hiding()`

`fn test_exposed_terrains_poor_hiding()`

`fn test_stamina_threshold_differences()`

`fn test_harsh_terrain_more_severe_stamina()`

---

## game/tests/stamina_combat_integration.rs

**Language:** Rust | **Size:** 1.9 KB | **Lines:** 56

**Imports:**
- `game::games::Game`
- `game::tributes::Tribute`
- `shared::messages::{MessagePayload, StaminaBand}`

**Declarations:**

`fn per_phase_loop_emits_stamina_band_changed_when_band_crosses()`

`fn fresh_tribute_emits_no_band_change_when_recovery_keeps_band()`

---

## game/tests/stamina_edge_cases_test.rs

**Language:** Rust | **Size:** 10.7 KB | **Lines:** 306

**Imports:**
- `game::terrain::{BaseTerrain, TerrainType}`
- `game::tributes::actions::Action`
- `game::tributes::{Tribute, calculate_stamina_cost}`
- `rstest::rstest`

**Declarations:**

`fn test_base_stamina_costs(#[case] action: Action, #[case] expected_base: u32)`

`fn test_terrain_multiplier(#[case] base_terrain: BaseTerrain, #[case] multiplier: f32)`

`fn test_affinity_modifier_with_affinity()`

`fn test_affinity_modifier_without_affinity()`

`fn test_desperation_multiplier(#[case] health: u32, #[case] desperation: f32)`

`fn test_all_multipliers_combined()`

`fn test_best_case_combined()`

`fn test_stamina_restoration()`

`fn test_zero_stamina_calculation()`

`fn test_cost_exceeds_max_stamina()`

`fn test_multiple_terrain_affinities()`

`fn test_negative_health_clamped()`

`fn test_tribute_new_initializes_stamina()`

`fn test_action_type_ordering()`

`fn test_rounding_behavior()`

`fn test_rounding_up()`

---

## game/tests/survival_integration.rs

**Language:** Rust | **Size:** 3.1 KB | **Lines:** 90

**Imports:**
- `game::areas::weather::Weather`
- `game::tributes::Tribute`
- `game::tributes::survival::{
    ThirstBand, apply_dehydration_drain, apply_starvation_drain, drink_water, thirst_band,
    tick_survival,
}`

**Declarations:**

`fn mid_tribute(name: &str) -> Tribute`

`fn no_food_no_water_dies_of_dehydration_first()`

`fn carrying_water_extends_survival()`

`fn sheltered_in_heatwave_does_not_accrue_weather_thirst()`

---

## game/tests/terrain_assignment_test.rs

**Language:** Rust | **Size:** 2.2 KB | **Lines:** 72

**Imports:**
- `game::terrain::{BaseTerrain, Harshness, TerrainType}`
- `rand::SeedableRng`
- `rand::rngs::SmallRng`

**Declarations:**

`fn test_random_terrain_generates_valid_terrain()`

`fn test_random_safe_generates_mild_terrains()`

`fn test_balance_constraint_limits_harsh_terrains()`

`fn test_descriptor_generation_respects_compatibility()`

---

## game/tests/terrain_compatibility_test.rs

**Language:** Rust | **Size:** 1.4 KB | **Lines:** 47

**Imports:**
- `game::terrain::{BaseTerrain, TerrainDescriptor, TerrainType}`

**Declarations:**

`fn test_desert_cannot_be_wet()`

`fn test_tundra_cannot_be_hot()`

`fn test_tundra_must_be_cold_or_frozen()`

`fn test_geothermal_must_be_hot()`

`fn test_forest_can_be_wet()`

`fn test_empty_descriptors_valid()`

---

## game/tests/terrain_config_test.rs

**Language:** Rust | **Size:** 1.5 KB | **Lines:** 60

**Imports:**
- `game::terrain::{BaseTerrain, Harshness, Visibility}`
- `strum::IntoEnumIterator`

**Declarations:**

`fn test_movement_costs_within_range()`

`fn test_clearing_is_mild()`

`fn test_tundra_is_harsh()`

`fn test_forest_is_concealed()`

`fn test_desert_is_exposed()`

`fn test_item_weights_sum_to_one()`

`fn test_item_spawn_modifiers_reasonable()`

---

## game/tests/terrain_specific_events_test.rs

**Language:** Rust | **Size:** 6.2 KB | **Lines:** 168

**Imports:**
- `game::areas::events::AreaEvent`
- `game::areas::{Area, AreaDetails}`
- `game::games::Game`
- `game::terrain::{BaseTerrain, TerrainType}`
- `std::collections::HashMap`

**Declarations:**

`fn test_game_loop_generates_terrain_appropriate_events()`

`fn test_terrain_event_diversity()`

`fn test_terrain_specific_events_have_logical_severity()`

---

## migrations/codemap.md

**Language:** Markdown | **Size:** 374 B | **Lines:** 19

**Declarations:**

---

## migrations/definitions/20260419_133608_ItemDurability.json

**Language:** JSON | **Size:** 23.4 KB | **Lines:** 1

---

## migrations/definitions/20260427_120000_GameEventPayload.json

**Language:** JSON | **Size:** 2.6 KB | **Lines:** 1

---

## migrations/definitions/20260427_180000_GameMessagePayloadV2.json

**Language:** JSON | **Size:** 3.0 KB | **Lines:** 1

---

## migrations/definitions/20260501_120000_TributeAlliesString.json

**Language:** JSON | **Size:** 924 B | **Lines:** 1

---

## migrations/definitions/20260501_180000_DisplayGameWinnerTributeRef.json

**Language:** JSON | **Size:** 56 B | **Lines:** 1

---

## migrations/definitions/20260503_120000_TributeSurvivalFields.json

**Language:** JSON | **Size:** 813 B | **Lines:** 1

---

## migrations/definitions/_initial.json

**Language:** JSON | **Size:** 10.8 KB | **Lines:** 1

---

## migrations/definitions/codemap.md

**Language:** Markdown | **Size:** 386 B | **Lines:** 19

**Declarations:**

---

## rustfmt.toml

**Language:** TOML | **Size:** 17 B | **Lines:** 1

**Declarations:**

`edition = "2024"`

---

## schemas/codemap.md

**Language:** Markdown | **Size:** 6.4 KB | **Lines:** 183

**Declarations:**

---

## shared/Cargo.toml

**Language:** TOML | **Size:** 330 B | **Lines:** 12

**Imports:**
- `chrono`
- `serde`
- `serde_json`
- `validator`
- `uuid`

**Declarations:**

---

## shared/codemap.md

**Language:** Markdown | **Size:** 2.1 KB | **Lines:** 55

**Declarations:**

---

## shared/src/codemap.md

**Language:** Markdown | **Size:** 374 B | **Lines:** 19

**Declarations:**

---

## shared/src/combat_beat.rs

**Language:** Rust | **Size:** 5.4 KB | **Lines:** 146

**Imports:**
- `crate::messages::{ItemRef, TributeRef}`
- `serde::{Deserialize, Serialize}`

**Declarations:**

`mod tests`

---

## shared/src/lib.rs

**Language:** Rust | **Size:** 9.7 KB | **Lines:** 318

**Imports:**
- `serde::{Deserialize, Serialize}`
- `std::fmt::{Debug, Display}`
- `std::str::FromStr`
- `validator::{Validate, ValidationError}`
- `crate::messages::{GameMessage, TributeRef}`

**Declarations:**

**`impl ItemQuantity`**
  `pub fn base_item_count(&self) -> u32`


**`impl EventFrequency`**
  `pub fn event_probability(&self) -> f32`


`fn validate_uuid(value: &str) -> Result<(), ValidationError>`

**`impl EditTribute`**
  `pub fn from_tuple(data: (String, String, String, String)) -> Self`

  `pub fn to_tuple(&self) -> (String, String, String, String)`


**`impl Display for GameStatus`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl FromStr for GameStatus`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


`mod tests`

---

## shared/src/messages.rs

**Language:** Rust | **Size:** 38.5 KB | **Lines:** 1283

**Imports:**
- `chrono::{DateTime, Utc}`
- `serde::{Deserialize, Serialize}`
- `std::str::FromStr`
- `uuid::Uuid`

**Declarations:**

**`impl Phase`**
  `pub const fn ord(self) -> u8`

  `pub const fn next(self) -> Phase`

  `pub const fn all() -> [Phase; 4]`


**`impl std::fmt::Display for Phase`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl std::fmt::Display for ParsePhaseError`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl std::error::Error for ParsePhaseError`**

**`impl FromStr for Phase`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


**`impl StaminaBand`**
  `pub fn as_str(self) -> &'static str`


**`impl std::fmt::Display for StaminaBand`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl HungerBand`**
  `pub fn as_str(self) -> &'static str`


**`impl std::fmt::Display for HungerBand`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl ThirstBand`**
  `pub fn as_str(self) -> &'static str`


**`impl std::fmt::Display for ThirstBand`**
  `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`


**`impl MessagePayload`**
  `pub fn kind(&self) -> MessageKind`

  `pub fn involves(&self, tribute_identifier: &str) -> bool`


**`impl PartialEq for GameMessage`**
  `fn eq(&self, other: &Self) -> bool`


**`impl GameMessage`**
  `pub fn new( source: MessageSource, game_day: u32, phase: Phase, tick: u32, emit_index: u32, subject: String, content: String, payload: MessagePayload, ) -> Self`


`mod tests`

`mod survival_event_tests`

---

## src/lib.rs

**Language:** Rust | **Size:** 0 B | **Lines:** 0

---

## web/Cargo.toml

**Language:** TOML | **Size:** 852 B | **Lines:** 32

**Imports:**
- `chrono`
- `dioxus`
- `dioxus-query`
- `futures-util`
- `game`
- `gloo-net`
- `gloo-storage`
- `serde`
- `shared`
- `reqwest`
- *... and 8 more imports*

**Declarations:**

---

## web/Dioxus.toml

**Language:** TOML | **Size:** 966 B | **Lines:** 48

**Declarations:**

---

## web/assets/icons.svg

**Language:** XML | **Size:** 57.2 KB | **Lines:** 1

**Declarations:**

---

## web/assets/images/map.svg

**Language:** XML | **Size:** 2.1 KB | **Lines:** 29

**Declarations:**

---

## web/assets/images/waves.svg

**Language:** XML | **Size:** 7.7 KB | **Lines:** 86

**Declarations:**

---

## web/assets/package-lock.json

**Language:** JSON | **Size:** 30.2 KB | **Lines:** 940

**Declarations:**

---

## web/assets/package.json

**Language:** JSON | **Size:** 92 B | **Lines:** 6

**Imports:**
- `@tailwindcss/cli`
- `tailwindcss`

**Declarations:**

---

## web/assets/src/main.css

**Language:** CSS | **Size:** 1.8 KB | **Lines:** 78

**Imports:**
- `tailwindcss`

**Declarations:**

---

## web/build.rs

**Language:** Rust | **Size:** 4.2 KB | **Lines:** 139

**Imports:**
- `dotenvy::dotenv_override`
- `std::env`
- `std::fs`
- `std::fs::File`
- `std::io::Write`
- `std::path::Path`

**Declarations:**

`fn main()`

`fn generate_env_file()`

`fn generate_sprite_sheet()`

`fn process_icons_dir(dir: &Path, output: &mut String)`

`fn extract_svg_data(content: &str) -> Option<(String, String)>`

---

## web/codemap.md

**Language:** Markdown | **Size:** 367 B | **Lines:** 19

**Declarations:**

---

## web/src/api_url.rs

**Language:** Rust | **Size:** 1.6 KB | **Lines:** 50

**Imports:**
- `crate::env::APP_API_HOST`

**Declarations:**

`fn origin() -> String`

`fn origin() -> String`

`mod tests`

---

## web/src/cache.rs

**Language:** Rust | **Size:** 569 B | **Lines:** 26

**Declarations:**

`pub(crate) enum QueryError`
> Variants: `BadJson`, `GameNotFound`, `NoGames`, `TributeNotFound`, `Unauthorized`, `Unknown`, `ServerNotFound`, `ServerVersionNotFound`

`pub(crate) enum MutationError`
> Variants: `UnableToCreateGame`, `Unauthorized`, `Unknown`, `UnableToAdvanceGame`, `UnableToRegisterUser`, `RegistrationFailed`, `UnableToAuthenticateUser`, `_UnableToPublishGame`, `_UnableToUnpublishGame`

---

## web/src/codemap.md

**Language:** Markdown | **Size:** 10.1 KB | **Lines:** 234

**Declarations:**

---

## web/src/components/accounts.rs

**Language:** Rust | **Size:** 16.5 KB | **Lines:** 436

**Imports:**
- `crate::cache::MutationError`
- `crate::components::{Input, ThemedButton}`
- `crate::http::WithCredentials`
- `crate::routes::Routes`
- `crate::storage::{AppState, use_persistent}`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `shared::{AuthenticatedUser, RegistrationUser}`
- `validator::Validate`

**Declarations:**

`pub(crate) struct RegisterUserM`

**`impl MutationCapability for RegisterUserM`**
  `async fn run(&self, user: &RegistrationUser) -> Result<AuthenticatedUser, MutationError>`


`pub(crate) struct LoginUserM`

**`impl MutationCapability for LoginUserM`**
  `async fn run(&self, user: &RegistrationUser) -> Result<AuthenticatedUser, MutationError>`


`fn LoginForm() -> Element`

`fn RegisterForm() -> Element`

`fn LogoutButton() -> Element`

---

## web/src/components/app.rs

**Language:** Rust | **Size:** 3.6 KB | **Lines:** 113

**Imports:**
- `crate::LoadingState`
- `crate::components::game_edit::EditGameModal`
- `crate::components::icons::svg_icon::SpriteSheetLoader`
- `crate::components::loading_modal::LoadingModal`
- `crate::components::server_version::ServerVersion`
- `crate::components::tribute_edit::EditTributeModal`
- `crate::routes::Routes`
- `crate::storage::{AppState, use_persistent}`
- `crate::theme::Theme`
- `dioxus::prelude::*`
- *... and 2 more imports*

**Declarations:**

---

## web/src/components/area_detail.rs

**Language:** Rust | **Size:** 5.5 KB | **Lines:** 141

**Imports:**
- `crate::cache::QueryError`
- `crate::components::icons::lock_closed::LockClosedIcon`
- `crate::components::icons::lock_open::LockOpenIcon`
- `crate::components::icons::uturn::UTurnIcon`
- `crate::components::item_icon::ItemIcon`
- `crate::http::WithCredentials`
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `game::areas::AreaDetails`

**Declarations:**

`pub(crate) struct AreaDetailQ`

**`impl QueryCapability for AreaDetailQ`**
  `async fn run(&self, keys: &(String, String)) -> Result<Box<AreaDetails>, QueryError>`


---

## web/src/components/button.rs

**Language:** Rust | **Size:** 1.8 KB | **Lines:** 67

**Imports:**
- `dioxus::prelude::*`
- `std::rc::Rc`

**Declarations:**

---

## web/src/components/codemap.md

**Language:** Markdown | **Size:** 15.2 KB | **Lines:** 497

**Declarations:**

---

## web/src/components/create_game.rs

**Language:** Rust | **Size:** 6.2 KB | **Lines:** 188

**Imports:**
- `crate::LoadingState`
- `crate::cache::MutationError`
- `crate::components::games_list::GamesListQ`
- `crate::components::{Input, ThemedButton}`
- `crate::http::WithCredentials`
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `game::games::Game`
- `shared::CreateGame`

**Declarations:**

`pub(crate) struct CreateGameM`

**`impl MutationCapability for CreateGameM`**
  `async fn run(&self, args: &Self::Keys) -> Result<Game, MutationError>`

  `async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Game, MutationError>)`


---

## web/src/components/credits.rs

**Language:** Rust | **Size:** 8.6 KB | **Lines:** 124

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

`const LINK_CLASS: &str = "text-primary hover:underline"`

`const H2_CLASS: &str = "font-display text-3xl tracking-wide text-text mt-4"`

---

## web/src/components/filter_chips.rs

**Language:** Rust | **Size:** 3.6 KB | **Lines:** 97

**Imports:**
- `crate::components::timeline::{FilterMode, PeriodFilters}`
- `dioxus::prelude::*`
- `shared::messages::MessageKind`
- `std::collections::HashSet`

**Declarations:**

`const CATEGORIES: &[(MessageKind, &str)] = &[ (MessageKind::Death, "Deaths"), (MessageKind::Combat, "Combat"), (MessageKind::Alliance, "Alliances"), (MessageKind::Movement, "Movement"), (MessageKind::Item, "Items"), ]`

---

## web/src/components/game_areas.rs

**Language:** Rust | **Size:** 6.0 KB | **Lines:** 166

**Imports:**
- `crate::cache::QueryError`
- `crate::components::icons::lock_closed::LockClosedIcon`
- `crate::components::icons::lock_open::LockOpenIcon`
- `crate::components::item_icon::ItemIcon`
- `crate::components::map::Map`
- `crate::http::WithCredentials`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `game::areas::AreaDetails`
- `shared::DisplayGame`

**Declarations:**

`pub(crate) struct GameAreasQ`

**`impl QueryCapability for GameAreasQ`**
  `async fn run(&self, identifier: &String) -> Result<Vec<AreaDetails>, QueryError>`


---

## web/src/components/game_delete.rs

**Language:** Rust | **Size:** 4.6 KB | **Lines:** 150

**Imports:**
- `crate::cache::MutationError`
- `crate::components::Button`
- `crate::components::games_list::GamesListQ`
- `crate::components::icons::delete::DeleteIcon`
- `crate::http::WithCredentials`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `gloo_storage::Storage`
- `shared::DeleteGame`

**Declarations:**

`pub(crate) struct DeleteGameM`

**`impl MutationCapability for DeleteGameM`**
  `async fn run(&self, args: &DeleteGame) -> Result<(String, String), MutationError>`

  `async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>)`


---

## web/src/components/game_detail.rs

**Language:** Rust | **Size:** 18.4 KB | **Lines:** 603

**Imports:**
- `crate::LoadingState`
- `crate::cache::{MutationError, QueryError}`
- `crate::components::ThemedButton`
- `crate::components::game_areas::GameAreaList`
- `crate::components::game_edit::GameEdit`
- `crate::components::game_tributes::GameTributes`
- `crate::components::games_list::GamesListQ`
- `crate::components::info_detail::InfoDetail`
- `crate::components::period_grid::PeriodGrid`
- `crate::components::recap_card::RecapCard`
- *... and 10 more imports*

**Declarations:**

`pub(crate) struct DisplayGameQ`

**`impl QueryCapability for DisplayGameQ`**
  `async fn run(&self, identifier: &String) -> Result<Box<DisplayGame>, QueryError>`


`pub(crate) struct NextStepM`

`pub(crate) enum NextStepResult`
> Variants: `Started`, `Finished`, `Advanced`

**`impl NextStepResult`**
  `pub fn identifier(&self) -> &String`


**`impl MutationCapability for NextStepM`**
  `async fn run(&self, args: &String) -> Result<NextStepResult, MutationError>`

  `async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>)`


`mod tests`

`fn GameState(identifier: String) -> Element`

`fn GameStats(identifier: String) -> Element`

`fn GameDetails(identifier: String) -> Element`

---

## web/src/components/game_edit.rs

**Language:** Rust | **Size:** 5.5 KB | **Lines:** 199

**Imports:**
- `crate::cache::MutationError`
- `crate::components::game_detail::DisplayGameQ`
- `crate::components::games_list::GamesListQ`
- `crate::components::icons::edit::EditIcon`
- `crate::components::modal::{Modal, Props as ModalProps}`
- `crate::components::{Button, Input}`
- `crate::http::WithCredentials`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `shared::EditGame`

**Declarations:**

`pub(crate) struct EditGameM`

**`impl MutationCapability for EditGameM`**
  `async fn run(&self, args: &EditGame) -> Result<String, MutationError>`

  `async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>)`


---

## web/src/components/game_period_page.rs

**Language:** Rust | **Size:** 5.7 KB | **Lines:** 160

**Imports:**
- `crate::cache::QueryError`
- `crate::components::TributeFilterChips`
- `crate::components::filter_chips::FilterChips`
- `crate::components::timeline::{FilterMode, PeriodFilters, Timeline}`
- `crate::hooks::use_timeline_summary::use_timeline_summary`
- `crate::http::WithCredentials`
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `reqwest::StatusCode`
- *... and 1 more imports*

**Declarations:**

`pub(crate) struct DayLogQ`

**`impl QueryCapability for DayLogQ`**
  `async fn run(&self, keys: &(String, u32)) -> Result<Vec<GameMessage>, QueryError>`


---

## web/src/components/game_tributes.rs

**Language:** Rust | **Size:** 9.0 KB | **Lines:** 268

**Imports:**
- `crate::cache::QueryError`
- `crate::components::icons::loading::LoadingIcon`
- `crate::components::icons::map_pin::MapPinIcon`
- `crate::components::item_icon::ItemIcon`
- `crate::components::tribute_edit::TributeEdit`
- `crate::components::tribute_state_strip::TributeStateStrip`
- `crate::components::tribute_status_icon::TributeStatusIcon`
- `crate::http::WithCredentials`
- `crate::routes::Routes`
- `dioxus::prelude::*`
- *... and 5 more imports*

**Declarations:**

`pub(crate) struct GameTributesQ`

**`impl QueryCapability for GameTributesQ`**
  `async fn run(&self, game_identifier: &String) -> Result<PaginatedTributesResponse, QueryError>`


---

## web/src/components/games.rs

**Language:** Rust | **Size:** 193 B | **Lines:** 12

**Imports:**
- `crate::routes::Routes`
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/games_list.rs

**Language:** Rust | **Size:** 7.3 KB | **Lines:** 236

**Imports:**
- `crate::cache::QueryError`
- `crate::components::game_edit::GameEdit`
- `crate::components::icons::eye_closed::EyeClosedIcon`
- `crate::components::icons::eye_open::EyeOpenIcon`
- `crate::components::{Button, CreateGameButton, CreateGameForm, DeleteGameModal, GameDelete}`
- `crate::http::WithCredentials`
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `serde::{Deserialize, Serialize}`
- *... and 1 more imports*

**Declarations:**

`pub(crate) struct GamesListQ`

**`impl QueryCapability for GamesListQ`**
  `async fn run(&self, _keys: &()) -> Result<PaginatedGamesResponse, QueryError>`


`fn NoGames() -> Element`

`fn RefreshButton() -> Element`

`fn LoadMoreButton(current_offset: u32, limit: u32) -> Element`

---

## web/src/components/home.rs

**Language:** Rust | **Size:** 427 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/codemap.md

**Language:** Markdown | **Size:** 388 B | **Lines:** 19

**Declarations:**

---

## web/src/components/icons/delete.rs

**Language:** Rust | **Size:** 905 B | **Lines:** 16

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/edit.rs

**Language:** Rust | **Size:** 727 B | **Lines:** 17

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/eye_closed.rs

**Language:** Rust | **Size:** 1.0 KB | **Lines:** 23

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/eye_open.rs

**Language:** Rust | **Size:** 762 B | **Lines:** 22

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/broken_bone.rs

**Language:** Rust | **Size:** 1.1 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/burned.rs

**Language:** Rust | **Size:** 1.2 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/codemap.md

**Language:** Markdown | **Size:** 403 B | **Lines:** 19

**Declarations:**

---

## web/src/components/icons/game_icons_net/dead.rs

**Language:** Rust | **Size:** 838 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/dehydrated.rs

**Language:** Rust | **Size:** 2.7 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/drowning.rs

**Language:** Rust | **Size:** 3.5 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/electrocuted.rs

**Language:** Rust | **Size:** 1.6 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/falling_rocks.rs

**Language:** Rust | **Size:** 724 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/fishing_net.rs

**Language:** Rust | **Size:** 4.4 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/fist.rs

**Language:** Rust | **Size:** 2.1 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/fizzing_flask.rs

**Language:** Rust | **Size:** 2.3 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/frozen_body.rs

**Language:** Rust | **Size:** 1.7 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/harpoon_trident.rs

**Language:** Rust | **Size:** 978 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/health_potion.rs

**Language:** Rust | **Size:** 1.2 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/hearts.rs

**Language:** Rust | **Size:** 450 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/heat_haze.rs

**Language:** Rust | **Size:** 544 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/high_shot.rs

**Language:** Rust | **Size:** 816 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/hypodermic_test.rs

**Language:** Rust | **Size:** 1.7 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/infection.rs

**Language:** Rust | **Size:** 3.8 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/mauled.rs

**Language:** Rust | **Size:** 2.5 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/mod.rs

**Language:** Rust | **Size:** 1.3 KB | **Lines:** 71

**Imports:**
- `pub use broken_bone::BrokenBoneIcon`
- `pub use burned::BurnedIcon`
- `pub use dead::*`
- `pub use dehydrated::*`
- `pub use drowning::*`
- `pub use electrocuted::*`
- `pub use falling_rocks::*`
- `pub use fishing_net::*`
- `pub use fist::*`
- `pub use fizzing_flask::*`
- *... and 25 more imports*

**Declarations:**

`mod broken_bone`

`mod burned`

`mod dead`

`mod dehydrated`

`mod drowning`

`mod electrocuted`

`mod falling_rocks`

`mod fishing_net`

`mod fist`

`mod fizzing_flask`

`mod frozen_body`

`mod harpoon_trident`

`mod health_potion`

`mod hearts`

`mod heat_haze`

`mod high_shot`

`mod hypodermic_test`

`mod infection`

`mod mauled`

`mod plain_dagger`

`mod pointy_sword`

`mod poison_bottle`

`mod powder`

`mod recently_dead`

`mod shield`

`mod spear_hook`

`mod spiked_mace`

`mod spinning_top`

`mod spray`

`mod starving`

`mod switchblade`

`mod trail_mix`

`mod vomiting`

`mod wood_axe`

`mod wounded`

---

## web/src/components/icons/game_icons_net/plain_dagger.rs

**Language:** Rust | **Size:** 1.2 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/pointy_sword.rs

**Language:** Rust | **Size:** 1.1 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/poison_bottle.rs

**Language:** Rust | **Size:** 2.6 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/powder.rs

**Language:** Rust | **Size:** 3.0 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/recently_dead.rs

**Language:** Rust | **Size:** 1.1 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/shield.rs

**Language:** Rust | **Size:** 334 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/spear_hook.rs

**Language:** Rust | **Size:** 1.4 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/spiked_mace.rs

**Language:** Rust | **Size:** 1.9 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/spinning_top.rs

**Language:** Rust | **Size:** 1.0 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/spray.rs

**Language:** Rust | **Size:** 2.5 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/starving.rs

**Language:** Rust | **Size:** 907 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/switchblade.rs

**Language:** Rust | **Size:** 926 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/trail_mix.rs

**Language:** Rust | **Size:** 4.5 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/vomiting.rs

**Language:** Rust | **Size:** 1.6 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/wood_axe.rs

**Language:** Rust | **Size:** 933 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/game_icons_net/wounded.rs

**Language:** Rust | **Size:** 3.8 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/loading.rs

**Language:** Rust | **Size:** 328 B | **Lines:** 16

**Imports:**
- `crate::components::icons::mockingjay_arrow::MockingjayArrow`
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/lock_closed.rs

**Language:** Rust | **Size:** 495 B | **Lines:** 16

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/lock_open.rs

**Language:** Rust | **Size:** 428 B | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/map_pin.rs

**Language:** Rust | **Size:** 793 B | **Lines:** 17

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/mockingjay.rs

**Language:** Rust | **Size:** 9.5 KB | **Lines:** 14

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/mockingjay_arrow.rs

**Language:** Rust | **Size:** 4.0 KB | **Lines:** 26

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/mockingjay_flight.rs

**Language:** Rust | **Size:** 6.3 KB | **Lines:** 13

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/mod.rs

**Language:** Rust | **Size:** 272 B | **Lines:** 14

**Declarations:**

---

## web/src/components/icons/svg_icon.rs

**Language:** Rust | **Size:** 3.2 KB | **Lines:** 96

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons/uturn.rs

**Language:** Rust | **Size:** 512 B | **Lines:** 16

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/icons_page.rs

**Language:** Rust | **Size:** 3.7 KB | **Lines:** 122

**Imports:**
- `crate::components::icons::game_icons_net::*`
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/info_detail.rs

**Language:** Rust | **Size:** 1.4 KB | **Lines:** 41

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/input.rs

**Language:** Rust | **Size:** 1.1 KB | **Lines:** 42

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/item_detail.rs

**Language:** Rust | **Size:** 3.7 KB | **Lines:** 102

**Imports:**
- `crate::cache::QueryError`
- `crate::components::icons::uturn::UTurnIcon`
- `crate::components::item_icon::ItemIcon`
- `crate::http::WithCredentials`
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `game::items::Item`

**Declarations:**

`pub(crate) struct ItemDetailQ`

**`impl QueryCapability for ItemDetailQ`**
  `async fn run(&self, keys: &(String, String)) -> Result<Box<Item>, QueryError>`


---

## web/src/components/item_icon.rs

**Language:** Rust | **Size:** 379 B | **Lines:** 15

**Imports:**
- `crate::components::icons::svg_icon::{SvgIcon, icon_name_for_item}`
- `dioxus::prelude::*`
- `game::items::Item`

**Declarations:**

---

## web/src/components/loading_modal.rs

**Language:** Rust | **Size:** 652 B | **Lines:** 26

**Imports:**
- `crate::LoadingState`
- `crate::components::icons::loading::LoadingIcon`
- `crate::components::modal::{Modal, Props as ModalProps}`
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/map.rs

**Language:** Rust | **Size:** 6.0 KB | **Lines:** 144

**Imports:**
- `crate::components::map_affordance_overlay::MapAffordanceOverlay`
- `dioxus::prelude::*`
- `game::areas::Area`
- `game::areas::AreaDetails`
- `game::areas::hex::{SUB_SIZE_RATIO, SUB_SLOTS, default_layout}`

**Declarations:**

`const HEX_SIZE: f64 = 90.0`

`const PADDING: f64 = 16.0`

`fn hex_corners(cx: f64, cy: f64, size: f64) -> String`

---

## web/src/components/map_affordance_overlay.rs

**Language:** Rust | **Size:** 1.6 KB | **Lines:** 55

**Imports:**
- `dioxus::prelude::*`
- `game::areas::AreaDetails`
- `game::areas::forage::forage_richness`
- `game::areas::shelter::shelter_quality`
- `game::areas::water::water_source`
- `game::areas::weather::current_weather`

**Declarations:**

---

## web/src/components/mod.rs

**Language:** Rust | **Size:** 1.7 KB | **Lines:** 71

**Imports:**
- `pub use accounts::{Accounts, AccountsPage}`
- `pub use app::App`
- `pub use area_detail::AreaDetail`
- `pub use button::{Button, ThemedButton}`
- `pub use create_game::{CreateGameButton, CreateGameForm}`
- `pub use credits::Credits`
- `pub use filter_chips::FilterChips`
- `pub use game_delete::{DeleteGameModal, GameDelete}`
- `pub use game_detail::GamePage`
- `pub use game_period_page::GamePeriodPage`
- *... and 21 more imports*

**Declarations:**

`mod accounts`

`mod app`

`mod area_detail`

`mod button`

`mod create_game`

`mod credits`

`mod filter_chips`

`mod game_areas`

`mod game_delete`

`mod game_detail`

`mod game_edit`

`mod game_period_page`

`mod games`

`mod home`

`mod icons_page`

`mod info_detail`

`mod input`

`mod item_icon`

`mod item_detail`

`mod loading_modal`

`mod map`

`mod map_affordance_overlay`

`mod navbar`

`mod period_card`

`mod period_grid`

`mod period_grid_empty`

`mod recap_card`

`mod tribute_detail`

`mod server_version`

`mod tribute_edit`

`mod tribute_filter_chips`

`mod tribute_state_strip`

`mod tribute_status_icon`

`mod tribute_survival_section`

---

## web/src/components/modal.rs

**Language:** Rust | **Size:** 1.6 KB | **Lines:** 61

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/navbar.rs

**Language:** Rust | **Size:** 1.7 KB | **Lines:** 47

**Imports:**
- `crate::components::ui::{Button, ButtonVariant, TopBar}`
- `crate::routes::Routes`
- `crate::storage::{AppState, use_persistent}`
- `crate::theme::Theme`
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/period_card.rs

**Language:** Rust | **Size:** 2.2 KB | **Lines:** 75

**Imports:**
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `shared::messages::Phase`

**Declarations:**

`fn phase_label(phase: Phase) -> &'static str`

`fn phase_visual(phase: Phase) -> (&'static str, &'static str)`

---

## web/src/components/period_grid.rs

**Language:** Rust | **Size:** 2.1 KB | **Lines:** 56

**Imports:**
- `crate::cache::QueryError`
- `crate::components::period_card::PeriodCard`
- `crate::components::period_grid_empty::{EmptyKind, PeriodGridEmpty}`
- `crate::hooks::use_timeline_summary`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`

**Declarations:**

---

## web/src/components/period_grid_empty.rs

**Language:** Rust | **Size:** 1.2 KB | **Lines:** 40

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/recap_card.rs

**Language:** Rust | **Size:** 2.3 KB | **Lines:** 65

**Imports:**
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `gloo_storage::{LocalStorage, Storage}`
- `shared::DisplayGame`

**Declarations:**

---

## web/src/components/server_version.rs

**Language:** Rust | **Size:** 1.0 KB | **Lines:** 38

**Imports:**
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`

**Declarations:**

`pub(crate) struct ServerVersionQ`

**`impl QueryCapability for ServerVersionQ`**
  `async fn run(&self, _keys: &()) -> Result<String, ()>`


---

## web/src/components/timeline/cards/alliance_card.rs

**Language:** Rust | **Size:** 1.5 KB | **Lines:** 44

**Imports:**
- `dioxus::prelude::*`
- `shared::messages::{GameMessage, MessagePayload, TributeRef}`

**Declarations:**

`fn joined(members: &[TributeRef]) -> String`

---

## web/src/components/timeline/cards/combat_card.rs

**Language:** Rust | **Size:** 2.2 KB | **Lines:** 61

**Imports:**
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `shared::messages::{CombatOutcome, TributeRef}`

**Declarations:**

---

## web/src/components/timeline/cards/combat_swing_card.rs

**Language:** Rust | **Size:** 5.0 KB | **Lines:** 121

**Imports:**
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `shared::combat_beat::{CombatBeat, SwingOutcome, WearOutcomeReport}`

**Declarations:**

`fn outcome_summary(outcome: &SwingOutcome) -> (&'static str, String)`

---

## web/src/components/timeline/cards/cycle_card.rs

**Language:** Rust | **Size:** 1.4 KB | **Lines:** 38

**Imports:**
- `dioxus::prelude::*`
- `shared::messages::{GameMessage, MessagePayload, Phase}`

**Declarations:**

`fn phase_visual(phase: Phase) -> (&'static str, &'static str)`

---

## web/src/components/timeline/cards/death_card.rs

**Language:** Rust | **Size:** 1.5 KB | **Lines:** 49

**Imports:**
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `shared::messages::TributeRef`

**Declarations:**

`fn cause_class(cause: &str) -> &'static str`

---

## web/src/components/timeline/cards/item_card.rs

**Language:** Rust | **Size:** 2.4 KB | **Lines:** 86

**Imports:**
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `shared::messages::{AreaRef, GameMessage, ItemRef, MessagePayload}`

**Declarations:**

`fn ItemLink(game_identifier: String, item: ItemRef) -> Element`

`fn AreaLink(game_identifier: String, area: AreaRef) -> Element`

---

## web/src/components/timeline/cards/mod.rs

**Language:** Rust | **Size:** 257 B | **Lines:** 12

**Declarations:**

---

## web/src/components/timeline/cards/movement_card.rs

**Language:** Rust | **Size:** 2.0 KB | **Lines:** 65

**Imports:**
- `crate::routes::Routes`
- `dioxus::prelude::*`
- `shared::messages::{AreaRef, GameMessage, MessagePayload}`

**Declarations:**

`fn AreaLink(game_identifier: String, area: AreaRef) -> Element`

---

## web/src/components/timeline/cards/sleep_card.rs

**Language:** Rust | **Size:** 1.2 KB | **Lines:** 47

**Imports:**
- `dioxus::prelude::*`
- `shared::messages::{GameMessage, MessagePayload, Phase}`

**Declarations:**

`fn phase_icon(phase: Phase) -> &'static str`

---

## web/src/components/timeline/cards/stamina_card.rs

**Language:** Rust | **Size:** 2.2 KB | **Lines:** 73

**Imports:**
- `dioxus::prelude::*`
- `shared::messages::{GameMessage, MessagePayload, StaminaBand}`

**Declarations:**

`enum Direction`
> Variants: `Worsening`, `Recovery`, `Unknown`

`fn transition_direction(from: StaminaBand, to: StaminaBand) -> Direction`

---

## web/src/components/timeline/cards/state_card.rs

**Language:** Rust | **Size:** 1.3 KB | **Lines:** 40

**Imports:**
- `dioxus::prelude::*`
- `shared::messages::{GameMessage, MessagePayload}`

**Declarations:**

---

## web/src/components/timeline/cards/survival_card.rs

**Language:** Rust | **Size:** 3.8 KB | **Lines:** 114

**Imports:**
- `dioxus::prelude::*`
- `shared::messages::{GameMessage, HungerBand, MessagePayload, ThirstBand}`

**Declarations:**

`fn hunger_class(band: HungerBand) -> &'static str`

`fn thirst_class(band: ThirstBand) -> &'static str`

---

## web/src/components/timeline/cards/wake_card.rs

**Language:** Rust | **Size:** 2.1 KB | **Lines:** 73

**Imports:**
- `dioxus::prelude::*`
- `shared::messages::{
    AreaEventKind, GameMessage, InterruptionKind, MessagePayload, Phase, WakeReason,
}`

**Declarations:**

`fn phase_icon(phase: Phase) -> &'static str`

`fn area_event_label(kind: AreaEventKind) -> &'static str`

---

## web/src/components/timeline/event_card.rs

**Language:** Rust | **Size:** 3.5 KB | **Lines:** 79

**Imports:**
- `crate::components::timeline::cards::{
    alliance_card::AllianceCard, combat_card::CombatCard, combat_swing_card::CombatSwingCard,
    cycle_card::CycleCard, death_card::DeathCard, item_card::ItemCard, movement_card::MovementCard,
    sleep_card::SleepCard, stamina_card::StaminaCard, state_card::StateCard,
    survival_card::SurvivalCard, wake_card::WakeCard,
}`
- `dioxus::prelude::*`
- `shared::messages::{GameMessage, MessageKind, MessagePayload}`

**Declarations:**

---

## web/src/components/timeline/filters.rs

**Language:** Rust | **Size:** 8.4 KB | **Lines:** 251

**Imports:**
- `gloo_storage::Storage`
- `shared::messages::MessageKind`
- `std::collections::{HashMap, HashSet}`

**Declarations:**

**`impl FilterMode`**
  `pub fn matches(&self, kind: MessageKind) -> bool`

  `pub fn is_all(&self) -> bool`

  `pub fn to_query_value(&self) -> String`

  `pub fn from_query_value(raw: &str) -> Self`


`fn message_kind_slug(kind: MessageKind) -> Option<&'static str>`

`fn message_kind_from_slug(slug: &str) -> Option<MessageKind>`

**`impl PeriodFilters`**
  `pub fn filter_for(&self, game_id: &str) -> FilterMode`

  `pub fn set_filter(&mut self, game_id: &str, mode: FilterMode)`

  `pub fn tribute_filter(&self, game_id: &str) -> Option<String>`

  `pub fn set_tribute_filter(&mut self, game_id: &str, tribute_id: Option<String>)`

  `pub fn hydrate(&mut self, game_id: &str)`

  `pub fn generation(&self, game_id: &str) -> u32`

  `pub fn bump(&mut self, game_id: &str)`


`struct SerializableFilter`
> Fields: `mode: String`, `kinds: Vec<MessageKind>`

**`impl From<&FilterMode> for SerializableFilter`**
  `fn from(m: &FilterMode) -> Self`


**`impl From<SerializableFilter> for FilterMode`**
  `fn from(s: SerializableFilter) -> Self`


`mod tests`

---

## web/src/components/timeline/mod.rs

**Language:** Rust | **Size:** 211 B | **Lines:** 9

**Imports:**
- `pub use event_card::EventCard`
- `pub use filters::{FilterMode, PeriodFilters}`
- `pub use timeline::Timeline`

**Declarations:**

---

## web/src/components/timeline/timeline.rs

**Language:** Rust | **Size:** 1.7 KB | **Lines:** 50

**Imports:**
- `crate::components::timeline::FilterMode`
- `crate::components::timeline::event_card::EventCard`
- `dioxus::prelude::*`
- `shared::messages::GameMessage`

**Declarations:**

---

## web/src/components/tribute_delete.rs

**Language:** Rust | **Size:** 2.8 KB | **Lines:** 97

**Imports:**
- `crate::cache::MutationError`
- `crate::components::game_tributes::GameTributesQ`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `game::games::GAME`
- `shared::DeleteTribute`

**Declarations:**

`pub(crate) struct DeleteTributeM`

**`impl MutationCapability for DeleteTributeM`**
  `async fn run(&self, name: &String) -> Result<String, MutationError>`

  `async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>)`


---

## web/src/components/tribute_detail.rs

**Language:** Rust | **Size:** 19.2 KB | **Lines:** 542

**Imports:**
- `crate::cache::QueryError`
- `crate::components::game_tributes::GameTributesQ`
- `crate::components::icons::uturn::UTurnIcon`
- `crate::components::info_detail::InfoDetail`
- `crate::components::item_icon::ItemIcon`
- `crate::components::tribute_status_icon::TributeStatusIcon`
- `crate::components::tribute_survival_section::TributeSurvivalSection`
- `crate::http::WithCredentials`
- `crate::routes::Routes`
- `dioxus::prelude::*`
- *... and 5 more imports*

**Declarations:**

`pub(crate) struct TributeQ`

**`impl QueryCapability for TributeQ`**
  `async fn run(&self, keys: &(String, String)) -> Result<Box<Tribute>, QueryError>`


`pub(crate) struct TributeLogQ`

**`impl QueryCapability for TributeLogQ`**
  `async fn run(&self, keys: &(String, String)) -> Result<Vec<GameMessage>, QueryError>`


`pub(crate) fn trait_chip_classes(t: &Trait) -> &'static str`

`fn TributeLog(game_identifier: String, identifier: String) -> Element`

`fn TributeAttributes(attributes: Attributes) -> Element`

`fn TributeStaminaRow(current: u32, max: u32) -> Element`

`fn TributeTraits(traits: Vec<Trait>, turns_since_last_betrayal: u8) -> Element`

`fn TributeAllies(game_identifier: String, ally_ids: Vec<uuid::Uuid>) -> Element`

`mod tests`

---

## web/src/components/tribute_edit.rs

**Language:** Rust | **Size:** 8.6 KB | **Lines:** 287

**Imports:**
- `crate::cache::MutationError`
- `crate::components::game_tributes::GameTributesQ`
- `crate::components::icons::edit::EditIcon`
- `crate::components::modal::{Modal, Props as ModalProps}`
- `crate::components::tribute_detail::TributeQ`
- `crate::components::{Button, Input}`
- `crate::http::WithCredentials`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`
- `shared::EditTribute`

**Declarations:**

`pub(crate) struct EditTributeM`

**`impl MutationCapability for EditTributeM`**
  `async fn run(&self, args: &(EditTribute, String)) -> Result<String, MutationError>`

  `async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>)`


---

## web/src/components/tribute_filter_chips.rs

**Language:** Rust | **Size:** 3.1 KB | **Lines:** 84

**Imports:**
- `crate::components::game_tributes::GameTributesQ`
- `crate::components::timeline::PeriodFilters`
- `dioxus::prelude::*`
- `dioxus_query::prelude::*`

**Declarations:**

---

## web/src/components/tribute_state_strip.rs

**Language:** Rust | **Size:** 4.2 KB | **Lines:** 123

**Imports:**
- `dioxus::prelude::*`
- `game::tributes::Tribute`
- `game::tributes::survival::{HungerBand, ThirstBand, hunger_band, thirst_band}`
- `shared::messages::StaminaBand`

**Declarations:**

`fn stamina_band_local(stamina: u32, max_stamina: u32) -> StaminaBand`

`fn HungerPip(band: HungerBand, raw: u8) -> Element`

`fn ThirstPip(band: ThirstBand, raw: u8) -> Element`

`fn StaminaPip(band: StaminaBand, current: u32, max: u32) -> Element`

`fn ShelterPip(phases_left: u32) -> Element`

---

## web/src/components/tribute_status_icon.rs

**Language:** Rust | **Size:** 3.0 KB | **Lines:** 83

**Imports:**
- `crate::components::icons::svg_icon::SvgIcon`
- `dioxus::prelude::*`
- `game::tributes::statuses::TributeStatus`

**Declarations:**

`pub(crate) fn icon_name_for_status(status: &TributeStatus) -> String`

`mod tests`

---

## web/src/components/tribute_survival_section.rs

**Language:** Rust | **Size:** 2.4 KB | **Lines:** 68

**Imports:**
- `dioxus::prelude::*`
- `game::tributes::Tribute`
- `game::tributes::survival::{HungerBand, ThirstBand, hunger_band, thirst_band}`

**Declarations:**

---

## web/src/components/ui/button.rs

**Language:** Rust | **Size:** 2.3 KB | **Lines:** 77

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

**`impl ButtonVariant`**
  `pub fn classes(self) -> &'static str`


`mod tests`

---

## web/src/components/ui/event_card.rs

**Language:** Rust | **Size:** 1.0 KB | **Lines:** 34

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/ui/live_pill.rs

**Language:** Rust | **Size:** 381 B | **Lines:** 13

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/ui/mod.rs

**Language:** Rust | **Size:** 483 B | **Lines:** 19

**Imports:**
- `pub use button::{Button, ButtonVariant}`
- `pub use event_card::EventCard`
- `pub use live_pill::LivePill`
- `pub use scoreboard::Scoreboard`
- `pub use section_label::SectionLabel`
- `pub use sidebar_hud::{SidebarHud, StatTile}`
- `pub use ticker::{Ticker, TickerItem}`
- `pub use topbar::TopBar`
- `pub use tribute_row::TributeRow`

**Declarations:**

---

## web/src/components/ui/scoreboard.rs

**Language:** Rust | **Size:** 1.8 KB | **Lines:** 58

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

`fn Shield(code: String) -> Element`

---

## web/src/components/ui/section_label.rs

**Language:** Rust | **Size:** 259 B | **Lines:** 11

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/ui/sidebar_hud.rs

**Language:** Rust | **Size:** 1.1 KB | **Lines:** 38

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/ui/ticker.rs

**Language:** Rust | **Size:** 596 B | **Lines:** 23

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/ui/topbar.rs

**Language:** Rust | **Size:** 460 B | **Lines:** 16

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/components/ui/tribute_row.rs

**Language:** Rust | **Size:** 1.0 KB | **Lines:** 34

**Imports:**
- `dioxus::prelude::*`

**Declarations:**

---

## web/src/hooks/mod.rs

**Language:** Rust | **Size:** 149 B | **Lines:** 5

**Imports:**
- `pub use use_game_websocket::*`
- `pub(crate) use use_timeline_summary::use_timeline_summary`

**Declarations:**

---

## web/src/hooks/use_game_websocket.rs

**Language:** Rust | **Size:** 7.7 KB | **Lines:** 215

**Imports:**
- `crate::env::APP_API_HOST`
- `dioxus::prelude::*`
- `futures_util::{SinkExt, StreamExt}`
- `gloo_net::websocket::{Message, futures::WebSocket}`
- `shared::WebSocketMessage`
- `shared::messages::GameMessage`

**Declarations:**

`const MAX_EVENTS: usize = 200`

`pub(crate) fn build_ws_url(api_host: &str, _game_id: &str) -> String`

`fn same_origin_ws_url() -> String`

`fn same_origin_ws_url() -> String`

`mod tests`

---

## web/src/hooks/use_timeline_summary.rs

**Language:** Rust | **Size:** 1.3 KB | **Lines:** 40

**Imports:**
- `crate::cache::QueryError`
- `crate::http::WithCredentials`
- `dioxus_query::prelude::*`
- `reqwest::StatusCode`
- `shared::messages::TimelineSummary`

**Declarations:**

`pub(crate) struct TimelineSummaryQ`

**`impl QueryCapability for TimelineSummaryQ`**
  `async fn run(&self, id: &String) -> Result<TimelineSummary, QueryError>`


`pub(crate) fn use_timeline_summary(game_id: String) -> UseQuery<TimelineSummaryQ>`

---

## web/src/http.rs

**Language:** Rust | **Size:** 728 B | **Lines:** 23

**Imports:**
- `reqwest::RequestBuilder`

**Declarations:**

**`impl WithCredentials for RequestBuilder`**
  `fn with_credentials(self) -> Self`

  `fn with_credentials(self) -> Self`


---

## web/src/lib.rs

**Language:** Rust | **Size:** 335 B | **Lines:** 20

**Imports:**
- `serde::{Deserialize, Serialize}`

**Declarations:**

`mod cache`

`pub(crate) mod env`

`mod routes`

`mod storage`

---

## web/src/main.rs

**Language:** Rust | **Size:** 82 B | **Lines:** 6

**Imports:**
- `dioxus::prelude::*`
- `web::components::App`

**Declarations:**

`fn main()`

---

## web/src/routes.rs

**Language:** Rust | **Size:** 1.9 KB | **Lines:** 57

**Imports:**
- `crate::components::{
    Accounts, AccountsPage, AreaDetail, Credits, GamePage, GamePeriodPage, Games, GamesList, Home,
    IconsPage, ItemDetail, Navbar, TributeDetail,
}`
- `dioxus::prelude::*`

**Declarations:**

`fn PageNotFound(route: Vec<String>) -> Element`

---

## web/src/storage.rs

**Language:** Rust | **Size:** 2.0 KB | **Lines:** 70

**Imports:**
- `crate::theme::Theme`
- `dioxus::prelude::*`
- `gloo_storage::{LocalStorage, Storage}`
- `serde::de::DeserializeOwned`
- `serde::{Deserialize, Serialize}`

**Declarations:**

`struct StorageEntry<T>`
> Fields: `key: String`, `value: T`

**`impl<T> Clone for UsePersistent<T>`**
  `fn clone(&self) -> Self`


**`impl<T> Copy for UsePersistent<T>`**

**`impl<T: Serialize + DeserializeOwned + Clone + 'static> UsePersistent<T>`**
  `pub fn get(&self) -> T`

  `pub fn set(&mut self, value: T)`


**`impl AppState`**
  `pub fn set_theme(&mut self, theme: Theme)`


---

## web/src/theme.rs

**Language:** Rust | **Size:** 1.6 KB | **Lines:** 68

**Imports:**
- `serde::{Deserialize, Serialize}`
- `std::fmt::{Display, Formatter}`
- `std::str::FromStr`

**Declarations:**

**`impl Theme`**
  `pub fn toggle(self) -> Self`


**`impl Display for Theme`**
  `fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result`


**`impl FromStr for Theme`**
  `fn from_str(s: &str) -> Result<Self, Self::Err>`


`mod tests`

---

## web/tests/button_test.rs

**Language:** Rust | **Size:** 3.3 KB | **Lines:** 137

**Imports:**
- `dioxus::prelude::*`
- `web::components::{Button, ThemedButton}`

**Declarations:**

`fn test_button_renders_with_defaults()`

`fn test_button_renders_with_all_props()`

`fn test_button_with_onclick()`

`fn test_button_disabled_state()`

`fn test_themed_button_renders()`

`fn test_themed_button_with_extra_classes()`

`fn test_themed_button_disabled()`

`fn test_multiple_buttons_render()`

---

## web/tests/map_test.rs

**Language:** Rust | **Size:** 1.2 KB | **Lines:** 41

**Imports:**
- `dioxus::prelude::*`
- `game::areas::{Area, AreaDetails}`
- `web::components::Map`

**Declarations:**

`fn all_areas() -> Vec<AreaDetails>`

`struct MapProps`
> Fields: `areas: Vec<AreaDetails>`

`fn MapHarness(props: MapProps) -> Element`

`fn test_map_renders_with_all_seven_areas()`

`fn test_map_renders_when_passed_minimal_areas()`

---

## web/tests/mod.rs

**Language:** Rust | **Size:** 427 B | **Lines:** 12

**Declarations:**

`mod button_test`

---

## web/tests/modal_test.rs

**Language:** Rust | **Size:** 1.0 KB | **Lines:** 48

**Imports:**
- `dioxus::prelude::*`
- `web::components::Modal`
- `web::components::modal::Props as ModalProps`

**Declarations:**

`struct Harness`
> Fields: `open: bool`, `title: String`

`fn ModalHarness(p: Harness) -> Element`

`fn test_modal_renders_closed_with_no_children()`

`fn test_modal_renders_open_with_children()`

---

## web/tests/tribute_status_icon_test.rs

**Language:** Rust | **Size:** 989 B | **Lines:** 44

**Imports:**
- `dioxus::prelude::*`
- `game::tributes::statuses::TributeStatus`
- `web::components::TributeStatusIcon`

**Declarations:**

`struct Harness`
> Fields: `status: TributeStatus`, `css_class: String`

`fn IconHarness(p: Harness) -> Element`

`fn test_status_icon_renders_for_healthy()`

`fn test_status_icon_renders_for_dead()`

---

## web/web/assets/icons.svg

**Language:** XML | **Size:** 69 B | **Lines:** 1

**Declarations:**

