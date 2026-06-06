
use sqlx::SqlitePool;
#[derive(serde::Deserialize)]
pub struct SignalQuery {
    pub ticker: Option<String>,
    pub limit: Option<i64>,
}
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub tx: tokio::sync::broadcast::Sender<String>,
}
pub async fn get_signals(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SignalQuery>,
) -> axum::response::Json<serde_json::Value> {
   let mut sql = String::from("SELECT id, title, published_at, score, reason FROM signals");

if let Some(ticker) = &query.ticker {
    sql.push_str(&format!(" WHERE title LIKE '%{}%'", ticker));
}

sql.push_str(" ORDER BY scored_at DESC");

if let Some(limit) = query.limit {
    sql.push_str(&format!(" LIMIT {}", limit));
}

let rows = sqlx::query_as::<_, (i64, String, String, f64, String)>(&sql)
    .fetch_all(&state.pool)
    .await
    .expect("failed to fetch signals");

    let signals: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.0,
            "title": row.1,
            "published_at": row.2,
            "score": row.3,
            "reason": row.4
        })
    }).collect();

    axum::response::Json(serde_json::json!({ "signals": signals }))
}
pub async fn ws_handler(
    ws: axum::extract::WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.tx.subscribe()))
}

async fn handle_socket(
    mut socket: axum::extract::ws::WebSocket,
    mut rx: tokio::sync::broadcast::Receiver<String>,
) {
    while let Ok(msg) = rx.recv().await {
        if socket
            .send(axum::extract::ws::Message::Text(msg.into()))
            .await
            .is_err()
        {
            break;
        }
    }
}