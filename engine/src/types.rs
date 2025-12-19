use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Bid, // Buy
    Ask, // Sell
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub trader_id: u64,
    pub side: OrderSide,
    pub price: u64,  // In cents/satoshis
    pub amount: u64, // Number of units
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    pub buyer_id: u64,
    pub seller_id: u64,
    pub price: u64,
    pub amount: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "ticker")]
    Ticker {
        price: u64,
        tick: u64,
        best_bid: u64,
        best_ask: u64,
    },
    #[serde(rename = "trade")]
    Trade {
        price: u64,
        quantity: u64,
        buyer_id: u64,
        seller_id: u64,
    }
}

/// Commands sent from the control plane (Axum handlers) to the simulation thread.
#[derive(Debug, Clone)]
pub enum SimulationCommand {
    /// Manually inject a buy or sell order into the book
    InjectOrder(Order),
    /// Pause/resume the simulation loop
    SetPaused(bool),
    /// Reset the order book and all agent inventories
    Reset,
    /// Inject massive sell orders to simulate a flash crash
    FlashCrash { seller_id: u64, quantity: u64, price: u64 },
    /// Adjust volatility multiplier for all random walkers
    UpdateVolatility(f64),
    /// Create sustained buy pressure (multiple large bids)
    Pump { buyer_id: u64, base_price: u64, magnitude: f64 },
    /// Create sustained sell pressure (multiple large asks)
    Dump { seller_id: u64, base_price: u64, magnitude: f64 },
    /// Earnings announcement effect (positive or negative surprise)
    Earnings { surprise_pct: f64 }, // +10.0 = beat by 10%, -15.0 = miss by 15%
    /// Tariff/trade war announcement (negative shock)
    Tariffs { severity: f64 }, // 0.0-1.0, higher = worse
    /// Insider rug pull (coordinated large sell)
    RugPull { magnitude: f64 },
    /// Whale accumulation (sustained buying)
    WhaleAccumulation { magnitude: f64 },
}
