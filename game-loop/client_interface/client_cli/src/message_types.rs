use serde::{Deserialize, Serialize};

/// Represents different types of messages that can be sent or received in the poker game system.
/// 
/// This enum uses serde's tagged enum representation for JSON serialization/deserialization,
/// where the variant type is stored in a "type" field.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MessageType {
    /// Represents a player's action during their turn.
    PlayerAction {
        /// The type of action taken (e.g., "check", "call", "raise", "fold")
        action: String,
        /// The name of the player taking the action
        player_name: String,
        /// The amount bet by the player (for bet or raise actions)
        bet_amount: u32,
    },
    
    /// Represents a discard action where a player selects cards to discard.
    DiscardAction {
        /// The name of the player discarding cards
        player_name: String,
        /// Indices of cards in the player's hand to discard
        card_index: Vec<usize>,
    },
    
    /// Controls the demo mode status.
    DemoMode {
        /// The name of the player toggling demo mode
        player_name: String,
        /// Current status of demo mode ("active" or "inactive")
        status: String,
    },
    
    /// Represents a selection in the statistics menu.
    StatsMenu {
        /// The type of statistics menu being accessed
        stats_menu_type: String,
        /// The specific option selected within the menu
        selected_option: String,
    },
    
    /// Signals the end of a round with various options.
    EndRound {
        /// The selected end-round option
        option: String,
        /// The dealer's selected option, if applicable
        dealer_option: String,
        /// The name of the player making the selection
        player_name: String,
    },
    
    /// Represents a game variant selection.
    GameSelection {
        /// The selected game variant (e.g., "Texas Hold'em", "Five Card Draw")
        game_var: String,
    },
    
    /// Contains login credentials for user authentication.
    UserLogin {
        /// The username for login
        username: String,
        /// The password for login (should be hashed before transmission in production)
        password: String,
    },
    
    /// Contains registration information for new user accounts.
    UserRegistration {
        /// The desired username for registration
        username: String,
        /// The password for the new account (should be hashed before transmission in production)
        password: String,
    },
}