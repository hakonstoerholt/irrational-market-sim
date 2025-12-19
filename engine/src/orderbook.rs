use std::collections::BinaryHeap;
use std::cmp::Ordering;
use crate::types::{Order, OrderSide, Trade};

#[derive(Debug, Eq, PartialEq)]
struct Bid(Order);

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.price.cmp(&other.0.price) {
            Ordering::Equal => other.0.timestamp.cmp(&self.0.timestamp),
            ordering => ordering,
        }
    }
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Ask(Order);

impl Ord for Ask {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.0.price.cmp(&self.0.price) {
            Ordering::Equal => other.0.timestamp.cmp(&self.0.timestamp),
            ordering => ordering,
        }
    }
}

impl PartialOrd for Ask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct OrderBook {
    bids: BinaryHeap<Bid>,
    asks: BinaryHeap<Ask>,
    pub trades: Vec<Trade>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BinaryHeap::new(),
            asks: BinaryHeap::new(),
            trades: Vec::new(),
        }
    }

    pub fn best_bid_price(&self) -> Option<u64> {
        self.bids.peek().map(|b| b.0.price)
    }

    pub fn best_ask_price(&self) -> Option<u64> {
        self.asks.peek().map(|a| a.0.price)
    }

    pub fn add_order(&mut self, mut order: Order) {
        match order.side {
            OrderSide::Bid => self.match_bid(order),
            OrderSide::Ask => self.match_ask(order),
        }
    }

    fn match_bid(&mut self, mut bid: Order) {
        while bid.amount > 0 {
            if let Some(best_ask_wrapper) = self.asks.peek() {
                let best_ask = &best_ask_wrapper.0;
                
                if bid.price >= best_ask.price {
                    let match_price = best_ask.price;
                    let match_amount = std::cmp::min(bid.amount, best_ask.amount);

                    let trade = Trade {
                        buyer_id: bid.trader_id,
                        seller_id: best_ask.trader_id,
                        price: match_price,
                        amount: match_amount,
                        timestamp: bid.timestamp,
                    };
                    self.trades.push(trade);
                    println!("Trade Executed: {} units @ ${:.2}", match_amount, match_price as f64 / 100.0);

                    bid.amount -= match_amount;
                    
                    let mut best_ask_wrapper = self.asks.pop().unwrap();
                    best_ask_wrapper.0.amount -= match_amount;

                    if best_ask_wrapper.0.amount > 0 {
                        self.asks.push(best_ask_wrapper);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if bid.amount > 0 {
            self.bids.push(Bid(bid));
        }
    }

    fn match_ask(&mut self, mut ask: Order) {
        while ask.amount > 0 {
            if let Some(best_bid_wrapper) = self.bids.peek() {
                let best_bid = &best_bid_wrapper.0;

                if ask.price <= best_bid.price {
                    let match_price = best_bid.price;
                    let match_amount = std::cmp::min(ask.amount, best_bid.amount);

                    let trade = Trade {
                        buyer_id: best_bid.trader_id,
                        seller_id: ask.trader_id,
                        price: match_price,
                        amount: match_amount,
                        timestamp: ask.timestamp,
                    };
                    self.trades.push(trade);
                    println!("Trade Executed: {} units @ ${:.2}", match_amount, match_price as f64 / 100.0);

                    ask.amount -= match_amount;

                    let mut best_bid_wrapper = self.bids.pop().unwrap();
                    best_bid_wrapper.0.amount -= match_amount;

                    if best_bid_wrapper.0.amount > 0 {
                        self.bids.push(best_bid_wrapper);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if ask.amount > 0 {
            self.asks.push(Ask(ask));
        }
    }

    pub fn drain_trades(&mut self) -> Vec<Trade> {
        self.trades.drain(..).collect()
    }
}
