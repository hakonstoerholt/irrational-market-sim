use market_engine::types::{ServerMessage, SimulationCommand};
use market_engine::orderbook::OrderBook;
use market_engine::agents::{Agent, Strategy};
use market_engine::routes::admin;
use market_engine::state::AppState;
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
use tower_http::cors::CorsLayer;
use tokio::sync::{broadcast, mpsc};
use std::sync::Arc;
use std::net::SocketAddr;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Create the Broadcast Channel for market data
    let (tx, _rx) = broadcast::channel::<ServerMessage>(100);

    // 2. Create the MPSC Command Channel for control plane
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<SimulationCommand>();

    // 3. Spawn the Simulation in a separate blocking thread
    let sim_tx = tx.clone();
    thread::spawn(move || {
        simulation_loop(sim_tx, cmd_rx);
    });

    // 4. Setup the Web Server
    let app_state = Arc::new(AppState { 
        tx, 
        cmd_tx,
    });
    
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .merge(admin::admin_routes())
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("WebSocket Server Listening on ws://{}", addr);
    println!("Admin API at http://127.0.0.1:3000/api/admin/");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn simulation_loop(
    sim_tx: broadcast::Sender<ServerMessage>,
    mut cmd_rx: mpsc::UnboundedReceiver<SimulationCommand>,
) {
    let mut book = OrderBook::new();
    let mut agents: HashMap<u64, Agent> = HashMap::new();
    let mut agent_ids: Vec<u64> = Vec::new();
    let mut rng = thread_rng();
    
    // CSV Writer
    let mut wtr = Writer::from_path("trades.csv").unwrap();

    // Initialize Agents (50 total for more realistic dynamics)
    let mut id_counter = 0;

    // 1. Random Walkers (20 agents - liquidity providers)
    for i in 0..20 {
        id_counter += 1;
        let agent = Agent::new(id_counter, format!("Noise_{}", i), 1000000, 1000, Strategy::RandomWalker);
        agents.insert(id_counter, agent);
        agent_ids.push(id_counter);
    }

    // 2. Trend Followers (15 agents - momentum traders)
    for i in 0..15 {
        id_counter += 1;
        let agent = Agent::new(id_counter, format!("Trend_{}", i), 1000000, 1000, Strategy::TrendFollower { window_size: 5 });
        agents.insert(id_counter, agent);
        agent_ids.push(id_counter);
    }

    // 3. Mean Reverters (10 agents - contrarian traders)
    for i in 0..10 {
        id_counter += 1;
        let agent = Agent::new(id_counter, format!("Mean_{}", i), 1000000, 1000, Strategy::MeanReverter { window_size: 10, std_dev_multiplier: 1.5 });
        agents.insert(id_counter, agent);
        agent_ids.push(id_counter);
    }

    // 4. Market Makers (5 agents - provide tight spreads)
    for i in 0..5 {
        id_counter += 1;
        let agent = Agent::new(id_counter, format!("MM_{}", i), 2000000, 2000, Strategy::MarketMaker { spread_bps: 50 }); // 0.5% spread
        agents.insert(id_counter, agent);
        agent_ids.push(id_counter);
    }

    println!("--- Initialized {} agents ---", agents.len());

    let mut current_price = 10000;
    let mut tick = 0;
    let mut paused = false;

    println!("--- Simulation Started (Background Thread) ---");

    loop {
        // Check for commands with a timeout (10ms per iteration for responsiveness)
        let deadline = std::time::Instant::now() + Duration::from_millis(10);
        while std::time::Instant::now() < deadline {
            match cmd_rx.try_recv() {
                Ok(cmd) => {
                    handle_command(&cmd, &mut paused, &mut book, &mut agents, &mut current_price, &mut tick);
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    println!("Command channel closed, exiting simulation.");
                    return;
                }
            }
        }

        // Skip tick processing if paused
        if paused {
            continue;
        }

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
        }

        // Broadcast Ticker (every tick)
        let _ = sim_tx.send(ServerMessage::Ticker {
            price: current_price,
            tick,
            best_bid: book.best_bid_price().unwrap_or(0),
            best_ask: book.best_ask_price().unwrap_or(0),
        });

        // Sleep to control tick rate
        thread::sleep(Duration::from_millis(100));
    }
}

fn handle_command(
    cmd: &SimulationCommand,
    paused: &mut bool,
    book: &mut OrderBook,
    agents: &mut HashMap<u64, Agent>,
    current_price: &mut u64,
    tick: &mut u64,
) {
    match cmd {
        SimulationCommand::InjectOrder(order) => {
            println!("God Mode: Injecting order from {:?}", order.side);
            book.add_order(*order);
        }
        SimulationCommand::SetPaused(is_paused) => {
            *paused = *is_paused;
            println!("God Mode: Simulation {}", if *is_paused { "PAUSED" } else { "RESUMED" });
        }
        SimulationCommand::Reset => {
            println!("God Mode: Resetting simulation!");
            *book = OrderBook::new();
            *current_price = 10000;
            *tick = 0;
            for agent in agents.values_mut() {
                agent.cash = 1000000;
                agent.inventory = 1000;
            }
        }
        SimulationCommand::FlashCrash { seller_id, quantity, price } => {
            println!("God Mode: FLASH CRASH incoming! {} units @ ${:.2}", quantity, *price as f64 / 100.0);
            let crash_order = market_engine::types::Order {
                id: rand::random(),
                trader_id: *seller_id,
                side: market_engine::types::OrderSide::Ask,
                price: *price,
                amount: *quantity,
                timestamp: *tick,
            };
            book.add_order(crash_order);
        }
        SimulationCommand::UpdateVolatility(_multiplier) => {
            println!("God Mode: Volatility update received (not yet implemented)");
        }
        SimulationCommand::Pump { buyer_id, base_price, magnitude } => {
            println!("God Mode: PUMP initiated! Creating buy pressure (magnitude: {:.1}x)", magnitude);
            let start_price = if *base_price == 0 { 
                *current_price + 200
            } else { 
                *base_price 
            };
            let quantity = (2000.0 * magnitude) as u64;
            for i in 0..5 {
                let price = start_price + (i * 50);
                let order = market_engine::types::Order {
                    id: rand::random(),
                    trader_id: *buyer_id,
                    side: market_engine::types::OrderSide::Bid,
                    price,
                    amount: quantity,
                    timestamp: *tick,
                };
                book.add_order(order);
            }
        }
        SimulationCommand::Dump { seller_id, base_price, magnitude } => {
            println!("God Mode: DUMP initiated! Creating sell pressure (magnitude: {:.1}x)", magnitude);
            let start_price = if *base_price == 0 {
                current_price.saturating_sub(200) 
            } else {
                *base_price
            };
            let quantity = (2000.0 * magnitude) as u64;
            for i in 0..5 {
                let price = start_price.saturating_sub(i * 50);
                let order = market_engine::types::Order {
                    id: rand::random(),
                    trader_id: *seller_id,
                    side: market_engine::types::OrderSide::Ask,
                    price: price.max(1),
                    amount: quantity,
                    timestamp: *tick,
                };
                book.add_order(order);
            }
        }
        SimulationCommand::Earnings { surprise_pct } => {
            println!("God Mode: EARNINGS announced! Surprise: {:.1}%", surprise_pct);
            let trader_id = 999_999;
            
            if *surprise_pct > 0.0 {
                let magnitude = (surprise_pct.abs() * 30.0) as u64;
                for i in 0..5 {
                    let price = *current_price + (i * 100) + 300;
                    let order = market_engine::types::Order {
                        id: rand::random(),
                        trader_id,
                        side: market_engine::types::OrderSide::Bid,
                        price,
                        amount: (1500 + magnitude * 10).min(5000),
                        timestamp: *tick,
                    };
                    book.add_order(order);
                }
            } else {
                let magnitude = (surprise_pct.abs() * 30.0) as u64;
                for i in 0..5 {
                    let price = current_price.saturating_sub((i * 100) + 300);
                    let order = market_engine::types::Order {
                        id: rand::random(),
                        trader_id,
                        side: market_engine::types::OrderSide::Ask,
                        price: price.max(1),
                        amount: (1500 + magnitude * 10).min(5000),
                        timestamp: *tick,
                    };
                    book.add_order(order);
                }
            }
        }
        SimulationCommand::Tariffs { severity } => {
            println!("God Mode: TARIFFS announced! Severity: {:.1}", severity);
            let trader_id = 999_998;
            
            let quantity = (1000.0 + severity * 500.0) as u64;
            let price_impact = (severity * 100.0) as u64;
            
            for i in 0..7 {
                let price = current_price.saturating_sub(price_impact + (i * 50));
                let order = market_engine::types::Order {
                    id: rand::random(),
                    trader_id,
                    side: market_engine::types::OrderSide::Ask,
                    price: price.max(1),
                    amount: quantity,
                    timestamp: *tick,
                };
                book.add_order(order);
            }
        }
        SimulationCommand::RugPull { magnitude } => {
            println!("God Mode: RUG PULL! Magnitude: {:.1}x", magnitude);
            let trader_id = 999_997;
            

            let quantity = (3000.0 * magnitude) as u64;
            let crash_price = ((*current_price as f64) * (1.0 - 0.15 * magnitude)) as u64;
            
            for i in 0..10 {
                let price = crash_price.saturating_sub(i * 20);
                let order = market_engine::types::Order {
                    id: rand::random(),
                    trader_id,
                    side: market_engine::types::OrderSide::Ask,
                    price: price.max(1),
                    amount: quantity,
                    timestamp: *tick,
                };
                book.add_order(order);
            }
        }
        SimulationCommand::WhaleAccumulation { magnitude } => {
            println!("God Mode: WHALE ACCUMULATION detected! Magnitude: {:.1}x", magnitude);
            let trader_id = 999_996;
            
            let quantity = (2000.0 * magnitude) as u64;
            let premium = (50.0 * magnitude) as u64;
            
            for i in 0..8 {
                let price = *current_price + premium + (i * 20);
                let order = market_engine::types::Order {
                    id: rand::random(),
                    trader_id,
                    side: market_engine::types::OrderSide::Bid,
                    price,
                    amount: quantity,
                    timestamp: *tick,
                };
                book.add_order(order);
            }
        }
    }
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
