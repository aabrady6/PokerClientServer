/// This module provides the menu system for the game, including login, user creation,
/// and various game-related options.
///
/// Functions in this module handle user input, database interactions, and menu navigation.
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use mongodb::bson;
use mongodb::bson::{doc, from_document, to_document, Document};
use serde::{Deserialize, Serialize};

use crate::db::dbclient::DbEntity;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use std::collections::HashMap;
use std::str::FromStr;

/// Represents a player's action during a game.
///
/// This enum defines the possible actions a player can take, such as checking, folding, calling, or betting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlayerChoice {
    Check,
    Fold,
    Call(u32),
    Raise(u32),
    Bet(u32),
    PlacedInPot(u32),
    AllIn,
}

impl ToString for PlayerChoice {
    /// Converts a `PlayerChoice` variant into a human-readable string.
    fn to_string(&self) -> String {
        match *self {
            PlayerChoice::Check => "Check".to_string(),
            PlayerChoice::Fold => "Fold".to_string(),
            PlayerChoice::Call(_) => "Call".to_string(),
            PlayerChoice::Raise(_) => "Raise".to_string(),
            PlayerChoice::Bet(_) => "Bet".to_string(),
            PlayerChoice::PlacedInPot(_) => "PlacedInPot".to_string(),
            PlayerChoice::AllIn => "AllIn".to_string(),
        }
    }
}

impl FromStr for PlayerChoice {
    type Err = &'static str;

    /// Parses a string into a `PlayerChoice` variant.
    ///
    /// Inputs:
    /// - `s`: A string slice containing the action name.
    ///
    /// Outputs:
    /// - `Ok(PlayerChoice)` if the input is valid.
    /// - `Err("invalid action")` if the input does not match any variant.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();

        match s.as_str() {
            "Check" => Ok(PlayerChoice::Check),
            "Fold" => Ok(PlayerChoice::Fold),
            "Call" => Ok(PlayerChoice::Call(0)),
            "Raise" => Ok(PlayerChoice::Raise(0)),
            "PlacedInPot" => Ok(PlayerChoice::PlacedInPot(0)),
            "AllIn" => Ok(PlayerChoice::AllIn),
            _ => Err("invalid action"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Holds important information about a player, including id,
/// name, money, games, wins, and so on.
/// This information is used for both short-term and long-term use and storage.
pub struct Player {
    pub player_id: u32,
    pub player_name: String,
    pub hashed_password: String,
    pub player_money: u32,
    pub round_win: u32,
    pub last_move: String,
    pub total_games: u32,
    pub total_wins: u32,
    pub total_losses: u32,
    pub total_earnings: u32,
    pub total_wagered_per_game: u32,
    pub player_hand: Hand,
    pub token: String,
    pub player_choices: HashMap<String, PlayerChoice>,
}

impl Player {
    /// Creates a new player instance with default values.
    ///
    /// This function initializes a player with a default hand size of 0 and assigns them a starting balance.
    /// Different game types may use different hand sizes, but this function does not assign one.
    ///
    /// Inputs:
    /// - `player_name`: A string slice representing the player's name.
    ///
    /// Outputs:
    /// - Returns a new `Player` instance with default values.
    pub fn new(player_name: &str) -> Self {
        let player_hand = Hand::new(0);
        let player_name = player_name.to_string();
        let hashed_password = "".to_string();
        let mut player_choices = HashMap::new();
        player_choices.insert("PlacedInPot".to_string(), PlayerChoice::PlacedInPot(0));
        Self {
            player_id: 0,
            player_name,
            hashed_password,
            player_money: 1000,
            total_games: 0,
            total_wins: 0,
            round_win: 0,
            last_move: "".to_string(),
            total_losses: 0,
            total_earnings: 0,
            total_wagered_per_game: 0,
            player_hand,
            token: "".to_string(),
            player_choices,
        }
    }

    pub fn empty() -> Self {
        Self {
            player_id: 0,
            player_name: String::new(),
            hashed_password: String::new(),
            player_money: 0,
            round_win: 0,
            total_games: 0,
            total_wins: 0,
            total_losses: 0,
            total_earnings: 0,
            last_move: "".to_string(),
            total_wagered_per_game: 0,
            token: "".to_string(),
            player_hand: Hand::new(0),
            player_choices: HashMap::new(),
        }
    }

    /// Converts the player object into a BSON document for database storage.
    ///
    /// Outputs:
    /// - Returns a `Result<Document, String>` containing the BSON representation of the player or an error message.
    pub fn to_document(&self) -> Result<Document, String> {
        to_document(self).map_err(|e| e.to_string())
    }

    /// Creates a player instance from a BSON document.
    ///
    /// Inputs:
    /// - `doc`: A reference to a BSON `Document` containing player data.
    ///
    /// Outputs:
    /// - Returns a `Result<Self, String>` containing the deserialized player object or an error message.
    pub fn from_document(doc: &Document) -> Result<Self, String> {
        from_document(doc.clone()).map_err(|e| e.to_string())
    }

    /// Alternative constructor to create a new player with specified initial values.
    ///
    /// Unlike `new`, this constructor allows specifying the player's hand size, name, and ID.
    ///
    /// Inputs:
    /// - `hand_size`: The size of the player's hand, specific to the game type.
    /// - `player_name`: A `String` representing the player's name.
    /// - `player_id`: A `u32` representing the player's unique identifier.
    ///
    /// Outputs:
    /// - Returns a new `Player` instance with specified attributes.
    pub fn new_player(
        player_name: String,
        player_password: String,
        player_id: u32,
    ) -> Result<Self, String> {
        let hashed_password = Self::hash_password(&player_password)?;
        let player_hand = Hand::new(0);
        let player_choices = HashMap::new();

        Ok(Self {
            player_hand,
            player_name,
            hashed_password,
            player_id,
            player_money: 1000,
            round_win: 0,
            player_choices,
            last_move: "".to_string(),
            total_games: 0,
            total_wins: 0,
            total_losses: 0,
            total_wagered_per_game: 0,
            token: "".to_string(),
            total_earnings: 0,
        })
    }

    pub fn hash_password(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(password_hash) => Ok(password_hash.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(&self.hashed_password);
        match parsed_hash {
            Ok(parsed) => Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok(),
            Err(_) => false,
        }
    }

    //
    // These getters and setters are found with https://stackoverflow.com/questions/35390615/writing-getter-setter-properties-in-rust
    // These I dont want to pass through immediately but instead want to leave open to usage
    // Later can easily pass this into a constructor to instantiate it but for now we might want to change and get
    // in relation to communication
    pub fn set_name(&mut self, new_name: String) {
        self.player_name = new_name;
    }

    pub fn set_id(&mut self, new_id: u32) {
        self.player_id = new_id
    }

    pub fn get_money(&self) -> &u32 {
        &self.player_money
    }

    pub fn set_money(&mut self, new_money: u32) {
        self.player_money = new_money;
    }

    pub fn get_name(&self) -> &String {
        &self.player_name
    }

    pub fn get_id(&self) -> &u32 {
        &self.player_id
    }

    pub fn get_games(&self) -> u32 {
        self.total_games
    }

    pub fn set_games(&mut self, new_games: u32) {
        self.total_games = new_games;
    }

    pub fn get_wins(&self) -> u32 {
        self.total_wins
    }

    pub fn set_wins(&mut self, new_wins: u32) {
        self.total_wins = new_wins;
    }

    pub fn get_losses(&self) -> u32 {
        self.total_losses
    }

    pub fn set_losses(&mut self, new_losses: u32) {
        self.total_losses = new_losses;
    }

    pub fn get_earnings(&self) -> u32 {
        self.total_earnings
    }

    pub fn set_earnings(&mut self, new_earnings: u32) {
        self.total_earnings = new_earnings;
    }

    pub fn set_hand(&mut self, hand: Hand) {
        self.player_hand = hand
    }

    pub fn infinite_money(&mut self) {
        if self.player_money < 10 {
            self.player_money = 1000;
            println!(
                "INFINITE MONEY: {:?} bank refilled to $1000",
                self.player_name
            );
        }
    }

    /// Getter for "Call"
    ///
    /// This function retrieves the player's choice for the "Call" action. If no such action is found,
    /// it returns an error with a message indicating that the player has not made a "Call" choice.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// let bet_choice = player.get_call();
    /// ```
    pub fn get_call(&self) -> Result<PlayerChoice, String> {
        match self.player_choices.get("Call") {
            Some(choice) => Ok(choice.clone()),
            None => Err(format!(
                "No Call action found for player: {}",
                self.get_name()
            )),
        }
    }

    /// Getter for "Raise"
    ///
    /// This function retrieves the player's choice for the "Raise" action. If no such action is found,
    /// it returns an error with a message indicating that the player has not made a "Raise" choice.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// let bet_choice = player.get_raise();
    /// ```
    pub fn get_raise(&self) -> Result<PlayerChoice, String> {
        match self.player_choices.get("Raise") {
            Some(choice) => Ok(choice.clone()),
            None => Err(format!(
                "No Raise action found for player: {}",
                self.get_name()
            )),
        }
    }

    /// Getter for "Bet"
    ///
    /// This function retrieves the player's choice for the "Bet" action. If no such action is found,
    /// it returns an error with a message indicating that the player has not made a "Bet" choice.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// let bet_choice = player.get_bet();
    /// ```
    pub fn get_bet(&self) -> Result<PlayerChoice, String> {
        match self.player_choices.get("Bet") {
            Some(choice) => Ok(choice.clone()),
            None => Err(format!(
                "No Bet action found for player: {}",
                self.get_name()
            )),
        }
    }

    /// Getter for "Check"
    ///
    /// This function retrieves the player's choice for the "Check" action. If no such action is found,
    /// it returns an error with a message indicating that the player has not made a "Check" choice.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// let check_choice = player.get_check();
    /// ```
    pub fn get_check(&self) -> Result<PlayerChoice, String> {
        match self.player_choices.get("Check") {
            Some(choice) => Ok(choice.clone()),
            None => Err(format!(
                "No Check action found for player: {}",
                self.get_name()
            )),
        }
    }

    /// Getter for "Fold"
    ///
    /// This function retrieves the player's choice for the "Fold" action. If no such action is found,
    /// it returns an error with a message indicating that the player has not made a "Fold" choice.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// let fold_choice = player.get_fold();
    /// ```
    pub fn get_fold(&self) -> Result<PlayerChoice, String> {
        match self.player_choices.get("Fold") {
            Some(choice) => Ok(choice.clone()),
            None => Err(format!(
                "No Fold action found for player: {}",
                self.get_name()
            )),
        }
    }

    /// Getter for "PlacedInPot"
    ///
    /// This function retrieves the player's choice for the "PlacedInPot" action. If no such action is found,
    /// it returns an error with a message indicating that the player has not made a "PlacedInPot" choice.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// let placed_in_pot_choice = player.get_placedinpot();
    /// ```
    pub fn get_placedinpot(&self) -> Result<PlayerChoice, String> {
        match self.player_choices.get("PlacedInPot") {
            Some(choice) => Ok(choice.clone()),
            None => Err(format!(
                "No Placed In Pot action found for player: {}",
                self.get_name()
            )),
        }
    }

    /// Getter for "All In"
    ///
    /// This function retrieves the player's choice for the "All In" action. If no such action is found,
    /// it returns an error with a message indicating that the player has not made an "All In" choice.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// let all_in_choice = player.get_all_in();
    /// ```
    pub fn get_all_in(&self) -> Result<PlayerChoice, String> {
        match self.player_choices.get("AllIn") {
            Some(choice) => Ok(choice.clone()),
            None => Err(format!(
                "No All In action found for player: {}",
                self.get_name()
            )),
        }
    }

    /// Adds money to the player's balance.
    ///
    /// This function increases the player's money by a specified amount. It assumes that the
    /// additional money is valid and that the player should not exceed any upper limit or go negative
    /// (though this error checking has not been implemented yet).
    ///
    /// Inputs:
    /// additional - a u32 for the amount of money we want to add to the players active money
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.add_money(100 as u32);
    /// ```
    pub fn add_money(&mut self, additional: u32) {
        self.player_money += additional;
    }

    /// Removes money from the player's balance.
    ///
    /// This function decreases the player's money by a specified amount. It assumes that the player
    /// has sufficient funds to cover the removal (error checking is not implemented).
    ///
    /// Inputs:
    /// removal - a u32 for the amount of money we want to remove from the players active money
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.remove_money(50 as u32);
    /// ```
    pub fn remove_money(&mut self, removal: u32) {
        self.player_money -= removal;
    }

    /// Draws a single card from the deck and adds it to the player's hand.
    ///
    /// This function attempts to draw a card from the provided deck. If successful, it adds the card
    /// to the player's hand. If drawing the card fails (e.g., the deck is empty), it returns an error.
    /// We are passing the deck further to the hand to execute the actual drawing of the card and here we are handling
    /// the error and passing the responsibility back up to the caller of draw_card
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// let mut deck = Deck::new();
    /// player.draw_card(&mut deck);
    /// ```
    pub fn draw_card(&mut self, deck: &mut Deck) -> Result<(), &'static str> {
        match self.player_hand.draw(deck, 1) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Resets the player's choices for the round.
    ///
    /// This function clears all choices made by the player for the current round, essentially
    /// resetting them so they can start fresh for the next round.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.reset_choices();
    /// ```
    pub fn reset_choices(&mut self) {
        self.player_choices.clear();
        self.last_move.clear();
        self.total_wagered_per_game = 0;
    }

    /// Prints the player's action choices for the current round.
    ///
    /// This function outputs all of the player's choices for the current round, printing each
    /// action and its associated value (e.g., "Call" or "Raise"). It can be used for debugging or
    /// displaying the player's actions during gameplay.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.print_enums();
    /// ```
    pub fn print_enums(&self) {
        println!("=============");
        println!("{}'s choices:", self.get_name());

        for (key, choice) in &self.player_choices {
            println!("{}: {}", key, choice.to_string());
        }
    }

    /// Updates the player's choices for a new round, keeping only the "Fold" action.
    ///
    /// This function retains only the "Fold" action in the player's choices, removing all other
    /// actions from the previous round. It is used at the start of a new round when the player may
    /// only have the option to fold.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.new_round_choices();
    /// ```
    pub async fn new_round_choices(&mut self) {
        self.player_choices.retain(|key, _| key == "Fold");
        self.player_choices
            .insert("PlacedInPot".to_string(), PlayerChoice::PlacedInPot(0));
    }

    /// Conduct the player action check
    ///
    /// We want to confirm that the player has not folded yet, if they have then we cannot check.
    /// Checking means we want to only retain the amount the player has placed in pot, this was
    /// originally from placed in pot being the amount the player has placed in total for the round.
    /// This was changed so now its per betting round and not needed anymore
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.check();
    ///
    /// ```
    pub fn check(&mut self) {
        if !self.player_choices.contains_key("Fold") {
            self.player_choices.retain(|key, _| key == "PlacedInPot");
            self.player_choices
                .insert("Check".to_string(), PlayerChoice::Check);
        } else {
            // Handle error
        }
    }

    /// Conduct the player action fold
    ///
    /// We want to confirm that the player has not folded yet, if they have then we cannot fold again.
    /// Folding we dont care about any other actions and as a result want to only keep the amount placed in pot.
    /// This again was from before and was kept for stats and can still be used for stats.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.fold();
    ///
    /// ```
    pub fn fold(&mut self) {
        let _compare = "PlacedInPot".to_string();
        self.player_choices.retain(|key, _| matches!(key, _compare));
        self.player_choices
            .insert("Fold".to_string(), PlayerChoice::Fold);
    }

    /// Conduct the player action call
    ///
    /// We want to confirm that the player has not folded yet, if they have then we cannot call.
    /// Calling we dont care about any other actions and as a result want to only keep the amount placed in pot.
    /// This again was from before and was kept for stats and can still be used for stats.
    ///
    /// Inputs:
    ///  call_amount - a u32 that represents how much we are putting as the call amount, this amount should be determined
    ///                by the game and passed back to this function for tracking
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.call(100 as u32);
    ///
    /// ```
    pub fn call(&mut self, call_amount: u32) {
        if !self.player_choices.contains_key("Fold") {
            self.player_choices.retain(|key, _| key == "PlacedInPot");
            let current_call = self
                .player_choices
                .entry("Call".to_string())
                .or_insert(PlayerChoice::Call(0));
            if let PlayerChoice::Call(ref mut amount) = current_call {
                *amount = call_amount;
            }
            self.add_to_placed_in_pot(call_amount);
            self.remove_money(call_amount);
            // println!("Player call {}", self.get_id());
        } else {
            // Handle error if the player has folded
        }
    }

    /// Conduct the player action Bet
    ///
    /// We want to confirm that the player has not folded yet, if they have then we cannot bet.
    /// For betting we dont care about any other actions and as a result want to only keep the amount placed in pot.
    /// This again was from before and was kept for stats and can still be used for stats.
    /// This is action shouldn't be used for the current games that are going to be implimented but other games might want this action.
    ///
    /// Inputs:
    ///  bet_amount - a u32 that represents how much we are putting as the bet amount, this amount should be determined
    ///                by the game and passed back to this function for tracking
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.bet(100 as u32);
    /// ```
    pub fn bet(&mut self, bet_amount: u32) {
        if let Some(PlayerChoice::PlacedInPot(ref mut amount)) =
            self.player_choices.get_mut("PlacedInPot")
        {
            *amount += bet_amount;
        } else {
            self.player_choices.insert(
                "PlacedInPot".to_string(),
                PlayerChoice::PlacedInPot(bet_amount),
            );
        }
        self.player_money -= bet_amount;
        self.total_wagered_per_game += bet_amount;
    }

    /// Call to update or insert the PlacedInPot enum into the players choices
    ///
    /// The main if not only calls for this are inside of the other enum calls in player.
    /// Upon a call to call or raise we want to update this but this call can also be used for other things like placing blinds.
    ///
    /// Inputs:
    ///  amount - a u32 that represents how much we are updating or putting into the enum to store as a stat.
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.add_to_placed_in_pot(100 as u32);
    /// ```
    pub fn add_to_placed_in_pot(&mut self, amount: u32) {
        let current_pot = self
            .player_choices
            .entry("PlacedInPot".to_string())
            .or_insert(PlayerChoice::PlacedInPot(0));
        if let PlayerChoice::PlacedInPot(ref mut pot) = current_pot {
            *pot += amount;
            self.total_wagered_per_game += amount;
        }
    }

    /// This call is instead of a clear we are setting a reference to a brand new struct
    /// This is incase we want to fully reset all assocaited data and references
    ///
    /// ```
    /// let mut player = Player::new("TestPlayer");
    /// player.reset_hand();
    /// ```
    pub fn reset_hand(&mut self) {
        self.player_hand = Hand::new(self.player_hand.get_hand_limit());
    }
}

impl DbEntity for Player {
    fn collection_name() -> &'static str {
        "players"
    }

    fn to_document(&self) -> Result<Document, String> {
        Ok(doc! {
            "player_id": self.player_id as i64,
            "player_name": self.player_name.clone(),
            "hashed_password": self.hashed_password.clone(),
            "player_money": self.player_money as i64,
            "total_games": self.total_games as i64,
            "total_wins": self.total_wins as i64,
            "round_win": self.round_win as i64,
            "total_losses": self.total_losses as i64,
            "total_earnings": self.total_earnings as i64,
            "total_wagered_per_game": self.total_wagered_per_game as i64,
            "player_hand": bson::to_bson(&self.player_hand).map_err(|e| e.to_string())?,
            "player_choices": bson::to_bson(&self.player_choices).map_err(|e| e.to_string())?,
            "last_move": self.last_move.clone(),
            "token": self.token.clone(),
        })
    }

    fn from_document(doc: &Document) -> Result<Self, String> {
        Ok(Player {
            player_id: doc.get_i64("player_id").map_err(|e| e.to_string())? as u32,
            player_name: doc
                .get_str("player_name")
                .map_err(|e| e.to_string())?
                .to_string(),
            hashed_password: doc
                .get_str("hashed_password")
                .map_err(|e| e.to_string())?
                .to_string(),
            player_money: doc.get_i64("player_money").map_err(|e| e.to_string())? as u32,
            total_games: doc.get_i64("total_games").map_err(|e| e.to_string())? as u32,
            total_wins: doc.get_i64("total_wins").map_err(|e| e.to_string())? as u32,
            total_losses: doc.get_i64("total_losses").map_err(|e| e.to_string())? as u32,
            round_win: doc.get_i64("round_win").map_err(|e| e.to_string())? as u32,
            total_earnings: doc.get_i64("total_earnings").map_err(|e| e.to_string())? as u32,
            total_wagered_per_game: doc
                .get_i64("total_wagered_per_game")
                .map_err(|e| e.to_string())? as u32,
            last_move: doc
                .get_str("last_move")
                .map_err(|e| e.to_string())?
                .to_string(),
            player_hand: bson::from_bson(
                doc.get("player_hand").cloned().unwrap_or(bson::Bson::Null),
            )
            .map_err(|e| e.to_string())?,
            token: doc.get_str("token").map_err(|e| e.to_string())?.to_string(),
            player_choices: bson::from_bson(
                doc.get("player_choices")
                    .cloned()
                    .unwrap_or(bson::Bson::Null),
            )
            .map_err(|e| e.to_string())?,
        })
    }

    fn unique_field(&self) -> Document {
        doc! { "player_name": self.player_name.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new("TestPlayer");
        assert_eq!(player.player_name, "TestPlayer");
        assert_eq!(player.player_money, 1000);
        assert!(player.player_choices.contains_key("PlacedInPot"));
    }

    #[test]
    fn test_player_setters_getters() {
        let mut player = Player::new("TestPlayer");
        player.set_name("NewName".to_string());
        player.set_money(500);
        player.set_games(10);
        player.set_wins(5);
        player.set_losses(5);
        player.set_earnings(1000);

        assert_eq!(player.get_name(), "NewName");
        assert_eq!(*player.get_money(), 500);
        assert_eq!(player.get_games(), 10);
        assert_eq!(player.get_wins(), 5);
        assert_eq!(player.get_losses(), 5);
        assert_eq!(player.get_earnings(), 1000);
    }

    #[test]
    fn test_money_operations() {
        let mut player = Player::new("TestPlayer");
        player.add_money(500);
        assert_eq!(*player.get_money(), 1500);

        player.remove_money(300);
        assert_eq!(*player.get_money(), 1200);
    }

    #[test]
    fn test_player_document_conversion() {
        let player = Player::new("TestPlayer");
        let doc = player.to_document().unwrap();
        let new_player = Player::from_document(&doc).unwrap();
        assert_eq!(player.player_name, new_player.player_name);
    }
}
