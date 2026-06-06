

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
            tickers: vec![
    "Reliance Industries stock".to_string(),
    "Infosys shares".to_string(),
    "TCS stock".to_string(),
    "HDFC Bank shares".to_string(),
    "Wipro stock".to_string(),],
        }
    }
}

