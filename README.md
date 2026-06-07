# Flux

A real-time sentiment-weighted trading signal engine built in Rust.

Flux monitors financial news headlines, scores them using an LLM, blends the sentiment with live price momentum, and streams the results over a REST API and websocket. Plug it into any frontend, trading dashboard, or downstream service — it just works.

---

## How it works

```
NewsAPI → deduplicate → relevance filter → LLM sentiment scoring
       → price momentum fetch → α·sentiment + β·momentum → SQLite
       → REST API + Websocket
```

Flux runs two concurrent tasks:

- **Poller** — fetches headlines every N minutes, filters irrelevant ones, scores new ones via Groq, blends with live price momentum, persists to SQLite and broadcasts over websocket
- **API server** — serves all scored signals over HTTP and websocket on a configurable port

Signals persist across restarts. No headline is ever scored twice.

---

## Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust |
| Async runtime | Tokio |
| HTTP + Websocket server | Axum |
| Database | SQLite via sqlx |
| HTTP client | reqwest |
| LLM scoring | Groq — LLaMA 3.1 8B |
| News data | NewsAPI |
| Price data | Alpha Vantage |
| Logging | tracing |

---

## Getting started

### Prerequisites

- Rust — [rustup.rs](https://rustup.rs)
- NewsAPI key — [newsapi.org](https://newsapi.org) (free tier)
- Groq API key — [console.groq.com](https://console.groq.com) (free tier)
- Alpha Vantage key — [alphavantage.co](https://alphavantage.co) (free tier)

### Setup

```bash
git clone https://github.com/yourusername/flux.git
cd flux
```

Create a `.env` file:

```
NEWS_API_KEY=your_newsapi_key
LLM_KEY=your_groq_key
ALPHA_VANTAGE_KEY=your_alphavantage_key
```

Edit `config.toml` to configure tickers, weights, and intervals:

```toml
poll_interval_secs = 120
price_fetch_interval_secs = 14400
port = 3000
alpha = 0.7
beta = 0.3

[[tickers]]
name = "Infosys shares"
symbol = "INFY.BSE"

[[tickers]]
name = "TCS stock"
symbol = "TCS.BSE"

[[tickers]]
name = "Reliance Industries stock"
symbol = "RELIANCE.BSE"

[[tickers]]
name = "HDFC Bank shares"
symbol = "HDFCBANK.BSE"

[[tickers]]
name = "Wipro stock"
symbol = "WIPRO.BSE"
```

Run:

```bash
cargo run
```

The engine starts polling immediately. API is live at `http://localhost:3000`.

---

## API

### Get all signals

```
GET /signals
```

### Filter by ticker

```
GET /signals?ticker=Infosys
```

### Limit results

```
GET /signals?limit=10
```

### Combine filters

```
GET /signals?ticker=HDFC&limit=5
```

**Response**

```json
{
  "signals": [
    {
      "id": 1,
      "title": "Infosys Q4 results beat estimates",
      "published_at": "2026-06-03T10:00:00Z",
      "sentiment_score": 0.8,
      "blended_score": 0.65,
      "reason": "Strong Q4 earnings beat suggests positive outlook for Infosys stock"
    }
  ]
}
```

### Websocket — real-time signals

Connect to `ws://localhost:3000/ws` to receive signals instantly as they are scored:

```javascript
const ws = new WebSocket("ws://localhost:3000/ws");
ws.onmessage = (event) => console.log(JSON.parse(event.data));
```

---

## Scoring

### Sentiment score

Every headline is scored by an LLM on a scale from -1.0 to 1.0:

| Score | Meaning |
|-------|---------|
| 1.0 | Strongly positive |
| 0.5 | Mildly positive |
| 0.0 | Neutral / irrelevant |
| -0.5 | Mildly negative |
| -1.0 | Strongly negative |

### Blended score

The final signal blends sentiment with live price momentum:

```
blended_score = (α × sentiment_score) + (β × momentum)
```

Where `α` and `β` are configurable weights in `config.toml`. Momentum is derived from the stock's daily price change, normalized to -1.0 to 1.0.

This is the research hook — tuning α and β lets you experiment with how much weight sentiment vs price movement should carry for different stocks and market conditions.

---

## Project structure

```
src/
  main.rs       — setup and wiring
  config.rs     — TOML + env var config
  models.rs     — data structs
  poller.rs     — fetch, score, and persist loop
  api.rs        — HTTP and websocket handlers
  price.rs      — Alpha Vantage client and price cache
```

---

## License

MIT