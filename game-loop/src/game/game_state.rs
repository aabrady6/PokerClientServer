//! Core game rules that are common amongst poker variants and poker game values
//!
//! Creates calls to structs that are inside of GameState and helps perform passing
//! of references and data between these structs.
//! GameState meant to be by in large asynchronous with updates being made to the database on game completion
//!

use crate::db::dbclient::{DbClient, DbEntity, MONGO_URI};
use crate::game::card::Card;
use crate::game::card::Suit;
use crate::game::card::Value;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::player::Player;
use crate::game::player::PlayerChoice;
use crate::game::score_hands::get_best_5_card_hand;
use crate::game::score_hands::get_best_from_7_card_hand;
use crate::game::score_hands::ScoredHand;
use mongodb::bson;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};

/// A struct that represents the state of the game, containing essential information about the game setup,
/// players, and current game status.
#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    pub game_variant: String,
    pub deck: Deck,
    pub game_id: u32,
    pub hand_size: u8,
    pub max_players: i64,
    pub demo_mode: String,
    pub round_number: u32,
    pub minimum_bet: u32,
    pub highest_bet: u32,
    pub current_player: Player,
    pub player_action: String,
    pub players: Vec<Player>,
    pub lobby: Vec<Player>,
    pub spectators: Vec<Player>,
    pub dealer_choice_spectators: Vec<Player>,
    pub winner: Vec<(Player, ScoredHand)>,
    pub community_cards: Hand,
    pub pot: u32,
    pub dealer: u32,
    pub discard_cards_prompted: bool,
    pub current_action_string: String,
    pub action_history: Vec<String>,
    pub raise_min_max: (u32, u32),
}

/// Implements `Clone` for `GameState`.
impl Clone for GameState {
    fn clone(&self) -> Self {
        Self {
            game_variant: self.game_variant.clone(),
            deck: self.deck.clone(),
            game_id: self.game_id,
            hand_size: self.hand_size,
            max_players: self.max_players,
            demo_mode: self.demo_mode.clone(),
            round_number: self.round_number,
            minimum_bet: self.minimum_bet,
            highest_bet: self.highest_bet,
            current_player: self.current_player.clone(),
            player_action: self.player_action.clone(),
            players: self.players.clone(),
            lobby: self.lobby.clone(),
            spectators: self.spectators.clone(),
            dealer_choice_spectators: self.dealer_choice_spectators.clone(),
            winner: self.winner.clone(),
            community_cards: self.community_cards.clone(),
            pot: self.pot,
            dealer: self.dealer,
            discard_cards_prompted: self.discard_cards_prompted,
            current_action_string: self.current_action_string.clone(),
            action_history: self.action_history.clone(),
            raise_min_max: self.raise_min_max,
        }
    }
}

/// Implements `Default` for `GameState`.
impl Default for GameState {
    fn default() -> Self {
        GameState {
            players: Vec::new(),
            lobby: Vec::new(),
            spectators: Vec::new(),
            dealer_choice_spectators: Vec::new(),
            deck: Deck::new(),
            game_variant: "".to_string(),
            game_id: 0,
            hand_size: 5,
            max_players: 5,
            pot: 0,
            demo_mode: "inactive".to_string(),
            round_number: 0,
            minimum_bet: 5,
            highest_bet: 0,
            dealer: 0,
            winner: vec![],
            current_player: Player::empty(),
            player_action: String::new(),
            community_cards: Hand::new(5),
            discard_cards_prompted: false,
            current_action_string: "".to_string(),
            action_history: Vec::new(),
            raise_min_max: (0, 0),
        }
    }
}

/// Implements functions for `GameState`.
impl GameState {
    /// Creates a new `GameState` instance using default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a dummy `GameState` instance with the given game ID.
    ///
    /// # Arguments
    /// * `id` - The unique identifier for the game.
    pub fn new_dummy_game_state(id: u32) -> Self {
        GameState {
            players: Vec::new(),
            lobby: Vec::new(),
            spectators: Vec::new(),
            dealer_choice_spectators: Vec::new(),
            deck: Deck::new(),
            game_variant: "".to_string(),
            game_id: id,
            hand_size: 5,
            max_players: 5,
            pot: 0,
            round_number: 0,
            minimum_bet: 5,
            highest_bet: 0,
            dealer: 0,
            winner: vec![],
            current_player: Player::empty(),
            player_action: String::new(),
            community_cards: Hand::new(5),
            discard_cards_prompted: false,
            current_action_string: "".to_string(),
            action_history: Vec::new(),
            raise_min_max: (0, 0),
            demo_mode: "inactive".to_string(),
        }
    }

    /// Retains only the lobby-related information in the game state.
    pub fn game_state_retain_lobby_info(&mut self) {
        let mut game_state = GameState::new();
        game_state.players = self.players.clone();
        game_state.lobby = self.lobby.clone();
        game_state.spectators = self.spectators.clone();
        game_state.winner = self.winner.clone();
        game_state.dealer = self.dealer;
        *self = game_state;
    }

    //****************************************************************
    //GAME INITIALIZATION FUNCTIONS
    //****************************************************************

    /// Starts a new 5 Card Draw round, initializing the game state and dealing cards to the players.
    ///
    /// # Details
    /// - This method initializes a new 5 Card Draw game, including setting the game variant to `"5 Card Draw"`,
    ///   setting game-related attributes such as hand size, maximum number of players, pot, and others.
    /// - Each player in the game has their hand reset and is assigned an infinite amount of money for the round.
    /// - Cards are dealt face down to all players, and the blinds are set for the game.
    ///
    /// # Parameters
    /// This function does not take any parameters.
    ///
    /// # Actions
    /// - Resets the game state for a new 5 Card Draw round.
    /// - Deals 5 face-down cards to each player.
    /// - Initializes the blind system for the round.
    pub async fn new_five_card_round(&mut self) {
        let uri = &MONGO_URI;
        let db_client = DbClient::new(uri).await.unwrap();
        let game_id = db_client.get_new_game_id().await;

        self.deck = Deck::new();
        self.game_variant = "5 Card Draw".to_string();
        self.game_id = game_id;
        self.hand_size = 5;
        self.max_players = 7;
        self.pot = 0;
        self.minimum_bet = 5;
        self.demo_mode = "inactive".to_string();
        self.round_number = 0;
        self.highest_bet = 0;
        self.winner = vec![];
        self.current_player = Player::empty();
        self.player_action = String::new();
        self.community_cards = Hand::new(5);
        self.discard_cards_prompted = false;
        self.current_action_string = "".to_string();
        self.action_history = Vec::new();
        self.raise_min_max = (0, 0);

        for player in &mut self.players {
            player.reset_choices();
            player.round_win = 0;
            player.player_hand = Hand::new(5);
            player.infinite_money();
        }

        self.deal_face_down(5).await;
        self.put_blinds().await;
        self.set_player_tokens_blinds();
        println!("Starting new 5 Card Draw.");
    }

    /// Starts a new 7 Card Stud round, initializing the game state and dealing cards to the players.
    ///
    /// # Details
    /// - This method initializes a new 7 Card Stud game, including setting the game variant to `"7 Card Stud"`,
    ///   setting game-related attributes such as hand size, maximum number of players, pot, and others.
    /// - Each player in the game has their hand reset and is assigned an infinite amount of money for the round.
    /// - The round starts by dealing 2 face-down cards to each player, followed by 1 face-up card.
    /// - The dealer is set up with special handling for token distribution.
    ///
    /// # Parameters
    /// This function does not take any parameters.
    ///
    /// # Actions
    /// - Resets the game state for a new 7 Card Stud round.
    /// - Deals 2 face-down cards and 1 face-up card to each player.
    /// - Initializes the token system for the dealer and players.
    pub async fn new_seven_card_round(&mut self) {
        let uri = &MONGO_URI;
        let db_client = DbClient::new(uri).await.unwrap();
        let game_id = db_client.get_new_game_id().await;

        self.deck = Deck::new();
        self.game_variant = "7 Card Stud".to_string();
        self.game_id = game_id;
        self.hand_size = 7;
        self.max_players = 7;
        self.pot = 0;
        self.minimum_bet = 5;
        self.demo_mode = "inactive".to_string();
        self.round_number = 0;
        self.highest_bet = 0;
        self.winner = vec![];
        self.current_player = Player::empty();
        self.player_action = String::new();
        self.community_cards = Hand::new(5);
        self.discard_cards_prompted = false;
        self.current_action_string = "".to_string();
        self.action_history = Vec::new();
        self.raise_min_max = (0, 0);

        for player in &mut self.players {
            player.reset_choices();
            player.round_win = 0;
            player.player_hand = Hand::new(7);
            player.infinite_money();
        }

        self.deal_face_down(2).await;
        self.deal_face_up(1).await;
        self.set_player_tokens_only_dealer();
        println!("Starting new Seven Card Stud Game.");
    }

    /// Starts a new Texas Hold'em round, initializing the game state and dealing cards to the players.
    ///
    /// # Details
    /// - This method initializes a new Texas Hold'em game, including setting the game variant to `"Texas Hold'em"`,
    ///   setting game-related attributes such as hand size, maximum number of players, pot, and others.
    /// - Each player in the game has their hand reset and is assigned an infinite amount of money for the round.
    /// - The round starts by dealing 2 face-down cards to each player.
    /// - The blinds system is set up for the round.
    ///
    /// # Parameters
    /// This function does not take any parameters.
    ///
    /// # Actions
    /// - Resets the game state for a new Texas Hold'em round.
    /// - Deals 2 face-down cards to each player.
    /// - Initializes the blind system for the round.
    pub async fn new_texas_holdem_round(&mut self) {
        let uri = &MONGO_URI;
        let db_client = DbClient::new(uri).await.unwrap();
        let game_id = db_client.get_new_game_id().await;

        self.deck = Deck::new();
        self.game_variant = "Texas Hold'em".to_string();
        self.game_id = game_id;
        self.hand_size = 2;
        self.max_players = 7;
        self.pot = 0;
        self.minimum_bet = 5;
        self.demo_mode = "inactive".to_string();
        self.round_number = 0;
        self.highest_bet = 0;
        self.winner = vec![];
        self.current_player = Player::empty();
        self.player_action = String::new();
        self.community_cards = Hand::new(5);
        self.discard_cards_prompted = false;
        self.current_action_string = "".to_string();
        self.action_history = Vec::new();
        self.raise_min_max = (0, 0);

        for player in &mut self.players {
            player.reset_choices();
            player.round_win = 0;
            player.player_hand = Hand::new(2);
            player.infinite_money();
        }

        self.deal_face_down(2).await;
        self.put_blinds().await;
        self.set_player_tokens_blinds();

        println!("Starting new Texas Hold Em Game.");
    }

    //****************************************************************
    // PLAYER FUNCTIONS
    //****************************************************************

    /// Inserts a player into the main list of players in the game.
    ///
    /// # Parameters
    /// - `player`: A reference to the `Player` object to be added.
    ///
    /// # Returns
    /// - `Ok(())`: If the player is successfully added to the list of players.
    /// - `Err(String)`: If the maximum number of players has already been reached, preventing the insertion.
    ///
    /// # Description
    /// This method attempts to add a player to the main `players` list. If the number of players in the game
    /// has reached the maximum limit (`max_players`), it returns an error. If the player is successfully added,
    /// it returns `Ok(())`.
    pub fn insert_player(&mut self, player: &Player) -> Result<(), String> {
        if self.players.len() >= self.max_players as usize {
            return Err("Unable to push".to_string());
        }
        self.players.push(player.clone());
        Ok(())
    }

    /// Inserts a player into the lobby list.
    ///
    /// # Parameters
    /// - `player`: A reference to the `Player` object to be added to the lobby.
    ///
    /// # Returns
    /// - `Ok(())`: If the player is successfully added to the lobby list.
    /// - `Err(String)`: This method does not currently return any error, as adding to the lobby is always successful.
    ///
    /// # Description
    /// This method adds a player to the lobby, a list of players waiting for the game to start or for matchmaking.
    pub fn insert_player_to_lobby(&mut self, player: &Player) -> Result<(), String> {
        self.lobby.push(player.clone());
        Ok(())
    }

    /// Inserts a player into the list of spectators for the current game.
    ///
    /// # Parameters
    /// - `player`: A reference to the `Player` object to be added to the spectators list.
    ///
    /// # Returns
    /// - `Ok(())`: If the player is successfully added to the spectators list.
    /// - `Err(String)`: This method does not currently return any error, as adding to the spectators is always successful.
    ///
    /// # Description
    /// This method adds a player to the list of spectators who can watch the game but are not participating.
    pub fn insert_player_to_spectators(&mut self, player: &Player) -> Result<(), String> {
        self.spectators.push(player.clone());
        Ok(())
    }

    /// Inserts a player into the list of spectators who have chosen the dealer for the game.
    ///
    /// # Parameters
    /// - `player`: A reference to the `Player` object to be added to the dealer choice spectators list.
    ///
    /// # Returns
    /// - `Ok(())`: If the player is successfully added to the dealer choice spectators list.
    /// - `Err(String)`: This method does not currently return any error, as adding to the dealer choice spectators is always successful.
    ///
    /// # Description
    /// This method adds a player to the list of spectators who are interested in the dealer's choice but are not
    /// actively participating in the game itself.
    pub fn insert_player_to_dealer_choice_spectators(
        &mut self,
        player: &Player,
    ) -> Result<(), String> {
        self.dealer_choice_spectators.push(player.clone());
        Ok(())
    }

    //****************************************************************
    // DEALERS, BLINDS AND STARTING POSITIONS FUNCTIONS
    //****************************************************************

    /// Rotates the dealer to the next player in the list.
    ///
    /// This method moves the dealer chip to the next player by removing the dealer token from the current dealer
    /// and assigning the dealer token to the next player in the list. The dealer's index is updated in a cyclic
    /// manner, wrapping around to the beginning of the list when necessary.
    pub async fn rotate_dealer(&mut self) {
        println!("MOVING DEALER CHIP");
        self.players[self.dealer as usize].token = "".to_string();
        self.dealer = (self.dealer + 1) % self.players.len() as u32;
        // self.players[self.dealer as usize].token = "D".to_string();
    }

    /// Returns the index of the player who should start the blinds phase of the game.
    ///
    /// The starting player is determined to be the one who is three players to the right of the dealer.
    ///
    /// # Returns
    /// - `usize`: The index of the player who will start the blinds phase.
    pub async fn get_blinds_starting_player_index(&self) -> usize {
        ((self.dealer + 3) % self.players.len() as u32) as usize
    }

    /// Returns the index of the player directly left of the dealer.
    ///
    /// This player is typically the one who acts first after the dealer's actions.
    ///
    /// # Returns
    /// - `usize`: The index of the player to the left of the dealer.
    pub async fn get_starting_player_left_of_dealer(&self) -> usize {
        ((self.dealer + 1) % self.players.len() as u32) as usize
    }

    /// Determines the player with the lowest hand value among those who have not folded.
    ///
    /// This method compares the smallest face-up card of each player's hand (if they haven't folded) and returns the
    /// index of the player with the lowest value. This player is considered to start the first betting round.
    ///
    /// # Returns
    /// - `usize`: The index of the player with the lowest hand.
    pub async fn determine_lowest_door(&self) -> usize {
        let mut lowest = self.players[0].clone();
        let mut lowest_index = 0;

        for (index, player) in self.players.iter().enumerate() {
            if !player.player_choices.contains_key("Fold") {
                let mut face_up = player.player_hand.get_face_up_cards();
                let player_value = face_up.get_smallest_value();

                let mut lowest_face_up = lowest.player_hand.get_face_up_cards();
                let lowest_value = lowest_face_up.get_smallest_value();

                if player_value < lowest_value {
                    lowest = player.clone();
                    lowest_index = index;
                }
            }
        }
        println!(
            "Player: {:?} has the lowest hand. They start the first betting round",
            self.players[lowest_index].player_name
        );
        lowest_index
    }

    /// Determines the player with the highest face-up card value among those who have not folded.
    ///
    /// This method evaluates each player's best 5-card hand (based on face-up cards) and returns the index of the player
    /// with the highest value. This player starts the betting round.
    ///
    /// # Returns
    /// - `usize`: The index of the player with the highest hand.
    pub async fn determine_highest_door(&self) -> usize {
        let mut highest_index = 0;
        let empty_vec: Vec<Card> = vec![Card::new(Value::Empty, Suit::Empty).unwrap(); 5];
        let mut empty_hand = Hand::from_cards(empty_vec);
        let mut highest_value: ScoredHand = ScoredHand::HighCard([Value::Empty; 5]);

        for (index, player) in self.players.iter().enumerate() {
            if !player.player_choices.contains_key("Fold") {
                let face_up = player.player_hand.get_face_up_cards();
                for (i, card) in face_up.cards.iter().enumerate() {
                    if i < empty_hand.cards.len() {
                        empty_hand.cards[i] = *card;
                    }
                }
                let player_value = get_best_5_card_hand(&empty_hand);

                if player_value > highest_value {
                    highest_value = player_value;
                    highest_index = index;
                }
            }
        }
        println!(
            "Player: {:?} has the highest hand. They start the betting round",
            self.players[highest_index].player_name
        );
        highest_index
    }

    /// Puts the small blind and big blind into the pot and updates the relevant players.
    ///
    /// The small blind is placed by the player immediately to the left of the dealer, and the big blind is placed by
    /// the player two positions to the left. The method updates the players' last move and the pot accordingly.
    ///
    /// # Updates
    /// - Updates the highest bet, pot amount, and last move for the players involved.
    pub async fn put_blinds(&mut self) {
        let index_small = ((self.dealer + 1) % self.players.len() as u32) as usize;
        let index_big = ((self.dealer + 2) % self.players.len() as u32) as usize;

        self.players[index_small].bet(self.minimum_bet / 2);
        self.players[index_big].bet(self.minimum_bet);

        self.players[index_small].last_move = "Small Blind".to_string();
        self.players[index_big].last_move = "Big Blind".to_string();

        self.highest_bet = self.minimum_bet;
        self.pot = self.minimum_bet + (self.minimum_bet / 2)
    }

    /// Handles the antes phase for a Seven Card Stud game.
    ///
    /// Each player contributes 2 to the pot, except the player at the `starting_idx` who contributes 3. The highest bet
    /// is updated, and the pot is adjusted accordingly.
    ///
    /// # Parameters
    /// - `starting_idx`: The index of the player who is required to place 3 in the ante phase.
    pub async fn seven_card_antes(&mut self, starting_idx: usize) {
        for player in &mut self.players {
            player.bet(2);
            self.pot += 2;
        }

        self.players[starting_idx].bet(3);
        self.pot += 3;
        self.highest_bet = self.minimum_bet;
    }

    /// Sets the tokens for the players involved in the blinds phase.
    ///
    /// The dealer token is set for the current dealer, the small blind token is set for the player to the left of the dealer,
    /// and the big blind token is set for the player two positions to the left of the dealer.
    pub fn set_player_tokens_blinds(&mut self) {
        let index_small = ((self.dealer + 1) % self.players.len() as u32) as usize;
        let index_big = ((self.dealer + 2) % self.players.len() as u32) as usize;

        self.players[self.dealer as usize].token = "D".to_string();

        if self.players[index_small].token != *"D".to_string() {
            self.players[index_small].token = "SB".to_string();
        }

        if self.players[index_big].token != *"D".to_string() {
            self.players[index_big].token = "BB".to_string();
        }
    }

    /// Sets only the dealer token for the current dealer.
    ///
    /// This method ensures that only the dealer is marked with the dealer token (`D`), without affecting other players.
    pub fn set_player_tokens_only_dealer(&mut self) {
        self.players[self.dealer as usize].token = "D".to_string();
    }

    //****************************************************************
    // DEALING INITIAL AND COMMUNITY CARDS
    //****************************************************************

    /// Deals face-down cards to all players who have not folded.
    ///
    /// This method draws a specified number of face-down cards for each player who has not folded and adds them to their hand.
    /// The cards are drawn from the game's deck. The number of cards dealt is controlled by the `num_cards` parameter.
    ///
    /// # Parameters
    /// - `num_cards`: The number of face-down cards to deal to each player who has not folded.
    pub async fn deal_face_down(&mut self, num_cards: u32) {
        if self.get_remaining_player_count() <= 1 {
            return;
        }

        for _ in 0..num_cards {
            for player in &mut self.players {
                if !player.player_choices.contains_key("Fold") {
                    let _ = player.player_hand.draw(&mut self.deck, 1);
                }
            }
        }
    }

    /// Deals face-up cards to all players who have not folded.
    ///
    /// This method draws a specified number of face-up cards for each player who has not folded and adds them to their hand.
    /// The cards are drawn from the game's deck. The number of cards dealt is controlled by the `num_cards` parameter.
    ///
    /// # Parameters
    /// - `num_cards`: The number of face-up cards to deal to each player who has not folded.
    pub async fn deal_face_up(&mut self, num_cards: u32) {
        if self.get_remaining_player_count() <= 1 {
            return;
        }

        for _ in 0..num_cards {
            for player in &mut self.players {
                if !player.player_choices.contains_key("Fold") {
                    let _ = player.player_hand.draw_face_up(&mut self.deck, 1);
                }
            }
        }
    }

    /// Deals community cards that are shared by all players.
    ///
    /// This method draws a specified number of face-up community cards from the deck and adds them to the community hand.
    /// The number of cards dealt is controlled by the `num` parameter.
    ///
    /// # Parameters
    /// - `num`: The number of community cards to deal.
    pub async fn deal_community_cards(&mut self, num: u32) {
        if self.get_remaining_player_count() <= 1_u32 {
            return;
        }

        for _ in 0..num {
            let _ = self.community_cards.draw_face_up(&mut self.deck, 1);
        }
    }

    //****************************************************************
    // BETTING ROUND FUNCTIONS
    //****************************************************************

    /// Starts a new betting round by resetting player choices and the highest bet.
    ///
    /// This method resets the "discard cards prompted" flag, and if the round number is zero (the first round),
    /// it returns early. For all other rounds, it calls `new_round_choices()` on each player to reset their
    /// choices for the new betting round. The highest bet is also reset to 0.
    pub async fn new_betting_round(&mut self) {
        self.discard_cards_prompted = false;

        if self.round_number == 0 {
            return;
        }

        for player in &mut self.players {
            player.new_round_choices().await;
        }
        self.highest_bet = 0;
    }

    /// Determines whether the current betting round is complete.
    ///
    /// The round is considered complete if all players have either folded or matched the highest bet. The method
    /// checks if all players have made their moves, considering folds and whether they have placed the appropriate
    /// bet (either by calling, raising, or checking). If any player still needs to act, the round is not complete.
    ///
    /// # Returns
    /// - `true`: If all players have completed their betting actions.
    /// - `false`: If there are players who still need to act.
    pub async fn is_betting_round_complete(&self) -> bool {
        let mut players_remaining = 0;
        if self.get_remaining_player_count() <= 1_u32 {
            return true;
        }

        for player in self.players.clone() {
            let placed_in_pot = match player.player_choices.get("PlacedInPot") {
                Some(PlayerChoice::PlacedInPot(amount)) => *amount,
                _ => 0,
            };

            if !player.player_choices.contains_key("Fold")
                && (placed_in_pot != self.highest_bet
                    || (placed_in_pot == 0 && !player.player_choices.contains_key("Check")))
            {
                players_remaining += 1;
            }
        }

        if players_remaining > 0 {
            return false;
        }
        true
    }

    /// Sets the current player based on the provided index.
    ///
    /// This method updates the `current_player` field to the player at the specified `idx` position. It ensures
    /// that the index is valid (i.e., within the range of the players list).
    ///
    /// # Parameters
    /// - `idx`: The index of the player who will become the current player.
    pub async fn set_current_player(&mut self, idx: usize) {
        if self.players.is_empty() {
            return;
        }

        if idx >= self.players.len() {
            return;
        }

        self.current_player = self.players[idx].clone();
    }

    /// Updates the current player to the next player who has not folded.
    ///
    /// This method iterates through the players list and finds the next player who has not folded. The `current_player`
    /// is then set to this player.
    pub async fn update_current_player(&mut self) {
        if self.players.is_empty() {
            return;
        }

        if let Some(current_index) = self
            .players
            .iter()
            .position(|p| p.player_name == self.current_player.player_name)
        {
            let mut i = 1;
            while i < self.players.len() {
                let potential_index = (current_index + i) % self.players.len();
                if !self.players[potential_index]
                    .player_choices
                    .contains_key("Fold")
                {
                    self.current_player = self.players[potential_index].clone();
                    return;
                }

                i += 1;
            }
        } else {
            println!("Player not in game_state.player list");
            self.current_player = self.players[0].clone();
        }
    }

    /// Retrieves the available actions for the current player.
    ///
    /// This method calculates the minimum and maximum raise amounts for the current player based on their previous bet,
    /// the highest bet, and their available money. It then updates the `raise_min_max` values to reflect the player's
    /// possible betting actions (i.e., the minimum and maximum raise amounts).
    pub async fn get_available_player_actions(&mut self) {
        self.discard_cards_prompted = false;

        let player = self.current_player.clone();

        if player.player_choices.contains_key("Fold") {
            return;
        }

        let min_raise = self.minimum_bet + self.highest_bet;

        let placed_in_pot = match player.player_choices.get("PlacedInPot") {
            Some(PlayerChoice::PlacedInPot(amount)) => *amount,
            _ => 0,
        };

        let max_raise = placed_in_pot + player.player_money;

        self.raise_min_max = (min_raise, max_raise);
    }

    /// Handles a betting action from the current player.
    ///
    /// This method processes a player's action (e.g., fold, check, raise, call) and updates the game state accordingly.
    /// It also records the action in the `action_history`.
    ///
    /// # Parameters
    /// - `action`: The action the player wants to perform (e.g., "Fold", "Check", "Raise", "Call").
    /// - `bet_amount`: The bet amount for actions like "Raise" (this parameter is ignored for actions like "Fold" or "Check").
    pub async fn handle_player_betting_message(&mut self, action: String, bet_amount: u32) {
        match action.as_str() {
            "Fold" => {
                self.current_player_fold().await;
                self.action_history
                    .push(format!("{} folded", self.current_player.player_name));
            }
            "Check" => {
                self.current_player_check().await;
                self.action_history
                    .push(format!("{} checked", self.current_player.player_name));
            }
            "Raise" => {
                self.current_player_bet(bet_amount).await;
                self.action_history.push(format!(
                    "{} raised to {}",
                    self.current_player.player_name, self.highest_bet
                ));
            }
            "Call" => {
                self.current_player_call().await;
                self.action_history
                    .push(format!("{} called", self.current_player.player_name));
            }
            _ => {
                println!("Unknown action: {}. Player folding", action);
                self.current_player_fold().await;
                self.action_history
                    .push(format!("{} folded", self.current_player.player_name));
            }
        }
    }

    /// Handles a player's fold action, marking them as having folded and updating their state accordingly.
    ///
    /// This method sets the player's last move to "Fold", marks them as folded, and updates the game state.
    /// It also prints a message to indicate that the player has folded.
    pub async fn current_player_fold(&mut self) {
        if let Some(player) = self.find_player_by_name().await {
            player.fold();
            player.last_move = "Fold".to_string();
            println!("PLAYER FOLDED: {:?}", player.player_name);
        } else {
            println!("Player not found in the list.");
        }
    }

    /// Handles a player's check action, marking them as having checked and updating their state accordingly.
    ///
    /// This method sets the player's last move to "Check" and updates their state accordingly.
    /// It also prints a message to indicate that the player has checked.
    pub async fn current_player_check(&mut self) {
        if let Some(player) = self.find_player_by_name().await {
            player.check();
            player.last_move = "Check".to_string();
            println!("PLAYER CHECKED: {:?}", player.player_name);
        } else {
            println!("Player not found in the list.");
        }
    }

    /// Handles a player's call action, making them call the highest bet by adding the necessary chips to the pot.
    ///
    /// This method calculates how much the player needs to call based on the difference between the highest bet
    /// and the player's current contribution, then updates the pot and player state accordingly. The player's
    /// last move is also set to "Call".
    pub async fn current_player_call(&mut self) {
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|p| p.player_name == self.current_player.player_name)
        {
            let player_put_in_pot = match player.player_choices.get("PlacedInPot") {
                Some(PlayerChoice::PlacedInPot(amount)) => *amount,
                _ => 0,
            };

            println!(
                "HIGHEST: {:?}, put in pot: {:?}",
                self.highest_bet, player_put_in_pot
            );
            let call_amount = self.highest_bet - player_put_in_pot;
            self.pot += call_amount;

            player.call(call_amount);
            player.last_move = "Call".to_string();
            println!("PLAYER CALLED: {:?}", player.player_name);
        } else {
            println!("Player not found in the list.");
        }
    }

    /// Handles a player's raise action, making them raise the bet by the specified amount.
    ///
    /// This method calculates the raise amount based on the difference between the desired bet value and the player's
    /// current contribution, updates the pot and the player's state, and sets their last move to "Raise".
    ///
    /// # Parameters
    /// - `bet_value`: The value the player wants to raise to.
    pub async fn current_player_bet(&mut self, bet_value: u32) {
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|p| p.player_name == self.current_player.player_name)
        {
            let player_put_in_pot = match player.player_choices.get("PlacedInPot") {
                Some(PlayerChoice::PlacedInPot(amount)) => *amount,
                _ => 0,
            };

            let raise_amount = bet_value - player_put_in_pot;

            self.pot += raise_amount;
            self.highest_bet = bet_value;

            player.bet(raise_amount);
            player.last_move = "Raise".to_string();
            println!("PLAYER RAISED: {:?}", player.player_name);
        } else {
            println!("Player not found in the list.");
        }
    }

    //****************************************************************
    // DISCARD ROUND
    //****************************************************************

    /// Starts a new discard round, where players can discard unwanted cards and replace them.
    ///
    /// This method sets the `discard_cards_prompted` flag to `true` and then calls `new_round_choices()`
    /// for each player to reset their choices for the discard round. This prepares the game state for players
    /// to decide which cards they wish to discard and replace.
    pub async fn new_discard_round(&mut self) {
        self.discard_cards_prompted = true;

        for player in &mut self.players {
            player.new_round_choices().await;
        }
    }

    /// Handles a player's discard action, where they swap selected cards for new ones.
    ///
    /// This method takes the player's name and a list of indices of the cards they wish to discard. If the indices
    /// are valid, it swaps the selected cards in the player's hand with new ones from the deck. The method logs the
    /// action of discarding the old card and receiving a new one. It also handles the error case if there is a problem
    /// dealing new cards.
    pub async fn handle_player_discard_message(
        &mut self,
        player_name: String,
        card_index: Vec<usize>,
    ) {
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|p| p.player_name == player_name)
        {
            if card_index.is_empty() {
                return;
            }

            for &index in &card_index {
                if index < player.player_hand.cards.len() {
                    match self.deck.deal_card() {
                        Ok(new_card) => {
                            println!(
                                "Player {:?} is swapping {:?} for the new card {:?}",
                                player.player_name, player.player_hand.cards[index], new_card
                            );
                            player.player_hand.cards[index] = new_card;
                        }
                        Err(e) => {
                            println!("Error dealing card: {}", e);
                        }
                    }
                }
            }
        }
    }

    //****************************************************************
    // SCORING ROUND
    //****************************************************************

    /// Determines the winner based on a five-card hand for each player.
    ///
    /// This function evaluates each player's best five-card hand and compares it to the best hand found
    /// so far. The winner is the player with the highest-ranking hand according to the standard poker hand rankings.
    ///
    /// # Asynchronous
    /// This function is asynchronous and calls `reveal_cards()` to ensure all players' cards are revealed before determining the winner.
    ///
    /// # Updates
    /// - Updates the `self.winner` field with the list of winning players.
    ///
    /// # Note
    /// The comparison of hands is done using the `ScoredHand` enum and assumes that the best 5-card hand for each player is determined
    /// by the `get_best_5_card_hand()` function.
    ///
    /// # Example
    /// ```rust
    /// game.determine_winner_five_card().await;
    /// ```
    #[allow(clippy::comparison_chain)]
    pub async fn determine_winner_five_card(&mut self) {
        let mut winners: Vec<(Player, ScoredHand)> = Vec::new();
        let mut best_hand = ScoredHand::HighCard([Value::Two; 5]);

        self.reveal_cards().await;

        for player in self.players.iter() {
            if !player.player_choices.contains_key("Fold") {
                let best_hand_from_player = get_best_5_card_hand(&player.player_hand);

                if best_hand_from_player > best_hand {
                    winners.clear();
                    winners.push((player.clone(), best_hand_from_player));
                    best_hand = best_hand_from_player;
                } else if best_hand_from_player == best_hand {
                    winners.push((player.clone(), best_hand_from_player));
                }
            }
        }
        self.winner = winners;
    }

    /// Determines the winner based on a seven-card hand for each player.
    ///
    /// This function evaluates each player's best seven-card hand and compares it to the best hand found
    /// so far. The winner is the player with the highest-ranking hand according to poker hand rankings.
    ///
    /// # Asynchronous
    /// This function is asynchronous and calls `reveal_cards()` to ensure all players' cards are revealed before determining the winner.
    ///
    /// # Special Case
    /// If there is only one player remaining, they win automatically.
    ///
    /// # Updates
    /// - Updates the `self.winner` field with the list of winning players.
    ///
    /// # Note
    /// The comparison of hands is done using the `ScoredHand` enum and assumes that the best 7-card hand for each player is determined
    /// by the `get_best_from_7_card_hand()` function.
    ///
    /// # Example
    /// ```rust
    /// game.determine_winner_seven_card().await;
    /// ```
    #[allow(clippy::comparison_chain)]
    pub async fn determine_winner_seven_card(&mut self) {
        let mut winners: Vec<(Player, ScoredHand)> = Vec::new();
        let mut best_hand = ScoredHand::HighCard([Value::Empty; 5]);

        self.reveal_cards().await;

        if self.get_remaining_player_count() <= 1 {
            for player in self.players.iter() {
                if !player.player_choices.contains_key("Fold") {
                    winners.push((player.clone(), best_hand));
                    self.winner.push(winners[0].clone());
                    return;
                }
            }
        }

        for player in self.players.iter() {
            if !player.player_choices.contains_key("Fold") {
                let best_hand_from_player = get_best_from_7_card_hand(&player.player_hand);

                if best_hand_from_player > best_hand {
                    winners.clear();
                    winners.push((player.clone(), best_hand_from_player));
                    best_hand = best_hand_from_player;
                } else if best_hand_from_player == best_hand {
                    winners.push((player.clone(), best_hand_from_player));
                }
            }
        }

        for player in winners {
            self.winner.push(player);
        }
    }

    /// Determines the winner based on Texas Hold'em rules (7 cards: 2 hole cards + 5 community cards).
    ///
    /// This function evaluates each player's best seven-card hand formed by combining their 2 private cards (hole cards)
    /// with the 5 community cards, then compares it to the best hand found so far. The winner is the player with the highest-ranking hand.
    ///
    /// # Asynchronous
    /// This function is asynchronous and calls `reveal_cards()` to ensure all players' cards are revealed before determining the winner.
    ///
    /// # Special Case
    /// If there is only one player remaining, they win automatically.
    ///
    /// # Updates
    /// - Updates the `self.winner` field with the list of winning players.
    ///
    /// # Note
    /// The comparison of hands is done using the `ScoredHand` enum and assumes that the best 7-card hand for each player is determined
    /// by combining their hole cards with the community cards and using the `get_best_from_7_card_hand()` function.
    ///
    /// # Example
    /// ```rust
    /// game.determine_winner_texas().await;
    /// ```
    #[allow(clippy::comparison_chain)]
    pub async fn determine_winner_texas(&mut self) {
        let mut highest_value: ScoredHand = ScoredHand::HighCard([Value::Empty; 5]);
        let mut winners: Vec<(Player, ScoredHand)> = Vec::new();

        self.reveal_cards().await;

        if self.get_remaining_player_count() <= 1 {
            for player in self.players.iter() {
                if !player.player_choices.contains_key("Fold") {
                    winners.push((player.clone(), highest_value));
                    self.winner.push(winners[0].clone());
                    return;
                }
            }
        }

        for player in self.players.iter() {
            if !player.player_choices.contains_key("Fold") {
                let mut combined_hand = Hand::new(7);
                for card in player.player_hand.cards.clone() {
                    combined_hand.cards.push(card);
                }
                for card in self.community_cards.cards.clone() {
                    combined_hand.cards.push(card);
                }

                let best_hand_from_player = get_best_from_7_card_hand(&combined_hand);

                if best_hand_from_player > highest_value {
                    winners.clear();
                    winners.push((player.clone(), best_hand_from_player));
                    highest_value = best_hand_from_player;
                } else if best_hand_from_player == highest_value {
                    winners.push((player.clone(), best_hand_from_player));
                }
            }
        }

        for player in winners {
            self.winner.push(player);
        }
    }

    /// Reveals all the cards of players who have not folded.
    ///
    /// This function ensures that all players who have not folded have their cards turned face-up.
    /// It is called before determining the winner to make sure all the relevant cards are visible.
    ///
    /// # Asynchronous
    /// This function is asynchronous and modifies each player's cards in place.
    ///
    /// # Example
    /// ```rust
    /// game.reveal_cards().await;
    /// ```
    pub async fn reveal_cards(&mut self) {
        for player in self.players.iter_mut() {
            if !player.player_choices.contains_key("Fold") {
                for card in player.player_hand.cards.iter_mut() {
                    card.face_up = true;
                }
            }
        }
    }

    /// Pays the winners by distributing the pot among them.
    ///
    /// This function divides the total pot among the winning players. If there are multiple winners, the pot is split evenly.
    /// It also updates the player's statistics like total money, total wins, total losses, etc.
    ///
    /// # Updates
    /// - Updates player stats (total games, total money, wins, losses).
    /// - Sets the pot to zero after distribution.
    ///
    /// # Special Case
    /// If there are no winners, the function prints a message and exits without changing anything.
    ///
    /// # Example
    /// ```rust
    /// game.pay_winners().await;
    /// ```
    pub async fn pay_winners(&mut self) {
        let winner_count = self.winner.len();

        if winner_count == 0 {
            println!("No winners found.");
            return;
        }

        let pot_share = self.pot / winner_count as u32;

        for (winner, _) in self.winner.clone() {
            for player in self.players.iter_mut() {
                player.total_games += 1;
                if player.player_name == winner.player_name {
                    player.player_money += pot_share;
                    player.total_earnings += pot_share;
                    player.round_win = pot_share;
                    player.total_wins += 1;
                    println!("{} won {} chips!", player.player_name, pot_share);
                } else {
                    player.total_losses += 1;
                }
            }
        }
        self.pot = 0;
    }
    //****************************************************************
    // DB FNS
    //****************************************************************

    /// Writes the current game results to the database.
    ///
    /// This function creates a new database client, inserts the current game state into the database,
    /// and updates each player's information in the database.
    ///
    /// # Asynchronous
    /// This function is asynchronous and makes calls to a MongoDB database using a client from `DbClient`.
    ///
    /// # Example
    /// ```rust
    /// game.write_results_to_db().await;
    /// ```
    pub async fn write_results_to_db(&self) {
        let db_client = DbClient::new(&MONGO_URI).await.unwrap();
        let _result = db_client.insert(self).await;

        for player in self.players.clone() {
            let _ = db_client.update_one(&player).await;
        }
    }

    //****************************************************************
    // MISC HELPER FUNCTIONS
    //****************************************************************

    /// Returns the current game state.
    ///
    /// This function returns the current instance of the game (`self`) as an immutable reference.
    /// It is useful for querying the current game state without modifying it.
    ///
    /// # Example
    /// ```rust
    /// let game_state = game.get_game_state();
    /// ```
    pub fn get_game_state(&self) -> &Self {
        self
    }

    /// Finds a player by their name and returns a mutable reference to the player.
    ///
    /// This function searches for a player based on the `current_player`'s name and returns a mutable
    /// reference to the player if they are found. This allows modifying the player's state directly.
    ///
    /// # Returns
    /// - `Some(&mut Player)` if a player with the given name is found.
    async fn find_player_by_name(&mut self) -> Option<&mut Player> {
        self.players
            .iter_mut()
            .find(|p| p.player_name == self.current_player.player_name)
    }

    /// Gets the number of remaining players who have not folded.
    ///
    /// This function counts the number of players who have not chosen to fold. It is typically used to determine
    /// the number of active players remaining in the game.
    ///
    /// # Returns
    /// - The number of players who have not folded as a `u32`.
    ///
    /// # Example
    /// ```rust
    /// let remaining_players = game.get_remaining_player_count();
    /// ```
    pub fn get_remaining_player_count(&self) -> u32 {
        let mut count = 0;
        for player in self.players.clone() {
            if !player.player_choices.contains_key("Fold") {
                count += 1;
            }
        }
        count
    }

    /// Handles the demo mode status message.
    ///
    /// This function sets the `demo_mode` status based on the input `status` string. The status
    /// is typically used to control whether the game runs in demo mode or not.
    ///
    /// # Parameters
    /// - `status`: A string representing the new demo mode status (e.g., "enabled" or "disabled").
    pub async fn handle_demo_mode_message(&mut self, status: String) {
        self.demo_mode = status
    }
}

//****************************************************************
// DATABASE IMPLEMENTATION
//****************************************************************

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
            "lobby": bson::to_bson(&self.lobby).map_err(|e| e.to_string())?,
            "spectators": bson::to_bson(&self.spectators).map_err(|e| e.to_string())?,
            "dealer_choice_spectators": bson::to_bson(&self.dealer_choice_spectators).map_err(|e| e.to_string())?,
            "deck": bson::to_bson(&self.deck).map_err(|e| e.to_string())?,
            "game_id": self.game_id as i64,
            "game_variant": self.game_variant.clone(),
            "max_players": self.max_players,
            "hand_size": self.hand_size as i64,
            "pot": self.pot as i64,
            "demo_mode": self.demo_mode.clone(),
            "round_number": self.round_number as i64,
            "minimum_bet": self.minimum_bet as i64,
            "highest_bet": self.highest_bet as i64,
            "dealer": self.dealer as i64,
            "winner": bson::to_bson(&self.winner).map_err(|e| e.to_string())?,
            "current_player": bson::to_bson(&self.current_player).map_err(|e| e.to_string())?,
            "player_action": &self.player_action.clone(),
            "community_cards": bson::to_bson(&self.community_cards).map_err(|e| e.to_string())?,
            "discard_cards_prompted": self.discard_cards_prompted,
            "current_action_string": self.current_action_string.clone(),
            "action_history": bson::to_bson(&self.action_history).map_err(|e| e.to_string())?,
            "raise_min_max": bson::to_bson(&self.raise_min_max).map_err(|e| e.to_string())?,
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
            lobby: bson::from_bson(doc.get("lobby").cloned().unwrap_or(bson::Bson::Null))
                .map_err(|e| e.to_string())?,
            spectators: bson::from_bson(doc.get("spectators").cloned().unwrap_or(bson::Bson::Null))
                .map_err(|e| e.to_string())?,
            dealer_choice_spectators: bson::from_bson(
                doc.get("dealer_choice_spectators")
                    .cloned()
                    .unwrap_or(bson::Bson::Null),
            )
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
            demo_mode: doc.get_str("demo_mode").unwrap_or("").to_string(),
            round_number: doc.get_i64("round_number").map_err(|e| e.to_string())? as u32,
            minimum_bet: doc.get_i64("minimum_bet").map_err(|e| e.to_string())? as u32,
            highest_bet: doc.get_i64("highest_bet").map_err(|e| e.to_string())? as u32,
            dealer: doc.get_i64("dealer").map_err(|e| e.to_string())? as u32,
            winner: bson::from_bson(doc.get("winner").cloned().unwrap_or(bson::Bson::Null))
                .map_err(|e| e.to_string())?,
            current_player: bson::from_bson(
                doc.get("current_player")
                    .cloned()
                    .unwrap_or(bson::Bson::Null),
            )
            .map_err(|e| e.to_string())?,
            player_action: doc
                .get_str("player_action")
                .map_err(|e| e.to_string())?
                .to_string(),
            community_cards: bson::from_bson(
                doc.get("community_cards")
                    .cloned()
                    .unwrap_or(bson::Bson::Null),
            )
            .map_err(|e| e.to_string())?,
            discard_cards_prompted: doc
                .get_bool("discard_cards_prompted")
                .map_err(|e| e.to_string())?,
            current_action_string: std::string::String::from(
                doc.get_str("current_action_string")
                    .map_err(|e| e.to_string())?,
            ),
            action_history: bson::from_bson(
                doc.get("action_history")
                    .cloned()
                    .unwrap_or(bson::Bson::Null),
            )
            .map_err(|e| e.to_string())?,
            raise_min_max: bson::from_bson(
                doc.get("raise_min_max")
                    .cloned()
                    .unwrap_or(bson::Bson::Null),
            )
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
