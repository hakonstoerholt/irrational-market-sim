# The "Irrational" Market Simulator

A toy market microstructure simulator. A Rust engine runs a limit order book
populated by simple trading agents (noise traders, trend followers, mean
reverters, market makers) and streams ticks and trades over WebSocket. A React
dashboard renders the live price, and there is a "God Mode" panel for injecting
shocks like flash crashes, pumps, dumps, and fake earnings beats to watch how
the agents react.

It is not a serious model of any real market. It is a sandbox for playing with
order matching and emergent price dynamics, and for having something live to
plot.

## Stack

- Engine: Rust, Tokio, Axum
- Transport: JSON over WebSocket (`ws://127.0.0.1:3000/ws`)
- Control plane: small JSON HTTP API under `/api/admin`
- Dashboard: Vite + React + Recharts
- Flow: a background sim thread pushes messages into a broadcast channel, Axum
  fans them out to every connected WebSocket client.

## Running it

### Engine

```bash
cd engine
cargo run
```

This starts the sim loop, serves the WebSocket at `ws://127.0.0.1:3000/ws`,
exposes the admin API at `http://127.0.0.1:3000/api/admin`, prints trades to
stdout, and appends every trade to `engine/trades.csv`.

### Dashboard

```bash
cd engine/frontend
npm install
npm run dev
```

Open the URL Vite prints (http://localhost:5173 by default). It connects to the
engine automatically and reconnects if the engine restarts. The floating button
in the bottom right opens the God Mode panel.

## Message format

Server messages are tagged JSON enums. Prices are integers in cents.

```jsonc
// Ticker, sent every tick
{
  "type": "ticker",
  "price": 10071,
  "tick": 182,
  "best_bid": 10070,
  "best_ask": 10072
}

// Trade, sent when orders cross
{
  "type": "trade",
  "price": 10071,
  "quantity": 1,
  "buyer_id": 12,
  "seller_id": 3
}
```

## God Mode (admin API)

The dashboard panel calls these, but they are plain POST endpoints you can hit
with curl:

```bash
# Pause / resume the sim
curl -X POST localhost:3000/api/admin/control -H 'content-type: application/json' -d '{"action":"pause"}'

# Drop a wall of sell orders far below the market
curl -X POST localhost:3000/api/admin/crash

# Earnings surprise (-50 to +50, percent)
curl -X POST localhost:3000/api/admin/earnings -H 'content-type: application/json' -d '{"surprise_pct":12.5}'
```

Other endpoints: `/order`, `/pump`, `/dump`, `/tariffs`, `/rugpull`, `/whale`.
See `engine/src/routes/admin.rs` for the request shapes and bounds.

## Agents

All agents post limit orders around the current price. Counts are set in
`engine/src/main.rs`.

- RandomWalker: buys or sells at random with a small price jitter. Provides most
  of the baseline liquidity.
- TrendFollower: compares the price now against `window_size` ticks ago and
  trades in the direction of the move.
- MeanReverter: keeps a rolling mean and standard deviation, sells when price is
  more than `k` standard deviations above the mean and buys when it is below.
- MarketMaker: quotes a fixed spread around the current price.

## Offline analysis

`analysis.py` reads `engine/trades.csv` and renders an OHLC candlestick chart
with volume to `market_analysis.png`. Let the engine run for a bit first so
there are trades to plot.

```bash
pip install -r requirements.txt
python analysis.py
```

## Layout

```
engine/
  src/
    main.rs            # sim loop, agent setup, God Mode command handling, WS server
    lib.rs             # module declarations
    orderbook.rs       # price-time priority limit order book
    agents.rs          # the trading strategies
    types.rs           # shared order / trade / message types
    state.rs           # shared app state (broadcast + command channels)
    routes/admin.rs    # /api/admin endpoints
  Cargo.toml

engine/frontend/
  src/
    App.jsx            # dashboard: WS connection, price chart, stats
    GodPanel.jsx       # God Mode control panel
  package.json

analysis.py            # offline candlestick chart from trades.csv
```

## Notes and caveats

- The order book is a pair of binary heaps with price-time priority. It only
  handles immediate matching of incoming limit orders. There is no order
  cancellation and no persistence.
- `trades.csv` is appended to on every run and is gitignored. Delete it if you
  want a clean analysis.
- The dashboard volatility tile is the standard deviation of tick-to-tick price
  changes over the visible window, so it is only as meaningful as a 50-tick toy
  sim allows.

## License

MIT. See [LICENSE](LICENSE).
