
#[derive(serde::Deserialize)]
pub struct Config {
    pub news_api_key: String,
    pub llm_key: String,
    pub tickers: Vec<String>,
    pub poll_interval_secs: u64,
    pub port: u16,
}
 #[derive(serde::Deserialize)]
struct TomlConfig {
    tickers: Vec<String>,
    poll_interval_secs: u64,
    port: u16,
}
impl Config {
    pub fn load() -> Self {
        let contents = std::fs::read_to_string("config.toml").expect("config.toml not found");
        let toml_config: TomlConfig = toml::from_str(&contents).expect("failed to parse config.toml");

        Config {
            news_api_key: std::env::var("NEWS_API_KEY").expect("NEWS_API_KEY not set"),
            llm_key: std::env::var("LLM_KEY").expect("LLM_KEY not set"),
            tickers: toml_config.tickers,
            poll_interval_secs: toml_config.poll_interval_secs,
            port: toml_config.port,
            
    }
}
}
