#[derive(serde::Deserialize, Clone)]
pub struct Ticker {
    pub name: String,
    pub symbol: String,
}
#[derive(serde::Deserialize)]
pub struct Config {
    pub news_api_key: String,
    pub llm_key: String,
    pub alpha_vantage_key: String,
    pub tickers: Vec<Ticker>,
    pub poll_interval_secs: u64,
    pub price_fetch_interval_secs: u64,
    pub port: u16,
    pub alpha: f64,
    pub beta: f64,
}
 #[derive(serde::Deserialize)]
struct TomlConfig {
    tickers: Vec<Ticker>,
    poll_interval_secs: u64,
    price_fetch_interval_secs: u64,
    port: u16,
    alpha: f64,
    beta: f64,
}
impl Config {
    pub fn load() -> Self {
        let contents = std::fs::read_to_string("config.toml").expect("config.toml not found");
        let toml_config: TomlConfig = toml::from_str(&contents).expect("failed to parse config.toml");

        Config {
            news_api_key: std::env::var("NEWS_API_KEY").expect("NEWS_API_KEY not set"),
            llm_key: std::env::var("LLM_KEY").expect("LLM_KEY not set"),
            alpha_vantage_key: std::env::var("ALPHA_VANTAGE_KEY").expect("ALPHA_VANTAGE_KEY not set"),
            tickers: toml_config.tickers,
            poll_interval_secs: toml_config.poll_interval_secs,
            price_fetch_interval_secs: toml_config.price_fetch_interval_secs,
            port: toml_config.port,
            alpha: toml_config.alpha,
            beta: toml_config.beta,

            
    }
}
}
