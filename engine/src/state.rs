use tokio::sync::{broadcast, mpsc};
use crate::types::ServerMessage;

pub struct AppState {
    pub tx: broadcast::Sender<ServerMessage>,
    pub cmd_tx: mpsc::UnboundedSender<crate::types::SimulationCommand>,
}
