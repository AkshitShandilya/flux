

#[derive(serde::Deserialize)]
pub struct Article {
    pub title: String,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
}

#[derive(serde::Deserialize)]
pub struct NewsResponse {
    pub articles: Vec<Article>,
}

#[derive(serde::Deserialize)]
pub struct LLMResponse {
    pub choices: Vec<Choice>,
}

#[derive(serde::Deserialize)]
pub struct Choice {
    pub message: Message,
}

#[derive(serde::Deserialize)]
pub struct Message {
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct SentimentScore {
    pub score: Option<f64>,
    pub reason: String,
    pub relevant: Option<bool>,
}