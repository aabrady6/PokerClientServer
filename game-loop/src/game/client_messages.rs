use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MessageType {
    PlayerAction {
        action: String,
        player_name: String,
        bet_amount: u32,
    },
    DiscardAction {
        player_name: String,
        card_index: Vec<usize>,
    },
    DemoMode {
        player_name: String,
        status: String,
    },
}
