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
