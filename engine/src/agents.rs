use crate::types::{Order, OrderSide, Trade};
use rand::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum Strategy {
    RandomWalker,
    TrendFollower { window_size: usize },
    MeanReverter { window_size: usize, std_dev_multiplier: f64 },
    MarketMaker { spread_bps: u64 },
}

#[derive(Debug)]
pub struct Agent {
    pub id: u64,
    pub name: String,
    pub cash: u64,
    pub inventory: u64,
    pub strategy: Strategy,
    pub price_history: VecDeque<u64>,
    pub max_history: usize,
}

impl Agent {
    pub fn new(id: u64, name: String, cash: u64, inventory: u64, strategy: Strategy) -> Self {
        Self {
            id,
            name,
            cash,
            inventory,
            strategy,
            price_history: VecDeque::new(),
            max_history: 50, // Default memory size
        }
    }

    pub fn update_market_data(&mut self, price: u64) {
        if price == 0 { return; }
        self.price_history.push_back(price);
        if self.price_history.len() > self.max_history {
            self.price_history.pop_front();
        }
    }

    pub fn on_trade(&mut self, trade: &Trade) {
        if trade.buyer_id == self.id {
            let cost = trade.price * trade.amount;
            if self.cash >= cost {
                self.cash -= cost;
                self.inventory += trade.amount;
            } else {
            }
        } else if trade.seller_id == self.id {
            let revenue = trade.price * trade.amount;
            self.cash += revenue;
            if self.inventory >= trade.amount {
                self.inventory -= trade.amount;
            }
        }
    }

    pub fn act(&mut self, current_price: u64) -> Option<Order> {
        if current_price == 0 { return None; }

        let mut rng = rand::thread_rng();
        let amount = 1;

        match self.strategy {
            Strategy::RandomWalker => {
                if rng.gen_bool(0.5) {
                    if self.cash >= current_price * amount {
                        let price_noise = rng.gen_range(-20..=20);
                        let price = (current_price as i64 + price_noise).max(1) as u64;
                        return Some(Order {
                            id: rng.next_u64(),
                            trader_id: self.id,
                            side: OrderSide::Bid,
                            price,
                            amount,
                            timestamp: 0,
                        });
                    }
                } else {
                    if self.inventory >= amount {
                        let price_noise = rng.gen_range(-20..=20);
                        let price = (current_price as i64 + price_noise).max(1) as u64;
                        return Some(Order {
                            id: rng.next_u64(),
                            trader_id: self.id,
                            side: OrderSide::Ask,
                            price,
                            amount,
                            timestamp: 0,
                        });
                    }
                }
            }
            Strategy::TrendFollower { window_size } => {
                if self.price_history.len() < window_size { return None; }
                
                let old_price = self.price_history[self.price_history.len() - window_size];
                
                if current_price > old_price {
                    if self.cash >= current_price * amount {
                        return Some(Order {
                            id: rng.next_u64(),
                            trader_id: self.id,
                            side: OrderSide::Bid,
                            price: current_price,
                            amount,
                            timestamp: 0,
                        });
                    }
                } else if current_price < old_price {
                    if self.inventory >= amount {
                        return Some(Order {
                            id: rng.next_u64(),
                            trader_id: self.id,
                            side: OrderSide::Ask,
                            price: current_price,
                            amount,
                            timestamp: 0,
                        });
                    }
                }
            }
            Strategy::MeanReverter { window_size, std_dev_multiplier } => {
                if self.price_history.len() < window_size { return None; }
                
                let sum: u64 = self.price_history.iter().sum();
                let mean = sum as f64 / self.price_history.len() as f64;
                
                let variance: f64 = self.price_history.iter()
                    .map(|&p| {
                        let diff = mean - p as f64;
                        diff * diff
                    })
                    .sum::<f64>() / self.price_history.len() as f64;
                let std_dev = variance.sqrt();

                let upper_bound = mean + std_dev_multiplier * std_dev;
                let lower_bound = mean - std_dev_multiplier * std_dev;

                if (current_price as f64) > upper_bound {
                    if self.inventory >= amount {
                        return Some(Order {
                            id: rng.next_u64(),
                            trader_id: self.id,
                            side: OrderSide::Ask,
                            price: current_price,
                            amount,
                            timestamp: 0,
                        });
                    }
                } else if (current_price as f64) < lower_bound {
                    if self.cash >= current_price * amount {
                        return Some(Order {
                            id: rng.next_u64(),
                            trader_id: self.id,
                            side: OrderSide::Bid,
                            price: current_price,
                            amount,
                            timestamp: 0,
                        });
                    }
                }
            }
            Strategy::MarketMaker { spread_bps } => {
                let spread_amount = (current_price as f64 * (spread_bps as f64 / 10000.0)) as u64;
                let spread_amount = spread_amount.max(10);
                
                if rng.gen_bool(0.5) {
                    if self.cash >= (current_price.saturating_sub(spread_amount)) * amount {
                        return Some(Order {
                            id: rng.next_u64(),
                            trader_id: self.id,
                            side: OrderSide::Bid,
                            price: current_price.saturating_sub(spread_amount),
                            amount,
                            timestamp: 0,
                        });
                    }
                } else {
                    if self.inventory >= amount {
                        return Some(Order {
                            id: rng.next_u64(),
                            trader_id: self.id,
                            side: OrderSide::Ask,
                            price: current_price + spread_amount,
                            amount,
                            timestamp: 0,
                        });
                    }
                }
            }
        }
        None
    }
}
