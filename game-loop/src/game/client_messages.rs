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
    StatsMenu {
        stats_menu_type: String,
        selected_option: String,
    },
    EndRound {
        option: String,
        dealer_option: String,
    },
    GameSelection {
        game_var: String,
    },
    UserLogin {
        username: String,
        password: String,
    },
    UserRegistration {
        username: String,
        password: String,
    },
}
