
use sqlx::SqlitePool;
#[derive(serde::Deserialize)]
pub struct SignalQuery {
    pub ticker: Option<String>,
    pub limit: Option<i64>,
}

pub async fn get_signals(
    axum::extract::State(pool): axum::extract::State<SqlitePool>,
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