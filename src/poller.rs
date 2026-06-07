
use reqwest;
use sqlx::SqlitePool;
use std::collections::HashSet;
use crate::{config::Ticker, models::*};



pub async fn run(
    pool: SqlitePool,
    mut seen: HashSet<String>,
    tx: tokio::sync::broadcast::Sender<String>,
    api_key: String,
    llm_key: String,
    tickers: Vec<Ticker>,
    poll_interval_secs:u64,
    alpha:f64,
    beta:f64,
    alpha_vantage_key:String,
    price_fetch_interval_secs:u64,
) {
   let query = tickers.iter().map(|t| t.name.as_str()).collect::<Vec<&str>>().join("+OR+");

    let client = reqwest::Client::new();
    let llm_url = "https://api.groq.com/openai/v1/chat/completions";
    let url = format!(
        "https://newsapi.org/v2/everything?q={}&language=en&apiKey={}",
        query,api_key
    );
    let mut price_cache = crate::price::PriceCache::new(price_fetch_interval_secs);

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
    tracing::warn!("Skipping irrelevant: {}", article.title);  
    continue;
}

let ticker_symbol = tickers.iter()
    .find(|t| article.title.contains(&t.name))
    .map(|t| t.symbol.as_str());

let momentum = match ticker_symbol {
    Some(symbol) => price_cache.get_momentum(symbol, &alpha_vantage_key, &client).await,
    None => 0.0,
};

let final_score = (alpha * score) + (beta * momentum);
let signal = serde_json::json!({
    "title": article.title,
    "published_at": article.published_at,
    "sentiment_score":score,
    "blended_score": final_score,
    "reason": sentiment.reason
});

tx.send(signal.to_string()).ok();

tracing::info!("Headline: {}", article.title);
tracing::info!("Published At: {} |Sentiment: {} | Blended: {} | Reason: {}", article.published_at,score, final_score, sentiment.reason);

sqlx::query("INSERT INTO signals (title,published_at, sentiment_score,blended_score, reason) VALUES (?,?,?, ?, ?)")
    .bind(&article.title)
    .bind(&article.published_at)
    .bind(score)
    .bind(final_score)
    .bind(&sentiment.reason)
    .execute(&pool)
    .await
    .expect("failed to insert");
tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;


}
        
tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval_secs)).await;
    }
}