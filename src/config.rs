

pub struct Config {
    pub news_api_key: String,
    pub llm_key: String,
    pub tickers: Vec<String>,
}

impl Config {
    pub fn load() -> Self {
        Config {
            news_api_key: std::env::var("NEWS_API_KEY").expect("NEWS_API_KEY not set"),
            llm_key: std::env::var("LLM_KEY").expect("LLM_KEY not set"),
            tickers: vec!["AAPL".to_string(), "TSLA".to_string(), "MSFT".to_string()],
        }
    }
}

