use reqwest;
use dotenvy::dotenv;
use sqlx::SqlitePool;
#[derive(serde::Deserialize)]
struct Article {
    title: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
}

#[derive(serde::Deserialize)]
struct NewsResponse {
    articles: Vec<Article>,
}

#[derive(serde::Deserialize)]
struct LLMResponse {
    choices: Vec<Choice>,
}

#[derive(serde::Deserialize)]
struct Choice {
    message: Message,
}

#[derive(serde::Deserialize)]
struct Message {
    content: String,
}

#[derive(serde::Deserialize)]
struct SentimentScore {
    score: Option<f64>,
    reason: String,
}






async fn get_signals(
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
#[tokio::main]
async fn main() {
    dotenv().ok();
 let api_key = std::env::var("NEWS_API_KEY").expect("NEWS_API_KEY not set");
    let url = format!(
    "https://newsapi.org/v2/everything?q=AAPL&apiKey={}",
    api_key
);
    let mut seen:std::collections::HashSet<String> = std::collections::HashSet::new();
    let llm_url = "https://api.groq.com/openai/v1/chat/completions";

    let llm_key = std::env::var("LLM_KEY").expect("LLM_KEY not set");

 
   let client = reqwest::Client::new();

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
    loop{
        

 let body = client
    .get(&url)
    .header("User-Agent", "flux-app/0.1")
    .send()
    .await
    .expect("failed to fetch")
    .text()
    .await
    .expect("failed to read response");

    let parsed: NewsResponse = serde_json::from_str(&body).expect("parse failed");
    
   for article in &parsed.articles {
    if seen.contains(&article.title) {
        continue; // skip, already seen
    }
    seen.insert(article.title.clone());

   
    
    // println!("Title: {}", first["title"]);
    // println!("Published: {}", first["publishedAt"]);

    
    let prompt = format!(
    "You are a financial sentiment analyzer. Analyze this stock news headline and return ONLY a valid JSON object with no extra text, no markdown, no backticks.
    
    The JSON must have exactly these two fields:
      - \"score\": a number between -1.0 and 1.0 (never null, use 0.0 if uncertain)
      - \"reason\": a single sentence string explaining the score

    Headline: {}",
    article.title
    );
    let body = serde_json::json!({
    "model": "llama-3.1-8b-instant",
    "messages": [
        { "role": "user", "content": prompt }
    ]
    });

    let response  = client.post(llm_url)
     .header("Authorization", format!("Bearer {}", llm_key))
    .header("Content-Type", "application/json")
    .json(&body)
    .send()
    .await;
    let result = response
    .expect("failed to send request")
    .text()
    .await
    .expect("failed to read response");

    let groq: LLMResponse = serde_json::from_str(&result)
    .expect("failed to parse Groq response");

let content = &groq.choices[0].message.content;
let clean = content
    .trim()
    .trim_start_matches("```json")
    .trim_start_matches("```")
    .trim_end_matches("```")
    .trim();

let sentiment: SentimentScore = serde_json::from_str(clean)
    .expect("failed to parse score JSON");
let score = sentiment.score.unwrap_or(0.0);

println!("Headline: {}", article.title);
println!("Published at: {}", article.published_at);
println!("Score: {}", score);
println!("Reason: {}", sentiment.reason);

sqlx::query("INSERT INTO signals (title,published_at, score, reason) VALUES (?,?, ?, ?)")
    .bind(&article.title)
    .bind(&article.published_at)
    .bind(score)
    .bind(&sentiment.reason)
    .execute(&pool_for_loop)
    .await
    .expect("failed to insert");
tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;


}
        
tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
    }
    
});

let app = axum::Router::new()
    .route("/signals", axum::routing::get(get_signals))
    .with_state(pool_for_api);

let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
axum::serve(listener, app).await.unwrap();

    
        
    
}
