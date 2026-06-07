
#[derive(serde::Deserialize)]
pub struct AlphaResponse {
    #[serde(rename = "Global Quote")]
    pub global_quote: GlobalQuote,
}

#[derive(serde::Deserialize)]
pub struct GlobalQuote {
    #[serde(rename = "10. change percent")]
    pub change_percent: String,
}
use std::collections::HashMap;
use std::time::{Instant, Duration};

pub struct PriceCache {
    cache: HashMap<String, (f64, Instant)>,
    interval: Duration,
}

impl PriceCache {
    pub fn new(interval_secs: u64) -> Self {
        PriceCache {
            cache: HashMap::new(),
            interval: Duration::from_secs(interval_secs),
        }
    }

    pub async fn get_momentum(
        &mut self,
        symbol: &str,
        api_key: &str,
        client: &reqwest::Client,
    ) -> f64 {
        let now = Instant::now();
        if let Some((cached, fetched_at)) = self.cache.get(symbol) {
            if now.duration_since(*fetched_at) < self.interval {
                return *cached;
            }
        }
        let m = fetch_momentum(symbol, api_key, client).await;
        self.cache.insert(symbol.to_string(), (m, now));
        m
    }
}

pub async fn fetch_momentum(
    ticker_symbol: &str,
    api_key: &str,
    client: &reqwest::Client,
) -> f64 {
    let price_url = format!("https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
    ticker_symbol,api_key);
    
    let body  = client
    .get(price_url)
    .send()
    .await
    .expect("failed to fetch price")
    .text()
    .await
    .expect("failed to read response");

    let alpha: AlphaResponse = serde_json::from_str(&body).expect("failed to parse alpha response");

    let clean = alpha.global_quote.change_percent
    .trim()
    .trim_end_matches("%");

    let change: f64 = clean.parse().unwrap_or(0.0);
   (change / 5.0).clamp(-1.0, 1.0)
    
}