
use reqwest;
use sqlx::SqlitePool;
use std::collections::HashSet;
use crate::models::*;

pub async fn run(
    pool: SqlitePool,
    mut seen: HashSet<String>,
    api_key: String,
    llm_key: String,
    tickers: Vec<String>,
    poll_interval_secs:u64,
) {
    let query = tickers.join("+OR+");

    let client = reqwest::Client::new();
    let llm_url = "https://api.groq.com/openai/v1/chat/completions";
    let url = format!(
        "https://newsapi.org/v2/everything?q={}&language=en&apiKey={}",
        query,api_key
    );

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
    
    The JSON must have exactly these three fields:
      - \"score\": a number between -1.0 and 1.0 (never null, use 0.0 if uncertain)
      - \"reason\": a single sentence string explaining the score
      - \"relevant\": true or false — set to false if the headline is not directly about Indian stock markets, NSE, BSE, or these companies: Reliance Industries, Infosys, TCS, HDFC Bank, Wipro
    
    If irrelevant, return: {{\"score\": 0.0, \"reason\": \"irrelevant\", \"relevant\": false}}
    
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
if !sentiment.relevant.unwrap_or(true) {
    println!("Skipping irrelevant headline: {}", article.title);
    continue;
}

println!("Headline: {}", article.title);
println!("Published at: {}", article.published_at);
println!("Score: {}", score);
println!("Reason: {}", sentiment.reason);

sqlx::query("INSERT INTO signals (title,published_at, score, reason) VALUES (?,?, ?, ?)")
    .bind(&article.title)
    .bind(&article.published_at)
    .bind(score)
    .bind(&sentiment.reason)
    .execute(&pool)
    .await
    .expect("failed to insert");
tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;


}
        
tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval_secs)).await;
    }
}