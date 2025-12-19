use axum::{
    extract::{State, Json},
    response::{IntoResponse, Response},
    http::StatusCode,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::types::{SimulationCommand, Order, OrderSide};
use crate::state::AppState;

#[derive(Debug, Clone, Deserialize)]
pub struct InjectOrderRequest {
    pub side: String,      
    pub price: u64,        
    pub quantity: u64,
    pub trader_id: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ControlRequest {
    pub action: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EarningsRequest {
    pub surprise_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TariffsRequest {
    pub severity: f64, 
}

#[derive(Debug, Clone, Deserialize)]
pub struct RugPullRequest {
    pub magnitude: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WhaleRequest {
    pub magnitude: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminResponse {
    pub success: bool,
    pub message: String,
}

pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/order", post(inject_order))
        .route("/api/admin/crash", post(trigger_crash))
        .route("/api/admin/control", post(control_simulation))
        .route("/api/admin/pump", post(pump_market))
        .route("/api/admin/dump", post(dump_market))
        .route("/api/admin/earnings", post(earnings_announcement))
        .route("/api/admin/tariffs", post(tariffs_announcement))
        .route("/api/admin/rugpull", post(rug_pull))
        .route("/api/admin/whale", post(whale_accumulation))
}

async fn inject_order(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InjectOrderRequest>,
) -> Result<Json<AdminResponse>, AdminError> {
    let side = match req.side.to_lowercase().as_str() {
        "bid" => OrderSide::Bid,
        "ask" => OrderSide::Ask,
        _ => return Err(AdminError::InvalidRequest("side must be 'bid' or 'ask'".to_string())),
    };

    let order = Order {
        id: rand::random(),
        trader_id: req.trader_id,
        side,
        price: req.price,
        amount: req.quantity,
        timestamp: 0,
    };

    state.cmd_tx.send(SimulationCommand::InjectOrder(order))
        .map_err(|_| AdminError::CommandChannelError)?;

    Ok(Json(AdminResponse {
        success: true,
        message: format!("Injected {} order: {} units @ ${:.2}", 
            req.side, req.quantity, req.price as f64 / 100.0),
    }))
}

async fn trigger_crash(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AdminResponse>, AdminError> {
    let crash_quantity = 20000;
    let crash_price = 4000;

    state.cmd_tx.send(SimulationCommand::FlashCrash {
        seller_id: 999,
        quantity: crash_quantity,
        price: crash_price,
    })
    .map_err(|_| AdminError::CommandChannelError)?;

    Ok(Json(AdminResponse {
        success: true,
        message: format!("🚨 FLASH CRASH triggered: {} units @ ${:.2}", 
            crash_quantity, crash_price as f64 / 100.0),
    }))
}

async fn control_simulation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ControlRequest>,
) -> Result<Json<AdminResponse>, AdminError> {
    let action = req.action.to_lowercase();
    let is_paused = match action.as_str() {
        "pause" => true,
        "resume" => false,
        _ => return Err(AdminError::InvalidRequest("action must be 'pause' or 'resume'".to_string())),
    };

    state.cmd_tx.send(SimulationCommand::SetPaused(is_paused))
        .map_err(|_| AdminError::CommandChannelError)?;

    Ok(Json(AdminResponse {
        success: true,
        message: format!("Simulation {}", if is_paused { "PAUSED" } else { "RESUMED" }),
    }))
}

async fn pump_market(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AdminResponse>, AdminError> {
    state.cmd_tx.send(SimulationCommand::Pump {
        buyer_id: 888,
        base_price: 0,
        magnitude: 1.0,
    })
    .map_err(|_| AdminError::CommandChannelError)?;

    Ok(Json(AdminResponse {
        success: true,
        message: "🚀 PUMP activated! 5 aggressive buy orders placed".to_string(),
    }))
}

async fn dump_market(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AdminResponse>, AdminError> {
    state.cmd_tx.send(SimulationCommand::Dump {
        seller_id: 888,
        base_price: 0,
        magnitude: 1.0,
    })
    .map_err(|_| AdminError::CommandChannelError)?;

    Ok(Json(AdminResponse {
        success: true,
        message: "📉 DUMP activated! 5 aggressive sell orders placed".to_string(),
    }))
}

async fn earnings_announcement(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EarningsRequest>,
) -> Result<Json<AdminResponse>, AdminError> {
    if req.surprise_pct < -50.0 || req.surprise_pct > 50.0 {
        return Err(AdminError::InvalidRequest(
            "surprise_pct must be between -50% and +50%".to_string()
        ));
    }

    state.cmd_tx.send(SimulationCommand::Earnings {
        surprise_pct: req.surprise_pct,
    })
    .map_err(|_| AdminError::CommandChannelError)?;

    let icon = if req.surprise_pct > 0.0 { "📈" } else { "📉" };
    Ok(Json(AdminResponse {
        success: true,
        message: format!("{} EARNINGS: {:+.1}% surprise!", icon, req.surprise_pct),
    }))
}

async fn tariffs_announcement(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TariffsRequest>,
) -> Result<Json<AdminResponse>, AdminError> {
    if req.severity < 0.0 || req.severity > 10.0 {
        return Err(AdminError::InvalidRequest(
            "severity must be between 0 and 10".to_string()
        ));
    }

    state.cmd_tx.send(SimulationCommand::Tariffs {
        severity: req.severity,
    })
    .map_err(|_| AdminError::CommandChannelError)?;

    Ok(Json(AdminResponse {
        success: true,
        message: format!("⚠️ TARIFFS announced! Severity: {:.1}", req.severity),
    }))
}

async fn rug_pull(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RugPullRequest>,
) -> Result<Json<AdminResponse>, AdminError> {
    if req.magnitude < 0.5 || req.magnitude > 5.0 {
        return Err(AdminError::InvalidRequest(
            "magnitude must be between 0.5 and 5.0".to_string()
        ));
    }

    state.cmd_tx.send(SimulationCommand::RugPull {
        magnitude: req.magnitude,
    })
    .map_err(|_| AdminError::CommandChannelError)?;

    let crash_pct = req.magnitude * 15.0;
    Ok(Json(AdminResponse {
        success: true,
        message: format!("💀 RUG PULL! Expected -{:.0}% crash", crash_pct),
    }))
}

async fn whale_accumulation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WhaleRequest>,
) -> Result<Json<AdminResponse>, AdminError> {
    if req.magnitude < 0.5 || req.magnitude > 5.0 {
        return Err(AdminError::InvalidRequest(
            "magnitude must be between 0.5 and 5.0".to_string()
        ));
    }

    state.cmd_tx.send(SimulationCommand::WhaleAccumulation {
        magnitude: req.magnitude,
    })
    .map_err(|_| AdminError::CommandChannelError)?;

    Ok(Json(AdminResponse {
        success: true,
        message: format!("🐋 WHALE ACCUMULATION detected! Magnitude: {:.1}x", req.magnitude),
    }))
}

#[derive(Debug)]
pub enum AdminError {
    InvalidRequest(String),
    CommandChannelError,
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AdminError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AdminError::CommandChannelError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to send command to simulation (channel closed?)".to_string(),
            ),
        };

        let body = Json(AdminResponse {
            success: false,
            message,
        });

        (status, body).into_response()
    }
}
