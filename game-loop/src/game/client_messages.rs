use serde::{Deserialize, Serialize};

/// Represents different types of messages exchanged in the system.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MessageType {
    /// Represents a player action, such as betting, calling or folding.
    PlayerAction {
        action: String,
        player_name: String,
        bet_amount: u32,
    },
    /// Represents a discard action, where a player discards specific cards.
    DiscardAction {
        player_name: String,
        card_index: Vec<usize>,
    },
    /// Message to skip all betting rounds to the final betting round
    DemoMode { player_name: String, status: String },
    /// Represents a request related to the statistics menu.
    StatsMenu {
        stats_menu_type: String,
        selected_option: String,
    },
    /// Represents the end of a round and associated options.
    EndRound {
        option: String,
        dealer_option: String,
        player_name: String,
    },
    /// Represents a user login request.
    UserLogin { username: String, password: String },
    /// Represents a user registration request.
    UserRegistration { username: String, password: String },
}
