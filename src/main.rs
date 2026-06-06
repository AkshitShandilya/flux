

mod poller;
mod api;
mod config;
mod models;


use dotenvy::dotenv;
use sqlx::SqlitePool;
use crate::api::get_signals;




#[tokio::main]
async fn main() {
    dotenv().ok();
 let config = crate::config::Config::load();

 let mut seen:std::collections::HashSet<String> = std::collections::HashSet::new();
 
  

   let pool = SqlitePool::connect("sqlite://flux.db?mode=rwc").await.expect("failed to connect to db");
   sqlx::query(
    "CREATE TABLE IF NOT EXISTS signals (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        published_at TEXT,
        score REAL NOT NULL,
        reason TEXT,
        scored_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )"
)
.execute(&pool)
.await
.expect("failed to create table");
let pool_for_loop = pool.clone();
let pool_for_api = pool.clone();

let existing: Vec<String> = sqlx::query_scalar("SELECT title FROM signals")
    .fetch_all(&pool)
    .await
    .expect("failed to load seen headlines");

for title in existing {
    seen.insert(title);
}
    
tokio::spawn(async move {
    poller::run(pool_for_loop, seen, config.news_api_key, config.llm_key,config.tickers,config.poll_interval_secs).await;
});

let app = axum::Router::new()
    .route("/signals", axum::routing::get(get_signals))
    .with_state(pool_for_api);
 
let addr = format!("0.0.0.0:{}", config.port);
let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
axum::serve(listener, app).await.unwrap();

    
        
    
}
