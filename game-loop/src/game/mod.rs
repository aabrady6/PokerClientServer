/// Card: The card.rs file contains two enums Value and Suit and a struct Card.
/// There are translations from the enums values to Rust universal values.
pub mod card;
/// Deck: The deck.rs file contains the deck struct which calls on card to fill its vector.
/// Calling new will automatically fill it with unique cards that together form a standard playing card deck
pub mod deck;
/// Hand: The hand.rs file contains the hand struct that contains a vector of cards and a limit.
/// When creating a new hand, pass in a value to be the hand limit, this value cannot be changed after
pub mod hand;
/// Player: The player.rs file contains the player struct and PlayerChoice enum. 
/// The enum is used as a reference for the actions that the player took and is used in a hashmap inside of player.
/// Player has two constructors inside of it, one that creates a blank player with a name and another that sets name, id, and a hand of a set size
pub mod player;
/// Score Hands: The score_hands.rs contains the enum ScoredHand.
/// The file itself overrides the base Rust operators of comparing and Displaying to allow the comparing of two different ScoredHands and displaying the cards with the suits stripped.
/// The intention is to pass in a hand of exactly five cards and an output will be returned of ScoredHand, and to use the ScoredHand against other ScoredHand's.
pub mod score_hands;
/// Base Rules: The base_rules.rs contains GameState; the difference in the naming is that base_rules.rs contains the base universal rules of poker which by virtue also has the GameState.
/// GameState contains the core logic of the poker game that other files will call upon to update, retrieve, and push. 
/// GameState is generalized with many basic calls for individual poker games to call and execute to suit their needs.
pub mod base_rules;
/// Server: The server.rs contains the struct server, this is a generalized template type where we input a game type (struct) such as Five Card Draw or Texas Hold'em.
/// Server also gives us a way to interface with other users and the database in part 3.
pub mod server;
/// Poker Game: The poker_game.rs contains the base template functions that the individual poker types will call on.
/// The universal nature of this allows server to call functions that ensure that each poker game can initalize and run. 
pub mod poker_game;
