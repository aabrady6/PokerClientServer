/// Card: The card.rs file contains two enums Value and Suit and a struct Card.
pub mod card;

/// Client Message: The client_messages.rs file contains the types of messages passed to the client
pub mod client_messages;

/// Deck: The deck.rs file contains the deck struct which calls on card to fill its vector.
pub mod deck;

/// Game State: The game_state.rs file contains the base rules for the game types.
pub mod game_state;

/// Hand: The hand.rs file contains the hand struct that contains a vector of cards and a limit.
pub mod hand;

/// Player: The player.rs file contains the player struct and PlayerChoice enum.
pub mod player;

/// Score Hands: The score_hands.rs contains the enum ScoredHand.
pub mod score_hands;

/// Server: This contains the server functions to interact between the game_state and client.
pub mod server;
