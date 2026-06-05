
use sqlx::SqlitePool;


pub async fn get_signals(
    axum::extract::State(pool): axum::extract::State<SqlitePool>
) -> axum::response::Json<serde_json::Value> {
    let rows = sqlx::query_as::<_, (i64, String, String, f64, String)>(
        "SELECT id, title, published_at, score, reason FROM signals ORDER BY scored_at DESC"
    )
    .fetch_all(&pool)
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