use market_engine::types::{ServerMessage};
use market_engine::orderbook::OrderBook;
use market_engine::agents::{Agent, Strategy};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::error::Error;
use csv::Writer;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::broadcast;
use std::sync::Arc;
use std::net::SocketAddr;

struct AppState {
    tx: broadcast::Sender<ServerMessage>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Create the Broadcast Channel
    let (tx, _rx) = broadcast::channel::<ServerMessage>(100);

    // 2. Spawn the Simulation in a separate blocking thread
    let sim_tx = tx.clone();
    thread::spawn(move || {
        let mut book = OrderBook::new();
        let mut agents: HashMap<u64, Agent> = HashMap::new();
        let mut agent_ids: Vec<u64> = Vec::new();
        let mut rng = thread_rng();
        
        // CSV Writer
        let mut wtr = Writer::from_path("trades.csv").unwrap();

        // Initialize Agents
        let mut id_counter = 0;

        // 1. Random Walkers
        for i in 0..10 {
            id_counter += 1;
            let agent = Agent::new(id_counter, format!("Noise_{}", i), 1000000, 1000, Strategy::RandomWalker);
            agents.insert(id_counter, agent);
            agent_ids.push(id_counter);
        }

        // 2. Trend Followers
        for i in 0..5 {
            id_counter += 1;
            let agent = Agent::new(id_counter, format!("Trend_{}", i), 1000000, 1000, Strategy::TrendFollower { window_size: 5 });
            agents.insert(id_counter, agent);
            agent_ids.push(id_counter);
        }

        // 3. Mean Reverters
        for i in 0..5 {
            id_counter += 1;
            let agent = Agent::new(id_counter, format!("Mean_{}", i), 1000000, 1000, Strategy::MeanReverter { window_size: 10, std_dev_multiplier: 2.0 });
            agents.insert(id_counter, agent);
            agent_ids.push(id_counter);
        }

        let mut current_price = 10000;
        let mut tick = 0;

        println!("--- Simulation Started (Background Thread) ---");

        loop {
            tick += 1;
            
            // Shuffle agents
            agent_ids.shuffle(&mut rng);

            // Agents act
            for id in &agent_ids {
                if let Some(agent) = agents.get_mut(id) {
                    agent.update_market_data(current_price);
                    if let Some(mut order) = agent.act(current_price) {
                        order.timestamp = tick;
                        book.add_order(order);
                    }
                }
            }

            // Process trades
            let new_trades = book.drain_trades();
            if !new_trades.is_empty() {
                current_price = new_trades.last().unwrap().price;
                
                for trade in &new_trades {
                    if let Some(buyer) = agents.get_mut(&trade.buyer_id) { buyer.on_trade(trade); }
                    if let Some(seller) = agents.get_mut(&trade.seller_id) { seller.on_trade(trade); }
                    
                    // Log to CSV
                    wtr.serialize(trade).ok();

                    // Broadcast Trade
                    let _ = sim_tx.send(ServerMessage::Trade {
                        price: trade.price,
                        quantity: trade.amount,
                        buyer_id: trade.buyer_id,
                        seller_id: trade.seller_id,
                    });
                    
                    println!("Tick {}: Trade @ ${:.2} ({} units)", tick, trade.price as f64 / 100.0, trade.amount);
                }
                wtr.flush().ok();
            }

            // Broadcast Ticker (every tick)
            let _ = sim_tx.send(ServerMessage::Ticker {
                price: current_price,
                tick,
                best_bid: book.best_bid_price().unwrap_or(0),
                best_ask: book.best_ask_price().unwrap_or(0),
            });

            // Sleep to control speed
            thread::sleep(Duration::from_millis(100));
        }
    });

    // 3. Setup the Web Server
    let app_state = Arc::new(AppState { tx });
    
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("WebSocket Server Listening on ws://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&msg) {
            if socket.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    }
}
