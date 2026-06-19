use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::{collections::{BTreeMap, BTreeSet}, env, net::SocketAddr, time::Duration};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db_pool: Option<PgPool>,
}

#[derive(Debug, Clone)]
struct AuthPlayer {
    id: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Serialize)]
struct DbHealthResponse {
    status: &'static str,
    database: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    status: &'static str,
    error: String,
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct UpdateProfileRequest {
    display_name: Option<String>,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    player: PlayerResponse,
}

#[derive(Serialize)]
struct PlayerResponse {
    id: String,
    username: Option<String>,
    display_name: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct PublicPlayerResponse {
    id: String,
    username: Option<String>,
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct PlayerSearchQuery {
    username: String,
}

#[derive(Deserialize)]
struct CreateGameRequest {
    client_game_type: Option<String>,
    opponent_player_id: Option<String>,
    initial_state: Option<Value>,
}

#[derive(Deserialize)]
struct SubmitTurnRequest {
    client_turn_id: Option<String>,
    turn_payload: Value,
    resulting_state: Option<Value>,
    resulting_status: Option<String>,
}

#[derive(Deserialize)]
struct CreateGameInviteRequest {
    invitee_player_id: Option<String>,
    invitee_player_ids: Option<Vec<String>>,
    client_game_type: Option<String>,
    initial_state: Option<Value>,
}

#[derive(Serialize)]
struct CreateGameInvitesResponse {
    game: GameResponse,
    invites: Vec<GameInviteResponse>,
}

#[derive(Serialize)]
struct GameInviteResponse {
    id: String,
    inviter_player: PublicPlayerResponse,
    invitee_player: PublicPlayerResponse,
    client_game_type: String,
    status: String,
    initial_state: Value,
    game_id: Option<String>,
    created_at: DateTime<Utc>,
    responded_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct GameResponse {
    id: String,
    owner_player_id: String,
    host_player: PublicPlayerResponse,
    client_game_type: String,
    status: String,
    state: Value,
    invited_player_count: i64,
    accepted_invited_player_count: i64,
    host_can_start: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct GameWithTurnsResponse {
    game: GameResponse,
    participants: Vec<PublicPlayerResponse>,
    turns: Vec<TurnResponse>,
}

#[derive(Serialize)]
struct TurnResponse {
    id: String,
    game_id: String,
    player_id: String,
    client_turn_id: Option<String>,
    sequence_no: i32,
    turn_payload: Value,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct DeletedGameResponse {
    game_id: String,
}

#[derive(Default)]
struct ExactGroupGameCandidate {
    invitee_ids: BTreeSet<String>,
    status: String,
    has_non_host_participants: bool,
    has_turns: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "friendbox=info,tower_http=info".into()),
        )
        .init();

    let bind_addr = env::var("FRIENDBOX_BIND_ADDR")
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let db_pool = create_db_pool().await;
    if let Some(pool) = db_pool.clone() {
        tokio::spawn(async move {
            run_game_cleanup_loop(pool).await;
        });
    }
    let state = AppState { db_pool };

    let app = Router::new()
        .route("/health", get(health))
        .route("/db-health", get(db_health))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/me", get(me).patch(update_profile).delete(delete_me))
        .route("/api/v1/players/search", get(search_players))
        .route("/api/v1/games", get(list_games).post(create_game))
        .route("/api/v1/games/{game_id}", get(get_game).delete(delete_game))
        .route("/api/v1/games/{game_id}/start", post(start_game))
        .route("/api/v1/games/{game_id}/turns", post(submit_turn))
        .route(
            "/api/v1/game-invites",
            get(list_game_invites).post(create_game_invite),
        )
        .route(
            "/api/v1/game-invites/{invite_id}/accept",
            post(accept_game_invite),
        )
        .route(
            "/api/v1/game-invites/{invite_id}/decline",
            post(decline_game_invite),
        )
        .with_state(state);

    let addr: SocketAddr = bind_addr
        .parse()
        .unwrap_or_else(|err| panic!("Invalid FRIENDBOX_BIND_ADDR '{}': {}", bind_addr, err));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to bind {}: {}", addr, err));

    info!("FriendBox API listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("FriendBox API server failed");
}

const STALE_GAME_RETENTION_DAYS: i64 = 14;
const GAME_CLEANUP_INTERVAL_SECS: u64 = 60 * 60;
const GAME_SELECT_WITH_PROGRESS: &str = r#"
    SELECT
        g.id::text,
        g.owner_player_id::text,
        p.id::text AS host_player_id,
        p.username AS host_player_username,
        p.display_name AS host_player_display_name,
        g.client_game_type,
        g.status,
        g.state,
        COALESCE(inv.invited_player_count, 0) AS invited_player_count,
        COALESCE(inv.accepted_invited_player_count, 0) AS accepted_invited_player_count,
        (
            g.status = 'pending'
            AND COALESCE(inv.accepted_invited_player_count, 0) > 0
        ) AS host_can_start,
        g.created_at,
        g.updated_at
    FROM games g
    INNER JOIN players p ON p.id = g.owner_player_id
    LEFT JOIN (
        SELECT
            gi.game_id,
            COUNT(*)::bigint AS invited_player_count,
            COUNT(*) FILTER (WHERE gi.status = 'accepted')::bigint AS accepted_invited_player_count
        FROM game_invites gi
        WHERE gi.game_id IS NOT NULL
        GROUP BY gi.game_id
    ) inv ON inv.game_id = g.id
"#;

async fn create_db_pool() -> Option<PgPool> {
    let database_url = match env::var("DATABASE_URL") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => {
            info!("DATABASE_URL is not set; /db-health will report not_configured");
            return None;
        }
    };

    match PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            info!("Connected to FriendBox database");
            Some(pool)
        }
        Err(err) => {
            error!("Failed to connect to FriendBox database: {}", err);
            None
        }
    }
}

async fn run_game_cleanup_loop(pool: PgPool) {
    if let Err(err) = cleanup_stale_games(&pool).await {
        error!("Initial game cleanup failed: {}", err);
    }

    let mut interval = tokio::time::interval(Duration::from_secs(GAME_CLEANUP_INTERVAL_SECS));
    loop {
        interval.tick().await;
        if let Err(err) = cleanup_stale_games(&pool).await {
            error!("Scheduled game cleanup failed: {}", err);
        }
    }
}

async fn cleanup_stale_games(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let stale_game_ids = sqlx::query_scalar::<_, String>(
        r#"
        SELECT id::text
        FROM games
        WHERE updated_at < now() - make_interval(days => $1)
        "#,
    )
    .bind(STALE_GAME_RETENTION_DAYS)
    .fetch_all(pool)
    .await?;

    if stale_game_ids.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;
    let mut deleted = 0u64;
    for game_id in stale_game_ids {
        if delete_game_records(&mut tx, &game_id).await? {
            deleted += 1;
        }
    }
    tx.commit().await?;

    if deleted > 0 {
        info!("Cleaned up {} stale game(s)", deleted);
    }

    Ok(deleted)
}

async fn delete_game_records(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    game_id: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM game_invites
        WHERE game_id = $1::uuid
        "#,
    )
    .bind(game_id)
    .execute(&mut **tx)
    .await?;

    let deleted = sqlx::query(
        r#"
        DELETE FROM games
        WHERE id = $1::uuid
        "#,
    )
    .bind(game_id)
    .execute(&mut **tx)
    .await?;

    Ok(deleted.rows_affected() > 0)
}

fn normalize_game_status(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
}

fn is_valid_game_status(status: &str) -> bool {
    matches!(status, "completed")
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "friendbox",
    })
}

async fn db_health(State(state): State<AppState>) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "database_not_configured_or_unavailable",
        );
    };

    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
    {
        Ok(1) => Json(DbHealthResponse {
            status: "ok",
            database: "connected",
        })
        .into_response(),
        Ok(value) => api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            &format!("unexpected_database_probe_result:{value}"),
        ),
        Err(err) => api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    }
}

async fn register(State(state): State<AppState>, Json(payload): Json<RegisterRequest>) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };

    let Some(username) = normalize_username(&payload.username) else {
        return api_error(StatusCode::BAD_REQUEST, "username_required");
    };
    if payload.password.len() < 8 {
        return api_error(StatusCode::BAD_REQUEST, "password_must_be_at_least_8_chars");
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(err) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, &err),
    };

    let player_id = Uuid::new_v4().to_string();
    let display_name =
        clean_optional_string(payload.display_name).or_else(|| Some(username.clone()));
    let token = format!("fb_{}_{}", Uuid::new_v4(), Uuid::new_v4());

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let player_row = match sqlx::query(
        r#"
        INSERT INTO players (id, username, password_hash, display_name)
        VALUES ($1::uuid, $2, $3, $4)
        RETURNING id::text, username, display_name, created_at, updated_at
        "#,
    )
    .bind(player_id)
    .bind(&username)
    .bind(password_hash)
    .bind(display_name)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(err) if is_unique_violation(&err) => {
            return api_error(StatusCode::CONFLICT, "username_already_taken");
        }
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    if let Err(err) = create_session(&mut tx, &token, player_row.get("id")).await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    if let Err(err) = tx.commit().await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    (
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            player: player_from_row(&player_row),
        }),
    )
        .into_response()
}

async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };

    let Some(username) = normalize_username(&payload.username) else {
        return api_error(StatusCode::BAD_REQUEST, "username_required");
    };

    let player_row = match sqlx::query(
        r#"
        SELECT id::text, username, password_hash, display_name, created_at, updated_at
        FROM players
        WHERE username = $1
        "#,
    )
    .bind(&username)
    .fetch_one(pool)
    .await
    {
        Ok(row) => row,
        Err(_) => return api_error(StatusCode::UNAUTHORIZED, "invalid_username_or_password"),
    };

    let password_hash: Option<String> = player_row.get("password_hash");
    let Some(password_hash) = password_hash else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_username_or_password");
    };
    if !verify_password(&payload.password, &password_hash) {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_username_or_password");
    }

    let token = format!("fb_{}_{}", Uuid::new_v4(), Uuid::new_v4());
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    if let Err(err) = create_session(&mut tx, &token, player_row.get("id")).await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }
    if let Err(err) = tx.commit().await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    Json(AuthResponse {
        token,
        player: player_from_row(&player_row),
    })
    .into_response()
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Some(token) = bearer_token(&headers) else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    match sqlx::query(
        r#"
        UPDATE player_sessions
        SET revoked_at = now()
        WHERE token = $1 AND revoked_at IS NULL
        RETURNING player_id
        "#,
    )
    .bind(token)
    .fetch_one(pool)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token"),
    }
}

async fn me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    match sqlx::query(
        r#"
        SELECT id::text, username, display_name, created_at, updated_at
        FROM players
        WHERE id = $1::uuid
        "#,
    )
    .bind(auth.id)
    .fetch_one(pool)
    .await
    {
        Ok(row) => Json(player_from_row(&row)).into_response(),
        Err(err) => api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    }
}

async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateProfileRequest>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    let Some(display_name) = clean_optional_string(payload.display_name) else {
        return api_error(StatusCode::BAD_REQUEST, "display_name_required");
    };
    if display_name.len() > 64 {
        return api_error(StatusCode::BAD_REQUEST, "display_name_too_long");
    }

    match sqlx::query(
        r#"
        UPDATE players
        SET display_name = $2, updated_at = now()
        WHERE id = $1::uuid
        RETURNING id::text, username, display_name, created_at, updated_at
        "#,
    )
    .bind(auth.id)
    .bind(display_name)
    .fetch_one(pool)
    .await
    {
        Ok(row) => Json(player_from_row(&row)).into_response(),
        Err(err) => api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    }
}

async fn delete_me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    match sqlx::query(
        r#"
        DELETE FROM players
        WHERE id = $1::uuid
        RETURNING id
        "#,
    )
    .bind(auth.id)
    .fetch_one(pool)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => api_error(StatusCode::NOT_FOUND, "player_not_found"),
    }
}

async fn create_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateGameRequest>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    let client_game_type = payload
        .client_game_type
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "friendbox".to_string());
    let initial_state = payload.initial_state.unwrap_or_else(|| json!({}));

    let game_id = Uuid::new_v4().to_string();
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    match sqlx::query(
        r#"
        INSERT INTO games (id, owner_player_id, client_game_type, state)
        VALUES ($1::uuid, $2::uuid, $3, $4)
        RETURNING id::text, owner_player_id::text, client_game_type, status, state, created_at, updated_at
        "#,
    )
    .bind(&game_id)
    .bind(&auth.id)
    .bind(&client_game_type)
    .bind(initial_state)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(_) => {}
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    if let Err(err) = add_participant(&mut tx, &game_id, &auth.id).await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    if clean_optional_string(payload.opponent_player_id).is_some() {
        return api_error(
            StatusCode::BAD_REQUEST,
            "use_game_invites_when_adding_an_opponent",
        );
    }

    if let Err(err) = tx.commit().await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    let game = match load_game_response(pool, &game_id).await {
        Ok(game) => game,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    (StatusCode::CREATED, Json(game)).into_response()
}

async fn search_players(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PlayerSearchQuery>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };
    let Some(username) = normalize_username(&query.username) else {
        return api_error(StatusCode::BAD_REQUEST, "username_required");
    };

    match sqlx::query(
        r#"
        SELECT id::text, username, display_name
        FROM players
        WHERE username LIKE $1
          AND id <> $2::uuid
        ORDER BY username ASC
        LIMIT 10
        "#,
    )
    .bind(format!("{username}%"))
    .bind(auth.id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            Json(rows.iter().map(public_player_from_row).collect::<Vec<_>>()).into_response()
        }
        Err(err) => api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    }
}

async fn list_games(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    let query = format!(
        r#"
        {}
        INNER JOIN game_participants gp ON gp.game_id = g.id
        WHERE gp.player_id = $1::uuid
        ORDER BY g.updated_at DESC
        "#,
        GAME_SELECT_WITH_PROGRESS
    );

    match sqlx::query(&query)
    .bind(auth.id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => Json(rows.iter().map(game_from_row).collect::<Vec<_>>()).into_response(),
        Err(err) => api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    }
}

async fn get_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(game_id): Path<String>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };
    if !is_game_participant(pool, &game_id, &auth.id).await {
        return api_error(StatusCode::NOT_FOUND, "game_not_found");
    }

    let game = match load_game_response(pool, &game_id).await {
        Ok(game) => game,
        Err(sqlx::Error::RowNotFound) => return api_error(StatusCode::NOT_FOUND, "game_not_found"),
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let turn_rows = match sqlx::query(
        r#"
        SELECT id::text, game_id::text, player_id::text, client_turn_id, sequence_no, turn_payload, created_at
        FROM turns
        WHERE game_id = $1::uuid
        ORDER BY sequence_no ASC
        "#,
    )
    .bind(&game_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let participants = match game_participants(pool, &game_id).await {
        Ok(participants) => participants,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    Json(GameWithTurnsResponse {
        game,
        participants,
        turns: turn_rows.iter().map(turn_from_row).collect(),
    })
    .into_response()
}

async fn delete_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(game_id): Path<String>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };
    if !is_game_participant(pool, &game_id, &auth.id).await {
        return api_error(StatusCode::NOT_FOUND, "game_not_found");
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let deleted_game_id = match delete_game_records(&mut tx, &game_id).await {
        Ok(true) => game_id,
        Ok(false) => return api_error(StatusCode::NOT_FOUND, "game_not_found"),
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    if let Err(err) = tx.commit().await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    Json(DeletedGameResponse {
        game_id: deleted_game_id.to_string(),
    })
    .into_response()
}

async fn start_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(game_id): Path<String>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    let participant_count = match sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM game_participants gp
        INNER JOIN games g ON g.id = gp.game_id
        WHERE gp.game_id = $1::uuid
          AND g.owner_player_id = $2::uuid
        "#,
    )
    .bind(&game_id)
    .bind(&auth.id)
    .fetch_one(pool)
    .await
    {
        Ok(count) => count,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    if participant_count == 0 {
        return api_error(StatusCode::NOT_FOUND, "game_not_found");
    }
    if participant_count < 2 {
        return api_error(StatusCode::BAD_REQUEST, "not_enough_players");
    }

    match sqlx::query(
        r#"
        UPDATE games
        SET status = 'active', updated_at = now()
        WHERE id = $1::uuid
          AND owner_player_id = $2::uuid
          AND status = 'pending'
        "#,
    )
    .bind(&game_id)
    .bind(&auth.id)
    .execute(pool)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => {}
        Err(_) => return api_error(StatusCode::BAD_REQUEST, "game_not_pending"),
        Ok(_) => return api_error(StatusCode::BAD_REQUEST, "game_not_pending"),
    };

    match load_game_response(pool, &game_id).await {
        Ok(game) => Json(game).into_response(),
        Err(sqlx::Error::RowNotFound) => api_error(StatusCode::NOT_FOUND, "game_not_found"),
        Err(err) => api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    }
}

async fn submit_turn(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(game_id): Path<String>,
    Json(payload): Json<SubmitTurnRequest>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };
    if !is_game_participant(pool, &game_id, &auth.id).await {
        return api_error(StatusCode::NOT_FOUND, "game_not_found");
    };
    if !is_game_active(pool, &game_id).await {
        return api_error(StatusCode::BAD_REQUEST, "game_not_active");
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let sequence_no = match sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COALESCE(MAX(sequence_no), 0) + 1
        FROM turns
        WHERE game_id = $1::uuid
        "#,
    )
    .bind(&game_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(value) => value,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let turn_id = Uuid::new_v4().to_string();
    let turn_row = match sqlx::query(
        r#"
        INSERT INTO turns (id, game_id, player_id, client_turn_id, sequence_no, turn_payload)
        VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6)
        RETURNING id::text, game_id::text, player_id::text, client_turn_id, sequence_no, turn_payload, created_at
        "#,
    )
    .bind(&turn_id)
    .bind(&game_id)
    .bind(&auth.id)
    .bind(clean_optional_string(payload.client_turn_id))
    .bind(sequence_no)
    .bind(payload.turn_payload)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let resulting_status = normalize_game_status(payload.resulting_status.as_deref());
    if resulting_status
        .as_deref()
        .is_some_and(|status| !is_valid_game_status(status))
    {
        return api_error(StatusCode::BAD_REQUEST, "invalid_game_status");
    }

    if let Some(resulting_state) = payload.resulting_state {
        if let Err(err) = sqlx::query(
            r#"
            UPDATE games
            SET state = $2, updated_at = now()
            WHERE id = $1::uuid
            "#,
        )
        .bind(&game_id)
        .bind(resulting_state)
        .execute(&mut *tx)
        .await
        {
            return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
        }
    } else if let Err(err) = sqlx::query(
        r#"
        UPDATE games
        SET updated_at = now()
        WHERE id = $1::uuid
        "#,
    )
    .bind(&game_id)
    .execute(&mut *tx)
    .await
    {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    if let Some(status) = resulting_status {
        if let Err(err) = sqlx::query(
            r#"
            UPDATE games
            SET status = $2, updated_at = now()
            WHERE id = $1::uuid
            "#,
        )
        .bind(&game_id)
        .bind(&status)
        .execute(&mut *tx)
        .await
        {
            return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
        }

        if status == "completed" {
            if let Err(err) = delete_game_records(&mut tx, &game_id).await {
                return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
            }
        }
    }

    if let Err(err) = tx.commit().await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    (StatusCode::CREATED, Json(turn_from_row(&turn_row))).into_response()
}

async fn create_game_invite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateGameInviteRequest>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    let invitee_player_ids =
        match collect_invitee_player_ids(payload.invitee_player_id, payload.invitee_player_ids) {
            Ok(ids) => ids,
            Err(error) => return api_error(StatusCode::BAD_REQUEST, error),
        };
    if invitee_player_ids.iter().any(|id| id == &auth.id) {
        return api_error(StatusCode::BAD_REQUEST, "cannot_invite_self");
    }
    for invitee_player_id in &invitee_player_ids {
        if !player_exists(pool, invitee_player_id).await {
            return api_error(StatusCode::NOT_FOUND, "invitee_not_found");
        }
    }

    let client_game_type = payload
        .client_game_type
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "friendbox".to_string());
    let initial_state = payload.initial_state.unwrap_or_else(|| json!({}));

    let game_id = Uuid::new_v4().to_string();
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let requested_invitee_set: BTreeSet<String> = invitee_player_ids.iter().cloned().collect();
    let exact_group_rows = match sqlx::query(
        r#"
        SELECT
            g.id::text AS game_id,
            g.status,
            gi.invitee_player_id::text AS invitee_player_id,
            EXISTS (
                SELECT 1
                FROM game_participants gp
                WHERE gp.game_id = g.id
                  AND gp.player_id <> g.owner_player_id
            ) AS has_non_host_participants,
            EXISTS (
                SELECT 1
                FROM turns t
                WHERE t.game_id = g.id
            ) AS has_turns
        FROM games g
        LEFT JOIN game_invites gi
            ON gi.game_id = g.id
        WHERE g.owner_player_id = $1::uuid
          AND g.client_game_type = $2
        ORDER BY g.updated_at DESC, g.created_at DESC
        "#,
    )
    .bind(&auth.id)
    .bind(&client_game_type)
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let mut exact_group_candidates: BTreeMap<String, ExactGroupGameCandidate> = BTreeMap::new();
    for row in exact_group_rows {
        let candidate_game_id: String = row.get("game_id");
        let entry = exact_group_candidates.entry(candidate_game_id).or_default();
        entry.status = row.get("status");
        entry.has_non_host_participants = row.get("has_non_host_participants");
        entry.has_turns = row.get("has_turns");
        if let Some(invitee_id) = row.get::<Option<String>, _>("invitee_player_id") {
            entry.invitee_ids.insert(invitee_id);
        }
    }

    let mut replaceable_game_ids = Vec::new();
    for (candidate_game_id, candidate) in exact_group_candidates {
        if candidate.invitee_ids != requested_invitee_set {
            continue;
        }

        let is_replaceable_pending_lobby = candidate.status == "pending"
            && !candidate.has_non_host_participants
            && !candidate.has_turns;

        if is_replaceable_pending_lobby {
            replaceable_game_ids.push(candidate_game_id);
            continue;
        }

        return api_error(
            StatusCode::CONFLICT,
            "exact_group_game_already_exists",
        );
    }

    for existing_game_id in replaceable_game_ids {
        if let Err(err) = delete_game_records(&mut tx, &existing_game_id).await {
            return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
        }
    }

    match sqlx::query(
        r#"
        INSERT INTO games (id, owner_player_id, client_game_type, status, state)
        VALUES ($1::uuid, $2::uuid, $3, 'pending', $4)
        RETURNING id::text, owner_player_id::text, client_game_type, status, state, created_at, updated_at
        "#,
    )
    .bind(&game_id)
    .bind(&auth.id)
    .bind(&client_game_type)
    .bind(&initial_state)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(_) => {}
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    if let Err(err) = add_participant(&mut tx, &game_id, &auth.id).await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    let mut invite_rows = Vec::with_capacity(invitee_player_ids.len());
    for invitee_player_id in invitee_player_ids {
        let invite_id = Uuid::new_v4().to_string();
        let row = match sqlx::query(
        r#"
        INSERT INTO game_invites (id, inviter_player_id, invitee_player_id, client_game_type, initial_state, game_id)
        VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6::uuid)
        RETURNING id::text, inviter_player_id::text, invitee_player_id::text, client_game_type, status, initial_state, game_id::text, created_at, responded_at
        "#,
    )
    .bind(invite_id)
    .bind(&auth.id)
    .bind(&invitee_player_id)
    .bind(&client_game_type)
    .bind(&initial_state)
    .bind(&game_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };
        invite_rows.push(row);
    }

    if let Err(err) = tx.commit().await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    let game = match load_game_response(pool, &game_id).await {
        Ok(game) => game,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let mut invites = Vec::with_capacity(invite_rows.len());
    for row in invite_rows {
        match invite_from_row(pool, &row).await {
            Ok(invite) => invites.push(invite),
            Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
        }
    }

    (
        StatusCode::CREATED,
        Json(CreateGameInvitesResponse {
            game,
            invites,
        }),
    )
        .into_response()
}

async fn list_game_invites(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    let rows = match sqlx::query(
        r#"
        SELECT id::text, inviter_player_id::text, invitee_player_id::text, client_game_type, status, initial_state, game_id::text, created_at, responded_at
        FROM game_invites
        WHERE invitee_player_id = $1::uuid AND status = 'pending'
        ORDER BY created_at DESC
        "#,
    )
    .bind(auth.id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let mut invites = Vec::with_capacity(rows.len());
    for row in rows {
        match invite_from_row(pool, &row).await {
            Ok(invite) => invites.push(invite),
            Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
        }
    }

    Json(invites).into_response()
}

async fn accept_game_invite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(invite_id): Path<String>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let invite_row = match sqlx::query(
        r#"
        SELECT id::text, inviter_player_id::text, invitee_player_id::text, client_game_type, status, initial_state, game_id::text, created_at, responded_at
        FROM game_invites
        WHERE id = $1::uuid AND invitee_player_id = $2::uuid
        FOR UPDATE
        "#,
    )
    .bind(&invite_id)
    .bind(&auth.id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(_) => return api_error(StatusCode::NOT_FOUND, "invite_not_found"),
    };

    let status: String = invite_row.get("status");
    if status != "pending" {
        return api_error(StatusCode::BAD_REQUEST, "invite_not_pending");
    }

    let inviter_player_id: String = invite_row.get("inviter_player_id");
    let invitee_player_id: String = invite_row.get("invitee_player_id");
    let game_id: String = match invite_row.get::<Option<String>, _>("game_id") {
        Some(game_id) => game_id,
        None => {
            let legacy_game_id = Uuid::new_v4().to_string();
            let client_game_type: String = invite_row.get("client_game_type");
            let initial_state: Value = invite_row.get("initial_state");

            if let Err(err) = sqlx::query(
                r#"
                INSERT INTO games (id, owner_player_id, client_game_type, state)
                VALUES ($1::uuid, $2::uuid, $3, $4)
                "#,
            )
            .bind(&legacy_game_id)
            .bind(&inviter_player_id)
            .bind(&client_game_type)
            .bind(initial_state)
            .execute(&mut *tx)
            .await
            {
                return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
            }

            if let Err(err) = add_participant(&mut tx, &legacy_game_id, &inviter_player_id).await {
                return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
            }

            legacy_game_id
        }
    };

    if let Err(err) = add_participant(&mut tx, &game_id, &invitee_player_id).await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    if let Err(err) = sqlx::query(
        r#"
        UPDATE games
        SET updated_at = now()
        WHERE id = $1::uuid
        "#,
    )
    .bind(&game_id)
    .execute(&mut *tx)
    .await
    {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    let accepted_row = match sqlx::query(
        r#"
        UPDATE game_invites
        SET status = 'accepted', game_id = $2::uuid, responded_at = now()
        WHERE id = $1::uuid
        RETURNING id::text, inviter_player_id::text, invitee_player_id::text, client_game_type, status, initial_state, game_id::text, created_at, responded_at
        "#,
    )
    .bind(&invite_id)
    .bind(&game_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    if let Err(err) = tx.commit().await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    match invite_from_row(pool, &accepted_row).await {
        Ok(invite) => Json(invite).into_response(),
        Err(err) => api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    }
}

async fn decline_game_invite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(invite_id): Path<String>,
) -> Response {
    let Ok(pool) = require_db(&state) else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
    };
    let Ok(auth) = authenticate(pool, &headers).await else {
        return api_error(StatusCode::UNAUTHORIZED, "invalid_or_missing_token");
    };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    };

    let row = match sqlx::query(
        r#"
        UPDATE game_invites
        SET status = 'declined', responded_at = now()
        WHERE id = $1::uuid
          AND invitee_player_id = $2::uuid
          AND status = 'pending'
        RETURNING id::text, inviter_player_id::text, invitee_player_id::text, client_game_type, status, initial_state, game_id::text, created_at, responded_at
        "#,
    )
    .bind(&invite_id)
    .bind(&auth.id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(_) => return api_error(StatusCode::NOT_FOUND, "pending_invite_not_found"),
    };

    if let Some(game_id) = row.get::<Option<String>, _>("game_id") {
        let deleted = match sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM games
                WHERE id = $1::uuid
                  AND status = 'pending'
            )
            "#,
        )
        .bind(&game_id)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(value) => value,
            Err(err) => return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
        };

        if deleted {
            if let Err(err) = delete_game_records(&mut tx, &game_id).await {
                return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
            }
        }
    }

    if let Err(err) = tx.commit().await {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string());
    }

    match invite_from_row(pool, &row).await {
        Ok(invite) => Json(invite).into_response(),
        Err(err) => api_error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string()),
    }
}

fn require_db(state: &AppState) -> Result<&PgPool, ()> {
    state.db_pool.as_ref().ok_or(())
}

async fn authenticate(pool: &PgPool, headers: &HeaderMap) -> Result<AuthPlayer, ()> {
    let Some(token) = bearer_token(headers) else {
        return Err(());
    };

    let row = sqlx::query(
        r#"
        UPDATE player_sessions
        SET last_seen_at = now()
        WHERE token = $1 AND revoked_at IS NULL
        RETURNING player_id::text
        "#,
    )
    .bind(token)
    .fetch_one(pool)
    .await
    .map_err(|_| ())?;

    Ok(AuthPlayer {
        id: row.get("player_id"),
    })
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get("authorization")?.to_str().ok()?;
    value
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}

async fn create_session(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    token: &str,
    player_id: String,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO player_sessions (token, player_id)
        VALUES ($1, $2::uuid)
        "#,
    )
    .bind(token)
    .bind(player_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn add_participant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    game_id: &str,
    player_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO game_participants (game_id, player_id)
        VALUES ($1::uuid, $2::uuid)
        ON CONFLICT (game_id, player_id) DO NOTHING
        "#,
    )
    .bind(game_id)
    .bind(player_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn is_game_participant(pool: &PgPool, game_id: &str, player_id: &str) -> bool {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM game_participants
            WHERE game_id = $1::uuid AND player_id = $2::uuid
        )
        "#,
    )
    .bind(game_id)
    .bind(player_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

async fn is_game_active(pool: &PgPool, game_id: &str) -> bool {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM games
            WHERE id = $1::uuid AND status = 'active'
        )
        "#,
    )
    .bind(game_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

async fn game_participants(
    pool: &PgPool,
    game_id: &str,
) -> Result<Vec<PublicPlayerResponse>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT p.id::text, p.username, p.display_name
        FROM game_participants gp
        INNER JOIN players p ON p.id = gp.player_id
        WHERE gp.game_id = $1::uuid
        ORDER BY gp.created_at ASC
        "#,
    )
    .bind(game_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(public_player_from_row).collect())
}

async fn player_exists(pool: &PgPool, player_id: &str) -> bool {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM players
            WHERE id = $1::uuid
        )
        "#,
    )
    .bind(player_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

fn collect_invitee_player_ids(
    invitee_player_id: Option<String>,
    invitee_player_ids: Option<Vec<String>>,
) -> Result<Vec<String>, &'static str> {
    let mut ids = Vec::new();
    if let Some(invitee_player_id) = clean_optional_string(invitee_player_id) {
        ids.push(invitee_player_id);
    }
    if let Some(invitee_player_ids) = invitee_player_ids {
        for invitee_player_id in invitee_player_ids {
            if let Some(invitee_player_id) = clean_optional_string(Some(invitee_player_id)) {
                if !ids.iter().any(|existing| existing == &invitee_player_id) {
                    ids.push(invitee_player_id);
                }
            }
        }
    }

    if ids.is_empty() {
        return Err("invitee_player_id_required");
    }

    Ok(ids)
}

async fn public_player_by_id(
    pool: &PgPool,
    player_id: &str,
) -> Result<PublicPlayerResponse, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id::text, username, display_name
        FROM players
        WHERE id = $1::uuid
        "#,
    )
    .bind(player_id)
    .fetch_one(pool)
    .await?;

    Ok(public_player_from_row(&row))
}

async fn invite_from_row(
    pool: &PgPool,
    row: &sqlx::postgres::PgRow,
) -> Result<GameInviteResponse, sqlx::Error> {
    let inviter_player_id: String = row.get("inviter_player_id");
    let invitee_player_id: String = row.get("invitee_player_id");

    Ok(GameInviteResponse {
        id: row.get("id"),
        inviter_player: public_player_by_id(pool, &inviter_player_id).await?,
        invitee_player: public_player_by_id(pool, &invitee_player_id).await?,
        client_game_type: row.get("client_game_type"),
        status: row.get("status"),
        initial_state: row.get("initial_state"),
        game_id: row.get("game_id"),
        created_at: row.get("created_at"),
        responded_at: row.get("responded_at"),
    })
}

async fn load_game_response(pool: &PgPool, game_id: &str) -> Result<GameResponse, sqlx::Error> {
    let query = format!(
        r#"
        {}
        WHERE g.id = $1::uuid
        "#,
        GAME_SELECT_WITH_PROGRESS
    );

    let row = sqlx::query(&query).bind(game_id).fetch_one(pool).await?;
    Ok(game_from_row(&row))
}

fn player_from_row(row: &sqlx::postgres::PgRow) -> PlayerResponse {
    PlayerResponse {
        id: row.get("id"),
        username: row.try_get("username").ok(),
        display_name: row.get("display_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn public_player_from_row(row: &sqlx::postgres::PgRow) -> PublicPlayerResponse {
    PublicPlayerResponse {
        id: row.get("id"),
        username: row.get("username"),
        display_name: row.get("display_name"),
    }
}

fn game_from_row(row: &sqlx::postgres::PgRow) -> GameResponse {
    GameResponse {
        id: row.get("id"),
        owner_player_id: row.get("owner_player_id"),
        host_player: PublicPlayerResponse {
            id: row.get("host_player_id"),
            username: row.get("host_player_username"),
            display_name: row.get("host_player_display_name"),
        },
        client_game_type: row.get("client_game_type"),
        status: row.get("status"),
        state: row.get("state"),
        invited_player_count: row.try_get("invited_player_count").unwrap_or(0),
        accepted_invited_player_count: row.try_get("accepted_invited_player_count").unwrap_or(0),
        host_can_start: row.try_get("host_can_start").unwrap_or(false),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn turn_from_row(row: &sqlx::postgres::PgRow) -> TurnResponse {
    TurnResponse {
        id: row.get("id"),
        game_id: row.get("game_id"),
        player_id: row.get("player_id"),
        client_turn_id: row.get("client_turn_id"),
        sequence_no: row.get("sequence_no"),
        turn_payload: row.get("turn_payload"),
        created_at: row.get("created_at"),
    }
}

fn clean_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_username(value: &str) -> Option<String> {
    let username = value.trim().to_ascii_lowercase();
    let is_valid = username.len() >= 3
        && username.len() <= 32
        && username
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-');

    if is_valid { Some(username) } else { None }
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| err.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|db_err| db_err.code())
        .as_deref()
        == Some("23505")
}

fn api_error(status: StatusCode, error: &str) -> Response {
    (
        status,
        Json(ErrorResponse {
            status: "error",
            error: error.to_string(),
        }),
    )
        .into_response()
}
