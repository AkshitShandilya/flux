# flux

A real-time sentiment-weighted trading signal engine built in Rust.

Flux monitors financial news headlines, scores them using an LLM, and streams the results over a REST API. Plug it into any frontend, trading dashboard, or downstream service — it just works.

---

## How it works

```
NewsAPI → deduplicate → LLM scoring → SQLite → REST API
```

Flux runs two concurrent tasks:

- **Poller** — fetches headlines every 2 minutes, skips ones already seen, scores new ones via Groq
- **API server** — serves all scored signals over HTTP on port 3000, ready for any client to consume

Signals persist across restarts. No headline is ever scored twice.

---

## Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust |
| Async runtime | Tokio |
| HTTP server | Axum |
| Database | SQLite via sqlx |
| HTTP client | reqwest |
| LLM scoring | Groq — LLaMA 3.1 8B |
| News data | NewsAPI |

---

## Getting started

### Prerequisites

- Rust — [rustup.rs](https://rustup.rs)
- NewsAPI key — [newsapi.org](https://newsapi.org) (free tier)
- Groq API key — [console.groq.com](https://console.groq.com) (free tier)

### Setup

```bash
git clone https://github.com/yourusername/flux.git
cd flux
```

Create a `.env` file in the project root:

```
NEWS_API_KEY=your_newsapi_key
LLM_KEY=your_groq_key
```

Run:

```bash
cargo run
```

That's it. The engine starts polling immediately and the API is live at `http://localhost:3000`.

---

## API

### Get all signals

```
GET /signals
```

Returns all scored headlines ordered by most recent first. Ready to plug into any frontend or downstream service.

**Response**

```json
{
  "signals": [
    {
      "id": 1,
      "title": "Apple hits record high after earnings beat",
      "published_at": "2026-06-03T10:00:00Z",
      "score": 0.9,
      "reason": "Strong earnings beat with record revenue suggests bullish outlook for AAPL"
    },
    {
      "id": 2,
      "title": "Apple faces antitrust probe in EU",
      "published_at": "2026-06-02T08:30:00Z",
      "score": -0.7,
      "reason": "Regulatory pressure from the EU could negatively impact Apple's European revenue"
    }
  ]
}
```

---

## Scoring

Every headline is scored by an LLM on a scale from -1.0 to 1.0:

| Score | Meaning |
|-------|---------|
| 1.0 | Strongly positive for the stock |
| 0.5 | Mildly positive |
| 0.0 | Neutral or insufficient context |
| -0.5 | Mildly negative |
| -1.0 | Strongly negative for the stock |

---

## Integrating with a frontend

The API returns plain JSON — drop it into any frontend with a single fetch call:

```javascript
const res = await fetch("http://localhost:3000/signals");
const { signals } = await res.json();
```

Each signal has everything you need to display a sentiment dashboard — title, score, reason, and timestamp.

---

## License

MIT
