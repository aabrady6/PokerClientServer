//! Core game rules that are common amongst poker variants and poker game values
//!
//! Creates calls to structs that are inside of GameState and helps perform passing
//! of references and data between these structs.
//! GameState meant to be by in large asynchronous with updates being made to the database on game completion
//!

use crate::db::dbclient::{DbClient, DbEntity};
use mongodb::bson;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};

use crate::game::card::Card;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::player::Player;
use crate::game::player::PlayerChoice;
use crate::game::score_hands::ScoredHand;
use std::cmp::max;
use std::io;
use std::io::Write;

/// An enum to represent the type of input for a given operation.
/// It can either hold a list of strings or be empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestInput {
    Input(Vec<String>),
    Empty,
}

/// An enum used to indicate the current load state of the game.
/// This helps track whether a game state exists or if it's empty.
pub enum LoadState {
    GameExist(GameState),
    Empty,
}

/// A struct that represents the state of the game, containing essential information about the game setup,
/// players, and current game status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub game_variant: String,
    pub players: Vec<Player>,
    pub deck: Deck,
    pub game_id: u32,
    pub hand_size: u8,
    pub max_players: i64,
    pub pot: u32,
    pub stage_number: u32,
    pub round_number: u32,
    pub minimum_bet: u32,
    pub highest_bet: u32,
    pub dealer: u32,
    pub winner: Vec<Player>,
}

// Implementation of functions that can be used to manipulate or interact with the GameState
impl GameState {
    /// A constructor to create a new, empty game state. The game will have default values.
    ///
    /// Arguments:
    /// * `id`: The unique identifier for the new game instance.
    ///
    /// Returns:
    /// * A new `GameState` object with default values and the provided `game_id`.
    pub fn empty_constructor(id: u32) -> Self {
        Self {
            players: Vec::new(),
            deck: Deck::new(),
            game_variant: "".to_string(),
            game_id: id,
            hand_size: 5,
            max_players: 5,
            pot: 0,
            stage_number: 0,
            round_number: 0,
            minimum_bet: 10, // Default values
            highest_bet: 0,
            dealer: 0,
            winner: vec![],
        }
    }

    /// A constructor to create a new game instance with a given `game_id`. This initializes an empty set of players
    /// and a fresh deck, but the game is not fully configured yet.
    ///
    /// Inputs:
    /// id - The unique identifier for the new game instance.
    ///
    /// Returns:
    /// * A new `GameState` object with the provided `game_id`, an empty set of players, and other default values.
    ///
    /// TODO: Implement error checking to ensure that the number of players is within a valid range.
    pub fn new_game(id: u32) -> Self {
        let players: Vec<Player> = Vec::new();
        let deck = Deck::new();
        Self {
            players,
            deck,
            game_id: id,
            game_variant: "".to_string(),
            hand_size: 0,
            max_players: 0,
            pot: 0,
            stage_number: 1,
            round_number: 0,
            minimum_bet: 10,
            highest_bet: 0,
            dealer: 0,
            winner: vec![],
        }
    }

    /// Setter for the game id
    pub fn set_game_id(&mut self, new_game_id: u32) {
        self.game_id = new_game_id;
    }

    /// Getter for the game id
    pub fn get_game_id(&self) -> &u32 {
        &self.game_id
    }
    /// Bool for now until I figure out the error
    pub fn set_highest_bet(&mut self, new_bet: u32) -> bool {
        if new_bet > self.highest_bet {
            self.highest_bet = new_bet;
            return true;
        }
        false
    }

    /// Getter for the highest bet that occured in the betting loop
    pub fn get_highest_bet(&self) -> &u32 {
        &self.highest_bet
    }

    /// Getter for the string that is the variant that the Game State is being called from
    pub fn get_variant(&self) -> &str {
        &self.game_variant
    }

    /// Setter for the string that is the variant that the Game State is being called from
    pub fn set_variant(&mut self, var: &String) {
        self.game_variant = var.to_string()
    }

    /// Getting the length of the player vector and returning it as a u8
    pub fn players_in_game(&self) -> u8 {
        self.players.len().try_into().unwrap()
    }

    /// Getter to get the reference to Game States vector of players in the game
    pub fn players(&self) -> &Vec<Player> {
        &self.players
    }

    /// Getter to get the reference to Game States deck
    pub fn deck(&self) -> &Deck {
        &self.deck
    }

    /// Inserts a new player into the game session.
    ///
    /// This function checks if the maximum player limit has been reached before inserting.
    ///
    /// Inputs:
    /// - `player`: A reference to a `Player` instance.
    ///
    /// Outputs:
    /// - Returns `Ok(())` if the player was successfully inserted.
    /// - Returns `Err(String)` if the player limit has been exceeded.
    pub fn insert_player(&mut self, player: &Player) -> Result<(), String> {
        //if self.max_players > self.players().len() {
        if 5 <= self.players.len() {
            return Err("Unable to push".to_string());
        }
        self.players.push(player.clone());

        Ok(())
    }

    /// Removes a player from the game session based on their unique ID.
    ///
    /// This function searches for a player by their ID and removes them if found.
    ///
    /// Inputs:
    /// - `p_id`: A `u32` representing the player's unique identifier.
    ///
    /// Outputs:
    /// - Returns `true` if the player was found and removed.
    /// - Returns `false` if no matching player was found.
    ///
    /// TODO:
    /// - Implement error checking to handle invalid cases properly.
    #[allow(dead_code)]
    fn remove_player(&mut self, p_id: u32) -> bool {
        // Attempt to find the player by p_id
        if let Some(index) = self
            .players
            .iter()
            .position(|player| player.get_id().clone() == p_id)
        {
            // Remove the player at the found index
            self.players.remove(index);
            return true;
        }
        false
    }

    /// Function to move the dealer chip over one player, the dealer chip is what the blinds and who starts is based off of
    ///
    /// Examples:
    /// ```
    /// let mut game = GameState::new_game(1);
    /// let mut player = Player::new_player(5 as u8, "David".to_string(), 1);
    /// game.insert_player(&player1).unwrap();
    /// ```
    pub fn rotate_blinds(&mut self) {
        self.dealer = (self.dealer + 1) % self.players.len() as u32;
    }

    /// Function to call when wanting to put money into the pot for a specific player, calls using the p_id help
    /// ensure syncroncity with the database and server
    ///
    /// Inputs:
    /// p_id - a u32 player ID to get the player that is putting money into the pot
    ///
    /// amount - a u32 for the associated amount of money that the player is putting into the pot
    ///
    /// Examples:
    /// ```
    /// let mut game = GameState::new_game(1);
    /// let mut player1 = Player::new_player(5 as u8, "P1".to_string(), 1);
    /// let mut player2 = Player::new_player(5 as u8,"P2".to_string(), 2);
    /// let mut player3 = Player::new_player( 5 as u8, "P3".to_string(), 3);
    /// let mut player4 = Player::new_player( 5 as u8, "P5".to_string(), 4);
    /// player1.set_money(1000 as u32);
    /// player2.set_money(1000 as u32);
    /// player3.set_money(1000 as u32);
    /// player4.set_money(1000 as u32);
    /// game.insert_player(&player1).unwrap();
    /// game.insert_player(&player2).unwrap();
    /// game.insert_player(&player3).unwrap();
    /// game.insert_player(&player4).unwrap();
    /// game.pot = 500;
    /// game.put_into_pot(1, 100);
    /// ```
    pub fn put_into_pot(&mut self, p_id: u32, amount: u32) {
        if let Some(index) = self
            .players
            .iter()
            .position(|player| player.get_id().clone() == p_id)
        {
            self.players[index].remove_money(amount);
            self.pot += amount;
        }
    }

    /// Given a specific player ID that is in the game we are giving the entire pot to that player upon successful
    /// obtaining of the reference to the player struct. Because we are calling this upon round completion we
    /// are also changing the wins and losses for the associated players within the current game.
    ///
    /// Inputs:
    /// p_id - a u32 of the player id for the player that we want to give the pot to
    ///
    /// highest_bet - a u32 for the current highest amount a single player has placed into the pot for the betting round
    ///
    /// Examples:
    /// ```rust
    /// let mut game = GameState::new_game(1);
    /// let mut player1 = Player::new_player(5 as u8, "David".to_string(), 1);
    /// let mut player2 = Player::new_player(5 as u8, "Frank".to_string(), 2);
    /// player.set_money(1000 as u32);
    /// game.insert_player(&player1).unwrap();
    /// game.insert_player(&player2).unwrap();
    /// game.pot = 500;
    /// game.give_pot(1);
    /// ```
    pub fn give_pot(&mut self, p_id: u32) {
        let mut winner_found = false;

        for player in &mut self.players {
            if player.get_id() == &p_id {
                let new_wins = player.get_wins() + 1;
                let new_games = player.get_games() + 1;
                let new_earnings = player.get_earnings() + self.pot;
                player.add_money(self.pot);
                player.set_wins(new_wins);
                player.set_games(new_games);
                player.set_earnings(new_earnings);
                player.round_win = self.pot as i64;

                winner_found = true;
            } else {
                // Update losses for non-winners
                let new_losses = player.get_losses() + 1;
                let new_games = player.get_games() + 1;

                player.set_losses(new_losses);
                player.set_games(new_games);
                player.round_win = 0;
            }
        }

        if winner_found {
            self.pot = 0;
        }
    }

    pub fn pay_out(&mut self, winners: Vec<(u32, ScoredHand)>) {
        println!("{}", "=".repeat(25));

        if self.get_remaining_player_count() <= 1 {
            let winner_index = winners[0].0 as usize;
            if let Some(winner) = self.players.get(winner_index) {
                println!(
                    "{:?} won the round. All other players folded",
                    winner.player_name
                );
                println!("{:?} wins {:?} chips!", winner.player_name, self.pot);
                self.winner.push(winner.clone());
                self.give_pot(*winner.get_id());
                println!("{}", "=".repeat(25));
                return;
            }
        }

        if winners.len() == 1 {
            let winner_index = winners[0].0 as usize;

            if let Some(winner) = self.players.get(winner_index) {
                let winning_hand = winners[0].1;

                println!(
                    "{:?} Won the round. Winning hand: {:?}",
                    winner.player_name, winning_hand
                );
                println!("{:?} wins {:?} chips!", winner.player_name, self.pot);

                self.winner.push(winner.clone());
                self.give_pot(*winner.get_id());
            }
        } else {
            println!("It's a tie between the following players:");

            for (winner_id, best_hand) in &winners {
                if let Some(winner) = self.players.iter().find(|p| p.get_id() == winner_id) {
                    println!("{:?}; Hand: {:?}", winner.get_name(), best_hand);
                    self.winner.push(winner.clone());
                }
            }

            println!(
                "Each player wins {:?} chips!",
                self.pot / (self.winner.len() as u32)
            );

            self.split_pot(&winners.iter().map(|(id, _)| *id).collect::<Vec<u32>>());
        }
        println!("{}", "=".repeat(25));
    }

    /// Function that is similar to give_pot but instead we are passing in a list of players.
    /// The pot is then evenly split between the players and any decimals are chopped off
    ///
    /// Inputs:
    /// player_list - a `&Vec<u32>` containing the player ids of those who we want to split the pot between
    ///
    ///
    /// Example:
    /// ```
    ///  let mut game = GameState::new_game(1);
    ///  let mut player1 = Player::new_player(5 as u8, "Eve".to_string(), 1);
    ///  let mut player2 = Player::new_player(5 as u8, "Frank".to_string(), 2);
    ///  player1.set_money(1000 as u32);
    ///  player2.set_money(1000 as u32);
    ///  game.insert_player(&player1).unwrap();
    ///  game.insert_player(&player2).unwrap();
    ///  game.pot = 500;
    ///  game.split_pot(&vec![1, 2]);
    /// ```
    pub fn split_pot(&mut self, player_list: &Vec<u32>) {
        let num_players = player_list.len() as u32;
        for winner_id in player_list {
            if let Some(index) = self
                .players
                .iter()
                .position(|player| player.get_id().clone() == winner_id.clone())
            {
                self.players[index].add_money(self.pot / num_players);
            }
        }
        self.pot = 0;
    }

    pub fn add_to_pot(&mut self, amount: u32) {
        self.pot += amount;
    }

    /// A more general method to draw a card from the deck, which is not tied to a specific player or
    /// hand. This function is useful for other game actions or for drawing cards that do not belong
    /// to any player’s hand.
    ///
    /// Returns:
    /// This function returns a `Result<Card, &'static str>` that contains if the desired action was successful or not. If
    /// the card is successfully drawn, it returns `Ok(Card)`. If an error occurs, it returns an error
    /// message as a string reference (`Err(&'static str)`).
    ///
    /// Example:
    /// ```
    /// match game.draw_card_from_deck() {
    ///     Ok(card) => println!("Card drawn: {:?}", card),
    ///     Err(e) => println!("Error drawing card: {}", e),
    /// }
    /// ```
    pub fn draw_card_from_deck(&mut self) -> Result<Card, &'static str> {
        self.deck.deal_card()
    }

    /// Function to print the cards of a player given a certain name
    ///
    /// Inputs:
    /// p_name - a &str for the players name to index and find that player in the game
    ///
    /// Examples:
    /// ```
    /// let mut game = Game_State::empty_constructor(1);
    /// let mut player1 = Player::new("P1");
    /// let mut player2 = Player::new("P2");
    /// let mut player3 = Player::new("P3");
    /// game.insert_player(&player1).unwrap();
    /// game.insert_player(&player2).unwrap();
    /// game.insert_player(&player3).unwrap();
    /// game.deal_hands();
    /// for player in &game.players{
    ///     game.print_player_cards(player.get_name());
    /// }
    ///
    /// ```
    pub fn print_player_cards(&self, p_name: &str) {
        print!("Player {} card's: ", p_name);
        if let Some(index) = self
            .players
            .iter()
            .position(|player| player.get_name().clone() == p_name)
        {
            for card in self.players[index].player_hand.get_hand_cards() {
                print!("{}, ", card);
            }
        }
        println!();
    }

    /// Function to ask the player what their next desired action to take is given the current game state
    ///
    /// Possible actions are inputted into a vector given the current players stats, and the games current state.
    /// An example of this is the ability to check only if no other player has put in money for the round.
    /// This vector is then shown to the user for their selection of available actions and this is then passed
    /// into do_player_action to process the logic for the given action.
    ///
    /// Inputs:
    /// player - a mutable reference to the player we are doing the action for
    ///
    /// highest_bet - a u32 for the current highest amount a single player has placed into the pot for the betting round
    ///
    /// max_bet - a u32 for the largest amount a player can bet in the given round, this is designed for all in so nobody can go higher than the lowest amount someone has
    ///
    /// minimum_bet - a u32 for the minimum bet, this is an input instead of self, reducing need for Game State
    ///
    /// Returns:
    /// tuple (String, u32) - This represents the action that was taken and the amount associated with the action
    ///
    /// Examples:
    /// ```
    /// let mut player1 = Player::new("P1");
    /// player1.set_id(1);
    /// player1.set_money(100);
    /// let (action, amount) = prompt_player_action(&mut player1, 100, 500, 10, &mut TestInput::Empty);
    /// ```
    pub fn prompt_player_action(
        player: &mut Player,
        highest_bet: u32,
        max_bet: u32,
        minimum_bet: u32,
        test_input: &mut TestInput,
    ) -> (String, u32) {
        loop {
            let placed_in_pot = match player.player_choices.get("PlacedInPot") {
                Some(PlayerChoice::PlacedInPot(amount)) => *amount,
                _ => 0,
            };
            println!(
                "Amount player placed in pot: {}, Call Amount: {}, Raise Amount: {}",
                placed_in_pot,
                max(highest_bet - placed_in_pot, 0),
                max(highest_bet * 2 - placed_in_pot, minimum_bet)
            );
            println!("What would you like to do, {}?", player.get_name());
            let mut valid_choices = Vec::new();
            // Players can only be given the option to fold if they haven't already
            // This isn't needed for an if statement because we are never asking a player whos folded what they want to do
            if !player.player_choices.contains_key("Fold") {
                valid_choices.push("Fold");
            }
            // Want to make sure that they aren't the player that raised and they have enough to call the raise
            if highest_bet > placed_in_pot
                && ((player.get_money().clone() + placed_in_pot) as i32 - highest_bet as i32) > 0
            {
                valid_choices.push("Call");
            }

            // If the player has checked and we are back to them, we should be in the next stage
            // We want to check only if it comes back to the raised player, typically the big blind
            if !player.player_choices.contains_key("Check") && highest_bet == placed_in_pot {
                valid_choices.push("Check");
            }

            // Raise only when the player can money wise and they are under the max bet money wise
            if (player.get_money().clone() as i32 - (highest_bet * 2) as i32) >= 0
                && max_bet > (player.get_money().clone() - (highest_bet * 2))
            {
                valid_choices.push("Raise");
            } else {
                valid_choices.push("All In");
            }

            // Showing the player their choices that we put into the vector above
            for (index, option) in valid_choices.iter().enumerate() {
                println!("{}. {}", index + 1, option);
            }
            println!("Choose an action (enter the number):");

            let mut input = String::new();
            match test_input {
                TestInput::Input(ref mut input_str) => {
                    input = input_str[0].clone();
                    input_str.remove(0);
                    if input_str.len() == 0 {
                        *test_input = TestInput::Empty;
                    }
                }
                TestInput::Empty => {
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");
                }
            }

            // Parsing the given choice into another function and handling main error here
            // This is the bottleneck for error as this choice affects all logic so preventing it here is key
            match input.trim().parse::<usize>() {
                Ok(choice) => {
                    if choice > 0 && choice <= valid_choices.len() {
                        let action = valid_choices[choice - 1];
                        return Self::do_player_action(
                            action,
                            placed_in_pot,
                            player,
                            highest_bet,
                            test_input,
                        );
                    } else {
                        println!("Invalid choice. Please select a valid action.");
                    }
                }
                Err(_) => {
                    println!("Invalid input. Please enter a number.");
                }
            }
        }
    }

    /// Function to fullfill the desired player action, if the prompt is raise we further prompt the players input
    /// for how much they would like to raise by given their current money and what the next raise is too
    ///
    /// Inputs:
    /// player_action - a &str representing the highest amount that the player can enter
    ///
    /// placed_in_pot - a u32 representing the amount of money the player has placed into the pot for the current betting round
    ///
    /// player - a mutable reference to the player we are doing the action for
    ///
    /// highest_bet - a u32 for the current highest amount a single player has placed into the pot for the betting round
    ///
    /// Returns:
    /// tuple (String, u32) - This represents the action that was taken and the amount associated with the action
    ///
    /// Examples:
    /// ```rust
    /// let mut player1 = Player::new("P1");
    /// player1.set_id(1);
    /// player1.set_money(100);
    /// let (action, amount) = do_player_action("Raise", 100, &mut player1, 20, &mut TestInput::Empty);
    /// let (action, amount) = do_player_action("Fold", 100, &mut player1, 20, &mut TestInput::Empty);
    /// ```
    pub fn do_player_action(
        player_action: &str,
        placed_in_pot: u32,
        player: &mut Player,
        highest_bet: u32,
        test_input: &mut TestInput,
    ) -> (String, u32) {
        match player_action {
            "Fold" => {
                player.fold();
                return (String::from("Fold"), 0 as u32);
            }
            "Call" => {
                let placed_in_pot = match player.get_placedinpot() {
                    Ok(PlayerChoice::PlacedInPot(amount)) => amount,
                    _ => 0,
                };
                let call_amount: u32 = highest_bet - placed_in_pot;
                player.call(call_amount);
                return (String::from("Call"), call_amount);
            }
            "Check" => {
                player.check();
                return (String::from("Check"), 0);
            }
            "Raise" => {
                let upper_limit = (player.get_money().clone()) as i32;
                let lower_limit = (highest_bet * 2 - placed_in_pot) as i32;
                let raise_amount =
                    Self::prompt_bet_amount(upper_limit, lower_limit, test_input) - placed_in_pot;
                if raise_amount == player.get_money().clone() {
                    player.all_in(raise_amount);
                    return (String::from("All In"), raise_amount);
                } else {
                    player.raise(raise_amount);
                    return (String::from("Raise"), raise_amount + placed_in_pot);
                }
            }
            "All In" => {
                let all_in_amount = player.get_money().clone();
                player.all_in(all_in_amount);
                return (String::from("All In"), all_in_amount);
            }
            _ => return ("".to_string(), 0),
        }
    }

    /// Function to prompt the player for how much they would like to bet
    ///
    /// This function is used in do_player_action when asking the player how much they would want to raise by.
    /// The upper and lower limits provide the minimum and maximum that we are wanting from the raise.
    ///
    /// Inputs:
    /// upper_limit - an i32 representing the highest amount that the player can enter
    ///
    /// lower_limit - an i32 representing the lowest amount that the player can enter
    ///
    /// Returns:
    /// Returning a u32 for the amount that the player is wanting to place into the pot
    ///
    /// Examples:
    /// ```rust
    /// let amount = Game_State(1000, 100, &mut TestInput::Empty);
    /// ```
    pub fn prompt_bet_amount(
        upper_limit: i32,
        lower_limit: i32,
        test_input: &mut TestInput,
    ) -> u32 {
        loop {
            print!(
                "Enter your bet amount between {} and {}: ",
                lower_limit, upper_limit
            );

            let mut input = String::new();
            match test_input {
                TestInput::Input(ref mut input_str) => {
                    input = input_str[0].clone();
                    input_str.remove(0);
                    if input_str.len() == 0 {
                        *test_input = TestInput::Empty;
                    }
                }
                TestInput::Empty => {
                    io::stdout().flush().unwrap();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");
                }
            }

            // Try to parse the input as a valid u32 value
            match input.trim().parse::<i32>() {
                Ok(bet_amount) => {
                    // Check if bet is within the valid range
                    if bet_amount >= lower_limit && bet_amount <= upper_limit {
                        return bet_amount as u32;
                    } else {
                        println!(
                            "Bet amount must be between {} and {}. Please try again.",
                            lower_limit, upper_limit
                        );
                    }
                }
                Err(_) => {
                    println!("Invalid input. Please enter a valid bet amount.");
                }
            }
        }
    }

    /// Printing general aspects of the players stats
    ///
    /// Displaying the players hand and how much they have placed in the pot for that betting round.
    /// This number will reset every round, can help the player decide what they want to do more in depth
    /// Displaying as well the amount that is inside of the pot currently
    ///
    /// Inputs:
    /// player - an reference Player that gives us the memory address to the player and its associated data
    /// current_bet - an u32 representing the current bet that the player has placed
    /// pot - an u32 representing the current pot of the game
    ///
    ///
    /// Returns:
    /// Returning a u32 for the amount that the player is wanting to place into the pot
    ///
    /// Examples:
    /// ```rust
    /// let mut test_game = GameState::empty_constructor(1);
    /// let mut player1 = Player::new_player(5 as u8, "P1".to_string(), 1);
    /// let mut player2 = Player::new_player(5 as u8,"P2".to_string(), 2);
    /// let mut player3 = Player::new_player( 5 as u8, "P3".to_string(), 3);
    /// player1.set_money(100 as u32);
    /// player2.set_money(100 as u32);
    /// player3.set_money(100 as u32);
    /// test_game.put_into_pot(player1.get_id().clone(), 20);
    /// let mut local_highest = test_game.get_highest_bet().clone();
    /// Self::print_betting_info(&mut player1, local_highest.clone(), self.pot);
    /// ```
    pub fn print_betting_info(
        player: &Player,
        current_bet: u32,
        pot: u32,
        community_cards: &Option<Hand>,
    ) {
        println!("{}'s turn", player.get_name());

        if community_cards.is_some() {
            println!(
                "Community Cards: {}",
                community_cards.clone().unwrap().clone()
            );
        };

        println!("{}'s hand: {}\n", player.get_name(), player.player_hand);

        let placed_in_pot = match player.get_placedinpot() {
            Ok(PlayerChoice::PlacedInPot(amount)) => amount,
            _ => 0, // Default to 0 if no amount placed in the pot
        };
        println!(
            "Current Max Bet: {}, {}'s current bet: {}, Available Funds: {}, Pot Total: {}",
            current_bet,
            player.get_name(),
            placed_in_pot,
            player.player_money,
            pot
        );
    }
    /// Core function, go through the call, check, raise, fold motions of a round that players can bet in
    ///
    /// In betting loop we want to ask all players what their desired action is, from that action give reactions from other players.
    /// This is best described when raising; upon raising we then need to ask all players that have not folded what their reaction to this action is.
    /// The index is the current player that we are looking at for their action and we want to constantly loop while the
    /// players that are remaining, this is the number of people who have not folded.
    ///
    /// There is three conditions to exit betting loop, the first being all but one player has folded, the second being that on
    /// a raise every other player that has not folded has called, and the last condition is when a player checks.
    ///
    /// All in is a form of raise/call, this is dependant on the players money and whether or not another
    /// player as done an all in call already.
    ///
    ///
    /// ```
    /// let mut game = Game_State::empty_constructor(1);
    /// let mut player1 = Player::new("P1");
    /// let mut player2 = Player::new("P2");
    /// let mut player3 = Player::new("P3");
    /// let mut player4 = Player::new("P4");
    /// player1.set_id(1);
    /// player2.set_id(2);
    /// player3.set_id(3);
    /// player4.set_id(4);
    /// player1.set_money(100);
    /// player2.set_money(100);
    /// player3.set_money(100);
    /// player4.set_money(100);
    /// game.insert_player(&player1).unwrap();
    /// game.insert_player(&player2).unwrap();
    /// game.insert_player(&player3).unwrap();
    /// game.insert_player(&player4).unwrap();
    ///
    /// test_game.game_initalize(1000);
    /// test_game.deal_hands();
    /// test_game.put_blinds();
    ///
    /// game.betting_loop(&mut TestInput::Empty);
    /// ```
    #[allow(unused_assignments)]
    pub async fn betting_loop(
        &mut self,
        mut index: u32,
        test_input: &mut TestInput,
        community_cards: Option<Hand>,
    ) {
        //let mut index = (self.dealer + 3) % (self.players.len() as u32);
        self.round_number += 1;
        let total_players = self.players.len() as u32;
        let mut players_searched = 0;
        let mut players_remaining = 0_u32;
        let mut all_in_amount = 1000000000_u32;
        let mut all_in_call = false;

        for player in &self.players {
            if !player.player_choices.contains_key("Fold") {
                players_remaining += 1;
            }
        }

        let mut local_highest = self.highest_bet;
        loop {
            local_highest = self.get_highest_bet().clone();
            while players_searched <= players_remaining {
                let mut player = &mut self.players[index as usize];
                index = (index + 1) % (total_players.clone());
                // If the player has not folded yet we want to ask them what their action is
                if !player.player_choices.contains_key("Fold") {
                    players_searched += 1;
                    Self::print_betting_info(
                        &mut player,
                        local_highest.clone(),
                        self.pot,
                        &community_cards,
                    );

                    // Prompt the player given the current game state what their decision is, need to pass the result of this
                    // back to here so we can update the referenced player data appropreately
                    let mut decision = ("Skip".to_string(), 0 as u32);
                    if !player.player_choices.contains_key("AllIn") {
                        decision = Self::prompt_player_action(
                            &mut player,
                            self.highest_bet,
                            all_in_amount,
                            self.minimum_bet.clone(),
                            test_input,
                        );
                    }
                    // Interpreting the players desired action and performing required changes to reflect this on the wider game
                    match decision.0.as_str() {
                        "Fold" => {
                            players_remaining -= 1;
                            players_searched -= 1;
                        }
                        "Check" => (),
                        "Raise" => {
                            // We reset players searched because we want to ask the remaining players
                            players_searched = 1;
                            // sHow much extra the player put into the pot
                            self.add_to_pot(decision.1.clone());
                            // make the raise if it was valid the new highest bet
                            self.set_highest_bet(decision.1.clone());
                            local_highest = decision.1.clone();
                        }
                        "Call" => {
                            // Also add the call value to the pot
                            self.add_to_pot(decision.1);
                        }
                        "All In" => {
                            // On an all in we need to firstly perform a similar action to call, then see
                            // if this all in is the first all in then if true also preform a raise action
                            self.add_to_pot(decision.1.clone());
                            if !all_in_call {
                                players_searched = 2;
                                local_highest = self.highest_bet + decision.1.clone();
                                all_in_amount = local_highest;
                                self.set_highest_bet(local_highest);
                                all_in_call = true;
                            }
                        }
                        // This is meant as a way to bypass all actions, impossible to encounter during regular
                        // code execution
                        "Skip" => (),
                        _ => {
                            println!("Invalid action. Please choose again.");
                            continue;
                        }
                    }
                }
                // Performing a check to see if we need to end the betting loop because all other players have folded
                if players_remaining == 1 {
                    return;
                }
                // We have iterated through the remaining possible players and as a result no player has re raised and
                // now we need to go to the next stage of the game
                if players_searched == players_remaining {
                    return;
                }
            }
        }
    }

    /// Preparing associated data with the game state for the new betting round
    ///
    /// For the new betting roudn we need to save only the players choices if they folded otherwise it is now blank
    /// Also setting the highest bet to 0, this is showing that raises now start at the minimum bet for the new round
    ///
    /// ```
    /// let mut game = Game_State::empty_constructor(1);
    /// let mut player1 = Player::new("P1");
    /// let mut player2 = Player::new("P2");
    /// let mut player3 = Player::new("P3");
    /// let mut player4 = Player::new("P4");
    /// player1.set_id(1);
    /// player2.set_id(2);
    /// player3.set_id(3);
    /// player4.set_id(4);
    /// player1.set_money(100);
    /// player2.set_money(100);
    /// player3.set_money(100);
    /// player4.set_money(100);
    /// game.insert_player(&player1).unwrap();
    /// game.insert_player(&player2).unwrap();
    /// game.insert_player(&player3).unwrap();
    /// game.insert_player(&player4).unwrap();
    ///
    /// test_game.game_initalize(1000);
    /// test_game.deal_hands();
    /// test_game.put_blinds();
    ///
    /// game.betting_loop(&mut TestInput::Empty);
    /// game.new_betting_round();
    /// ```
    pub fn new_betting_round(&mut self) {
        for player in &mut self.players {
            player.new_round_choices();
        }
        self.highest_bet = 0;
    }

    /// Issue cards to each of the players
    ///
    /// Given the hand size for the game that is set on construction, we pass the deck to player so the
    /// players hand calls draw card from the deck to put the card that was removed from the deck into their hand.
    /// Do this for all players that are in the player vector.
    ///
    /// ```
    /// let mut game = Game_State::empty_constructor(1);
    /// let mut player1 = Player::new("P1");
    /// let mut player2 = Player::new("P2");
    /// let mut player3 = Player::new("P3");
    /// let mut player4 = Player::new("P4");
    /// player1.set_id(1);
    /// player2.set_id(2);
    /// player3.set_id(3);
    /// player4.set_id(4);
    /// player1.set_money(100);
    /// player2.set_money(100);
    /// player3.set_money(100);
    /// player4.set_money(100);
    /// game.insert_player(&player1).unwrap();
    /// game.insert_player(&player2).unwrap();
    /// game.insert_player(&player3).unwrap();
    /// game.insert_player(&player4).unwrap();
    ///
    /// test_game.game_initalize(1000);
    /// test_game.deal_hands();
    /// ```
    pub fn deal_hands(&mut self) {
        for player in &mut self.players {
            for _ in 0..self.hand_size {
                let _ = player.draw_card(&mut self.deck);
            }
        }
    }

    /// Automatically take out money from the blinds and put it into the pot
    ///
    /// From the dealer position we find the small and big blind, then take out the assocaited money from each of the players.
    /// This money is then put into the pot and the highest bet is set to the big blind. This ensures that when the betting loop
    /// is called players have to match the value that is put in or raise on top of it.
    ///
    /// ```
    /// let mut game = Game_State::empty_constructor(1);
    /// let mut player1 = Player::new("P1");
    /// let mut player2 = Player::new("P2");
    /// let mut player3 = Player::new("P3");
    /// let mut player4 = Player::new("P4");
    /// player1.set_id(1);
    /// player2.set_id(2);
    /// player3.set_id(3);
    /// player4.set_id(4);
    /// player1.set_money(100);
    /// player2.set_money(100);
    /// player3.set_money(100);
    /// player4.set_money(100);
    /// game.insert_player(&player1).unwrap();
    /// game.insert_player(&player2).unwrap();
    /// game.insert_player(&player3).unwrap();
    /// game.insert_player(&player4).unwrap();
    ///
    /// test_game.game_initalize(1000);
    /// test_game.put_blinds();
    // ```
    pub fn put_blinds(&mut self) {
        // println!("Small");
        let index_small = ((self.dealer + 1) % self.players.len() as u32) as usize;
        let pid_small = self.players[index_small].get_id().clone();
        self.put_into_pot(pid_small, self.minimum_bet / 2);
        self.players[index_small].bet(self.minimum_bet / 2);
        // self.game_state.players[self.game_state.blinds[2] as usize].add_to_placed_in_pot(self.game_state.minimum_bet/2);
        // Big blind second
        // println!("big");
        let index_big = ((self.dealer + 2) % self.players.len() as u32) as usize;
        let pid_big = self.players[index_big].get_id().clone();
        self.put_into_pot(pid_big, self.minimum_bet);
        self.players[index_big].bet(self.minimum_bet);
        // self.game_state.players[self.game_state.blinds[2] as usize].add_to_placed_in_pot(self.game_state.minimum_bet);
        // Fix this later for error catching
        self.set_highest_bet(self.minimum_bet);
    }

    pub fn get_remaining_player_count(&self) -> u32 {
        let mut count = 0;
        for player in self.players.clone() {
            if !player.player_choices.contains_key("Fold") {
                count += 1;
            }
        }
        count
    }

    pub fn post_round_menu(&self) -> bool {
        loop {
            println!("Post Game Options (enter a number):");
            println!("    1. Play another round with the same players");
            println!("    2. Add a player");
            println!("    3. Remove a player");
            println!("    4. Close the Lobby and return to Main menu");

            io::stdout().flush().expect("Failed to flush output");

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            match input.trim().to_lowercase().as_str() {
                "1" => return true,
                "2" => println!("TODO!"), // TODO
                "3" => println!("TODO!"), //TODO
                "4" => return false,
                _ => println!("Invalid input. Please enter valid input (1-4)"),
            }
        }
    }

    pub async fn write_results_to_db(&self) {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();
        let _result = db_client.insert(self).await;

        for player in self.players.clone() {
            let _ = db_client.update_one(&player).await;
        }
    }
}

/// Implementation of the DbEntity trait for the GameState struct, enabling conversion to and from BSON documents
/// for storage in a MongoDB database.
///
/// This trait defines methods for transforming a `GameState` struct to and from a BSON `Document`, as well as
/// specifying a unique field for identifying each `GameState` in the database.
impl DbEntity for GameState {
    /// Returns the name of the MongoDB collection where the `GameState` entity will be stored.
    ///
    /// Outputs:
    /// - A static string `"games"` representing the collection name.
    ///
    /// Examples:
    /// ```rust
    /// let collection_name = GameState::collection_name();
    /// assert_eq!(collection_name, "games");
    /// ```
    fn collection_name() -> &'static str {
        "games"
    }

    /// Converts the `GameState` struct into a BSON document that can be inserted into the MongoDB database.
    ///
    /// This method converts each field of the `GameState` struct into a BSON format using the `bson::to_bson`
    /// method, handles errors that occur during the conversion, and returns a BSON `Document`.
    ///
    /// Inputs:
    /// - `self` - the current instance of `GameState` to be converted into a BSON document.
    ///
    /// Outputs:
    /// - A `Result<Document, String>`, where `Document` is the BSON representation of the `GameState` and
    ///   any errors are converted into a string.
    ///
    /// Examples:
    /// ```rust
    /// let game_state = GameState { /* fields */ };
    /// let doc = game_state.to_document().unwrap();
    /// ```
    fn to_document(&self) -> Result<Document, String> {
        Ok(doc! {
            "players": bson::to_bson(&self.players).map_err(|e| e.to_string())?,
            "deck": bson::to_bson(&self.deck).map_err(|e| e.to_string())?,
            "game_id": self.game_id as i64,
            "game_variant": self.game_variant.clone(),
            "max_players": self.max_players as i64,
            "hand_size": self.hand_size as i64,
            "pot": self.pot as i64,
            "stage_number": self.stage_number as i64,
            "round_number": self.round_number as i64,
            "minimum_bet": self.minimum_bet as i64,
            "highest_bet": self.highest_bet as i64,
            "dealer": self.dealer as i64,
            "winner": bson::to_bson(&self.winner).map_err(|e| e.to_string())?,
        })
    }

    /// Converts a BSON document from the database into a `GameState` struct.
    ///
    /// This method extracts fields from the provided BSON `Document`, converts them to their appropriate
    /// Rust types, and returns a new `GameState` instance. Errors during extraction are returned as strings.
    ///
    /// Inputs:
    /// - `doc` - A BSON `Document` representing a `GameState` to be deserialized.
    ///
    /// Outputs:
    /// - A `Result<GameState, String>`, where `GameState` is the deserialized struct and any errors are
    ///   returned as a string.
    ///
    /// Examples:
    /// ```rust
    /// let doc = /* BSON document from MongoDB */;
    /// let game_state = GameState::from_document(&doc).unwrap();
    /// ```
    fn from_document(doc: &Document) -> Result<Self, String> {
        Ok(GameState {
            players: bson::from_bson(doc.get("players").cloned().unwrap_or(bson::Bson::Null))
                .map_err(|e| e.to_string())?,
            deck: bson::from_bson(doc.get("deck").cloned().unwrap_or(bson::Bson::Null))
                .map_err(|e| e.to_string())?,
            game_id: doc.get_i64("game_id").map_err(|e| e.to_string())? as u32,
            max_players: doc.get_i64("max_players").map_err(|e| e.to_string())?,
            game_variant: doc
                .get_str("game_variant")
                .map_err(|e| e.to_string())?
                .to_string(),
            hand_size: doc.get_i64("hand_size").map_err(|e| e.to_string())? as u8,
            pot: doc.get_i64("pot").map_err(|e| e.to_string())? as u32,
            stage_number: doc.get_i64("stage_number").map_err(|e| e.to_string())? as u32,
            round_number: doc.get_i64("round_number").map_err(|e| e.to_string())? as u32,
            minimum_bet: doc.get_i64("minimum_bet").map_err(|e| e.to_string())? as u32,
            highest_bet: doc.get_i64("highest_bet").map_err(|e| e.to_string())? as u32,
            dealer: doc.get_i64("dealer").map_err(|e| e.to_string())? as u32,
            winner: bson::from_bson(doc.get("winner").cloned().unwrap_or(bson::Bson::Null))
                .map_err(|e| e.to_string())?,
        })
    }

    /// Returns a BSON document that uniquely identifies the `GameState` in the database.
    ///
    /// This method creates a BSON document with a unique identifier for the `GameState`, using the `game_id`
    /// field as the key.
    ///
    /// Inputs:
    /// - `self` - the current instance of `GameState`.
    ///
    /// Outputs:
    /// - A BSON `Document` containing a unique key-value pair for identifying the `GameState`.
    ///
    /// Examples:
    /// ```rust
    /// let game_state = GameState { /* fields */ };
    /// let unique_field = game_state.unique_field();
    /// assert_eq!(unique_field, doc! { "game_id": 1 });
    /// ```
    fn unique_field(&self) -> Document {
        doc! { "game_id": self.game_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = GameState::new_game(1);
        assert_eq!(game.game_id, 1);
        assert_eq!(game.players.len(), 0);
        assert_eq!(game.pot, 0);
    }

    #[test]
    fn test_set_highest_bet() {
        let mut game = GameState::new_game(1);
        assert!(game.set_highest_bet(100));
        assert_eq!(*game.get_highest_bet(), 100);
        assert!(!game.set_highest_bet(50)); // Should not update if lower
    }

    #[test]
    fn test_insert_player() {
        let mut game = GameState::new_game(1);
        let player = Player::new_player(5 as u8, "Alice".to_string(), 1);
        assert!(game.insert_player(&player).is_ok());
        assert_eq!(game.players.len(), 1);
    }

    #[test]
    fn test_remove_player() {
        let mut game = GameState::new_game(1);
        let mut player = Player::new_player(5 as u8, "Bob".to_string(), 1);
        let player_id = 1;
        player.set_id(player_id);
        game.insert_player(&player).unwrap();
        assert_eq!(game.players.len(), 1);
        game.remove_player(player_id);
        assert_eq!(game.players.len(), 0);
    }

    #[test]
    fn test_put_into_pot() {
        let mut game = GameState::new_game(1);
        let mut player = Player::new_player(5 as u8, "Charlie".to_string(), 1);
        player.set_money(1000);
        game.insert_player(&player).unwrap();
        game.put_into_pot(1, 100);
        assert_eq!(game.pot, 100);
        assert_eq!(*game.players[0].get_money(), 900);
    }

    #[test]
    fn test_give_pot() {
        let mut game = GameState::new_game(1);
        let mut player = Player::new_player(5 as u8, "David".to_string(), 1);
        player.set_money(1000 as u32);
        game.insert_player(&player).unwrap();
        game.pot = 500;
        game.give_pot(1);
        assert_eq!(*game.players[0].get_money(), 1500);
        assert_eq!(game.pot, 0);
    }

    #[test]
    fn test_split_pot() {
        let mut game = GameState::new_game(1);
        let mut player1 = Player::new_player(5 as u8, "Eve".to_string(), 1);
        let mut player2 = Player::new_player(5 as u8, "Frank".to_string(), 2);
        player1.set_money(1000 as u32);
        player2.set_money(1000 as u32);
        game.insert_player(&player1).unwrap();
        game.insert_player(&player2).unwrap();
        game.pot = 500;
        game.split_pot(&vec![1, 2]);
        assert_eq!(*game.players[0].get_money(), 1250);
        assert_eq!(*game.players[1].get_money(), 1250);
    }

    #[test]
    fn test_rotate_blinds() {
        let mut game = GameState::new_game(1);
        let player = Player::new_player(5 as u8, "Grace".to_string(), 1);
        game.insert_player(&player).unwrap();
        game.rotate_blinds();
        assert_eq!(game.dealer, 0);
    }

    #[test]
    fn test_draw_card_from_deck() {
        let mut game = GameState::new_game(1);
        let result = game.draw_card_from_deck();
        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_player_action_fold() {
        let mut player = Player::new_player(5 as u8, "P1".to_string(), 1);

        player.set_name("TestPlayer".to_string());
        player.set_money(1000);
        let highest_bet = 50;
        let max_bet = 100;
        let minimum_bet = 10;

        // Simulate multiple user inputs in a sequence
        let mut simulated_input = TestInput::Input(vec!["1".to_string(), "3".to_string()]);

        let (action, amount) = GameState::prompt_player_action(
            &mut player,
            highest_bet,
            max_bet,
            minimum_bet,
            &mut simulated_input,
        );

        assert_eq!(action, "Fold");
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_prompt_player_action_call() {
        let mut player = Player::new_player(5 as u8, "P1".to_string(), 1);

        player.set_name("TestPlayer".to_string());
        player.add_to_placed_in_pot(20);
        player.set_money(1000);
        let highest_bet = 50;
        let max_bet = 100;
        let minimum_bet = 10;

        // Simulate multiple user inputs in a sequence
        let mut simulated_input = TestInput::Input(vec!["2".to_string()]);

        let (action, amount) = GameState::prompt_player_action(
            &mut player,
            highest_bet,
            max_bet,
            minimum_bet,
            &mut simulated_input,
        );
        assert_eq!(action, "Call");
        assert_eq!(amount, 30);
    }

    #[test]
    fn test_prompt_player_action_check() {
        let mut player = Player::new_player(5 as u8, "P1".to_string(), 1);

        player.set_name("TestPlayer".to_string());
        player.add_to_placed_in_pot(20);
        player.set_money(1000);
        let highest_bet = 20;
        let max_bet = 100;
        let minimum_bet = 10;

        // Simulate multiple user inputs in a sequence
        let mut simulated_input = TestInput::Input(vec!["2".to_string()]);

        let (action, amount) = GameState::prompt_player_action(
            &mut player,
            highest_bet,
            max_bet,
            minimum_bet,
            &mut simulated_input,
        );
        assert_eq!(action, "Check");
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_prompt_player_action_raise() {
        let mut player = Player::new_player(5 as u8, "P1".to_string(), 1);

        player.set_name("TestPlayer".to_string());
        player.add_to_placed_in_pot(20);
        player.set_money(1000);
        let highest_bet = 50;
        let max_bet = 4000;
        let minimum_bet = 10;

        // Simulate multiple user inputs in a sequence
        let mut simulated_input = TestInput::Input(vec!["3".to_string(), "100".to_string()]);

        let (action, amount) = GameState::prompt_player_action(
            &mut player,
            highest_bet,
            max_bet,
            minimum_bet,
            &mut simulated_input,
        );
        assert_eq!(action, "Raise");
        assert_eq!(amount, 100);
    }

    #[test]
    fn test_prompt_player_action_all_in() {
        let mut player = Player::new_player(5 as u8, "P1".to_string(), 1);
        player.set_name("TestPlayer".to_string());
        player.add_to_placed_in_pot(20);
        player.set_money(1000);
        let highest_bet = 80;
        let max_bet = 100;
        let minimum_bet = 10;

        // Simulate multiple user inputs in a sequence
        let mut simulated_input = TestInput::Input(vec!["3".to_string()]);

        let (action, amount) = GameState::prompt_player_action(
            &mut player,
            highest_bet,
            max_bet,
            minimum_bet,
            &mut simulated_input,
        );
        assert_eq!(action, "All In");
        assert_eq!(amount, 1000);
    }

    #[test]
    fn test_prompt_bet_amount() {
        let upper_limit = 200;
        let lower_limit = 100;

        // Simulate multiple user inputs in a sequence
        let mut simulated_input = TestInput::Input(vec!["5".to_string(), "150".to_string()]);

        let amount = GameState::prompt_bet_amount(upper_limit, lower_limit, &mut simulated_input);
        assert_eq!(amount, 150);
    }

    #[test]
    fn test_round_and_game_initalize() {
        let mut test_game = GameState::empty_constructor(1);
        let mut player1 = Player::new_player(5 as u8, "P1".to_string(), 1);
        let mut player2 = Player::new_player(5 as u8, "P2".to_string(), 2);
        let mut player3 = Player::new_player(5 as u8, "P3".to_string(), 3);
        let mut player4 = Player::new_player(5 as u8, "P4".to_string(), 4);

        player1.set_money(1000);
        player2.set_money(1000);
        player3.set_money(1000);
        player4.set_money(1000);

        let _ = test_game.insert_player(&player1);
        let _ = test_game.insert_player(&player2);
        let _ = test_game.insert_player(&player3);
        let _ = test_game.insert_player(&player4);
        for player in &mut test_game.players {
            player.call(10);
        }

        test_game.game_initalize(1000);
        assert_eq!(test_game.stage_number, 1);
        // for player in test_game.players {
        //     assert_eq!(*player.get_money(), 1000);
        // }
    }

    #[test]
    fn test_put_blinds() {
        let mut game = GameState::new_game(1);
        let mut player1 = Player::new_player(5 as u8, "P1".to_string(), 1);
        let mut player2 = Player::new_player(5 as u8, "P2".to_string(), 2);
        let mut player3 = Player::new_player(5 as u8, "P3".to_string(), 3);

        player1.set_money(1000);
        player2.set_money(1000);
        player3.set_money(1000);

        game.insert_player(&player1).unwrap();
        game.insert_player(&player2).unwrap();
        game.insert_player(&player3).unwrap();

        game.put_blinds();

        assert_eq!(game.pot, game.minimum_bet + (game.minimum_bet / 2));
        assert_eq!(*game.players[1].get_money(), 1000 - (game.minimum_bet / 2));
        assert_eq!(*game.players[2].get_money(), 1000 - game.minimum_bet);
    }

    #[tokio::test]
    async fn test_betting_loop_fold() {
        let mut test_game = GameState::empty_constructor(1);
        let mut player1 = Player::new_player(5 as u8, "P1".to_string(), 1);
        let mut player2 = Player::new_player(5 as u8, "P2".to_string(), 2);
        let mut player3 = Player::new_player(5 as u8, "P3".to_string(), 3);
        let mut player4 = Player::new_player(5 as u8, "P5".to_string(), 4);

        player1.set_money(100);
        player2.set_money(100);
        player3.set_money(100);
        player4.set_money(100);

        test_game.insert_player(&player1).unwrap();
        test_game.insert_player(&player2).unwrap();
        test_game.insert_player(&player3).unwrap();
        test_game.insert_player(&player4).unwrap();

        // Simulate multiple user inputs in a sequence
        let mut simulated_input = TestInput::Input(vec![
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
        ]);
        test_game.game_initalize(1000);
        test_game.deal_hands();
        test_game.put_blinds();
        test_game.betting_loop(&mut simulated_input).await;
        assert_eq!(
            test_game.players[0].player_choices.contains_key("Fold"),
            true
        );
        assert_eq!(
            test_game.players[1].player_choices.contains_key("Fold"),
            true
        );
        assert_eq!(
            test_game.players[2].player_choices.contains_key("Fold"),
            true
        );
    }
    #[tokio::test]
    async fn test_betting_loop_raise() {
        let mut test_game = GameState::empty_constructor(1);
        let mut player1 = Player::new_player(5 as u8, "P1".to_string(), 1);
        let mut player2 = Player::new_player(5 as u8, "P2".to_string(), 2);
        let mut player3 = Player::new_player(5 as u8, "P3".to_string(), 3);
        let mut player4 = Player::new_player(5 as u8, "P5".to_string(), 4);

        player1.set_money(100);
        player2.set_money(100);
        player3.set_money(100);
        player4.set_money(100);

        test_game.insert_player(&player1).unwrap();
        test_game.insert_player(&player2).unwrap();
        test_game.insert_player(&player3).unwrap();
        test_game.insert_player(&player4).unwrap();

        // Simulate multiple user inputs in a sequence
        let mut simulated_input = TestInput::Input(vec![
            "3".to_string(),
            "20".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
        ]);
        test_game.game_initalize(1000);
        test_game.deal_hands();
        test_game.put_blinds();
        test_game.betting_loop(&mut simulated_input).await;
        for player in &test_game.players {
            player.print_enums();
        }
        assert_eq!(
            test_game.players[0].player_choices.contains_key("Raise"),
            true
        );
        assert_eq!(
            test_game.players[1].player_choices.contains_key("Call"),
            true
        );
        assert_eq!(
            test_game.players[2].player_choices.contains_key("Call"),
            true
        );
        assert_eq!(
            test_game.players[3].player_choices.contains_key("Call"),
            true
        );
    }
}
