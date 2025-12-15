# The "Irrational" Market Simulator

A toy market microstructure simulator with a full limit order book, simple trading agents (noise, trend-followers, mean-reverters), and a live dashboard. The Rust engine streams ticks and trades over WebSocket; a React UI renders real‑time price action.

## Overview
- Engine: Rust + Tokio + Axum
- Messaging: JSON over WebSocket (`ws://127.0.0.1:3000/ws`)
- Dashboard: Vite + React + Recharts
- Architecture: Background sim thread → broadcast channel → Axum WS → Browser

## Quick Start

### 1) Run the Engine
```bash
cd "engine"
cargo run
```
- Starts the sim loop
- Serves WS on `ws://127.0.0.1:3000/ws`
- Prints trade logs and writes `trades.csv`

### 2) Run the Dashboard
```bash
cd "engine/frontend"
npm install
npm run dev -- --host
```
- Opens http://localhost:5173
- Shows status, live price chart, and basic stats

## Data Format
Messages are tagged enums serialized as JSON.

```jsonc
// Ticker
{
  "type": "ticker",
  "price": 10071,   // cents
  "tick": 182,      // sim tick
  "best_bid": 10070,
  "best_ask": 10072
}

// Trade
{
  "type": "trade",
  "price": 10071,
  "quantity": 1,
  "buyer_id": 12,
  "seller_id": 3
}
```

## Project Structure
```
engine/
  src/
    main.rs         # Sim loop + Axum WS server
    orderbook.rs    # Limit order book
    agents.rs       # Noise / Trend / Mean-revert agents
    types.rs        # Shared message + order types
  Cargo.toml

engine/frontend/
  src/
    App.jsx         # Dashboard
    main.jsx
    index.css       # Minimal utility CSS (Tailwind optional)
  index.html
  package.json

trades.csv          # Appended by engine at runtime
```

## Notes
- UI Status: Uses native `WebSocket` with auto‑reconnect for robustness.
- Styling: Minimal utility CSS is included. Tailwind v4 configs exist but are optional.
- Performance: Chart keeps a small rolling window for smooth rendering.

## Troubleshooting
- Blank or blue screen: hard refresh (Cmd+Shift+R). If the UI crashes, an in‑page error will show stack traces.
- Stuck at "Connecting": ensure the engine is running and reachable at `127.0.0.1:3000`.
- Port busy: stop other dev servers or use `--host` or alternate ports.

## Roadmap Ideas
- Depth-of-book (L2) view + order flow heatmap
- Per-agent PnL and inventory
- Scenario controls (volatility, agent mix)
- Snapshot + replay from `trades.csv`

---
Made for tinkering with market microstructure and live visualizations.
