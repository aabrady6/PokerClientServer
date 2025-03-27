//! Core game rules that are common amongst poker variants and poker game values
//!
//! Creates calls to structs that are inside of GameState and helps perform passing
//! of references and data between these structs.
//! GameState meant to be by in large asynchronous with updates being made to the database on game completion
//!

use crate::db::dbclient::{DbClient, DbEntity};
use crate::game::client_messages::MessageType;
use crate::game::score_hands::get_best_5_card_hand;
use crate::game::score_hands::get_best_from_7_card_hand;
use mongodb::bson;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};

use crate::game::card::Value;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::player::Player;
use crate::game::player::PlayerChoice;
use crate::game::score_hands::ScoredHand;
use std::collections::HashMap;
use tokio::sync::oneshot;

/// A struct that represents the state of the game, containing essential information about the game setup,
/// players, and current game status.
#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    pub game_variant: String,
    pub deck: Deck,
    pub game_id: u32,
    pub hand_size: u8,
    pub max_players: i64,
    pub stage_number: u32,
    pub round_number: u32,
    pub minimum_bet: u32,
    pub highest_bet: u32,
    pub current_player: Player,
    pub player_action: String,
    pub players: Vec<Player>,
    pub winner: Vec<(Player, ScoredHand)>,
    pub community_cards: Hand,
    pub pot: u32,
    pub dealer: u32,
    pub discard_cards_prompted: bool,
    pub current_action_string: String,
    pub action_history: Vec<String>,
    pub raise_min_max: (u32, u32),
    #[serde(skip)]
    pub pending_actions: HashMap<String, oneshot::Sender<(String, u32)>>,
}

impl Clone for GameState {
    fn clone(&self) -> Self {
        Self {
            game_variant: self.game_variant.clone(),
            deck: self.deck.clone(),
            game_id: self.game_id,
            hand_size: self.hand_size,
            max_players: self.max_players,
            stage_number: self.stage_number,
            round_number: self.round_number,
            minimum_bet: self.minimum_bet,
            highest_bet: self.highest_bet,
            current_player: self.current_player.clone(),
            player_action: self.player_action.clone(),
            players: self.players.clone(),
            winner: self.winner.clone(),
            community_cards: self.community_cards.clone(),
            pot: self.pot,
            dealer: self.dealer,
            discard_cards_prompted: self.discard_cards_prompted,
            current_action_string: self.current_action_string.clone(),
            action_history: self.action_history.clone(),
            raise_min_max: self.raise_min_max,
            pending_actions: HashMap::new(),
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            players: Vec::new(),
            deck: Deck::new(),
            game_variant: "".to_string(),
            game_id: 0,
            hand_size: 5,
            max_players: 5,
            pot: 0,
            stage_number: 0,
            round_number: 0,
            minimum_bet: 5,
            highest_bet: 0,
            dealer: 0,
            winner: vec![],
            current_player: Player::new(""),
            player_action: String::new(),
            community_cards: Hand::new(5),
            discard_cards_prompted: false,
            current_action_string: "".to_string(),
            action_history: Vec::new(),
            raise_min_max: (0, 0),
            pending_actions: HashMap::new(),
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    //****************************************************************
    //GAME INITIALIZATION FUNCTIONS
    //****************************************************************

    pub async fn new_five_card_round(&mut self) {
        let uri = "mongodb://localhost:27017";
        let db_client = DbClient::new(uri).await.unwrap();
        let game_id = db_client.get_new_game_id().await;

        self.deck = Deck::new();
        self.game_variant = "5 Card Draw".to_string();
        self.game_id = game_id;
        self.hand_size = 5;
        self.max_players = 7;
        self.pot = 0;
        self.minimum_bet = 5;
        self.round_number = 0;
        self.highest_bet = 0;
        self.winner = vec![];
        self.current_player = Player::new("");
        self.player_action = String::new();
        self.community_cards = Hand::new(5);
        self.discard_cards_prompted = false;
        self.current_action_string = "".to_string();
        self.action_history = Vec::new();
        self.raise_min_max = (0, 0);
        self.pending_actions = HashMap::new();

        for player in &mut self.players {
            player.reset_choices();
            player.total_wagered_per_game = 0;
            player.round_win = 0;
            player.player_hand = Hand::new(5);
            player.infinite_money();
        }

        self.deal_face_down(5).await;
        self.put_blinds().await;
        self.set_player_tokens();
    }

    pub async fn new_seven_card_round(&mut self) {
        let uri = "mongodb://localhost:27017";
        let db_client = DbClient::new(uri).await.unwrap();
        let game_id = db_client.get_new_game_id().await;

        self.deck = Deck::new();
        self.game_variant = "7 Card Stud".to_string();
        self.game_id = game_id;
        self.hand_size = 7;
        self.max_players = 7;
        self.pot = 0;
        self.minimum_bet = 5;
        self.round_number = 0;
        self.highest_bet = 0;
        self.winner = vec![];
        self.current_player = Player::new("");
        self.player_action = String::new();
        self.community_cards = Hand::new(5);
        self.discard_cards_prompted = false;
        self.current_action_string = "".to_string();
        self.action_history = Vec::new();
        self.raise_min_max = (0, 0);
        self.pending_actions = HashMap::new();

        for player in &mut self.players {
            player.reset_choices();
            player.total_wagered_per_game = 0;
            player.round_win = 0;
            player.player_hand = Hand::new(7);
            player.infinite_money();
        }

        self.deal_face_down(2).await;
        self.deal_face_up(1).await;
        self.put_blinds().await;
        self.set_player_tokens();
    }

    pub async fn new_texas_holdem_round(&mut self) {
        let uri = "mongodb://localhost:27017";
        let db_client = DbClient::new(uri).await.unwrap();
        let game_id = db_client.get_new_game_id().await;

        self.deck = Deck::new();
        self.game_variant = "Texas Hold'em".to_string();
        self.game_id = game_id;
        self.hand_size = 2;
        self.max_players = 7;
        self.pot = 0;
        self.minimum_bet = 5;
        self.round_number = 0;
        self.highest_bet = 0;
        self.winner = vec![];
        self.current_player = Player::new("");
        self.player_action = String::new();
        self.community_cards = Hand::new(5);
        self.discard_cards_prompted = false;
        self.current_action_string = "".to_string();
        self.action_history = Vec::new();
        self.raise_min_max = (0, 0);
        self.pending_actions = HashMap::new();

        for player in &mut self.players {
            player.reset_choices();
            player.total_wagered_per_game = 0;
            player.round_win = 0;
            player.player_hand = Hand::new(2);
            player.infinite_money();
        }

        self.deal_face_down(2).await;
        self.put_blinds().await;
        self.set_player_tokens();
    }

    //****************************************************************
    // PLAYER FUNCTIONS
    //****************************************************************

    pub fn insert_player(&mut self, player: &Player) -> Result<(), String> {
        if self.players.len() >= self.max_players as usize {
            return Err("Unable to push".to_string());
        }
        self.players.push(player.clone());
        Ok(())
    }

    //****************************************************************
    // DEALERS, BLINDS AND STARTING POSITIONS FUNCTIONS
    //****************************************************************

    pub async fn rotate_dealer(&mut self) {
        self.players[self.dealer as usize].token = "".to_string();
        self.dealer = (self.dealer + 1) % self.players.len() as u32;
        self.players[self.dealer as usize].token = "D".to_string();
    }

    pub async fn get_blinds_starting_player_index(&self) -> usize {
        ((self.dealer + 3) % self.players.len() as u32) as usize
    }

    pub async fn determine_lowest_door(&self) -> u32 {
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

        lowest_index as u32
    }

    pub async fn determine_highest_door(&self) -> u32 {
        let mut highest = self.players[0].clone();
        let mut highest_index = 0;

        for (index, player) in self.players.iter().enumerate() {
            if !player.player_choices.contains_key("Fold") {
                let mut face_up = player.player_hand.get_face_up_cards();
                let player_value = face_up.get_largest_value();

                let mut highest_face_up = highest.player_hand.get_face_up_cards();
                let highest_value = highest_face_up.get_largest_value();

                if player_value > highest_value {
                    highest = player.clone();
                    highest_index = index;
                }
            }
        }

        highest_index as u32
    }

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

    pub fn set_player_tokens(&mut self) {
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

    //****************************************************************
    // DEALING INITIAL AND COMMUNITY CARDS
    //****************************************************************

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

    pub async fn deal_community_cards(&mut self, num: u32) {
        if self.get_remaining_player_count() <= 1_u32 {
            return;
        }

        for _ in 0..num {
            let _ = self.community_cards.draw(&mut self.deck, 1);
        }
    }

    //****************************************************************
    // BETTING ROUND FUNCTIONS
    //****************************************************************

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

    pub async fn set_current_player(&mut self, idx: usize) {
        if self.players.is_empty() {
            return;
        }

        if idx >= self.players.len() {
            return;
        }

        self.current_player = self.players[idx].clone();
    }

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
            println!("CANNNOOOOOOOTTT FIND THEM");
            self.current_player = self.players[0].clone();
        }
    }

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

        println!("MIN: {:?}, MAX: {:?}", min_raise, max_raise);
        self.raise_min_max = (min_raise, max_raise);
    }

    pub async fn handle_player_betting_message(&mut self, action_message: MessageType) {
        if let MessageType::PlayerAction {
            action,
            player_name,
            bet_amount,
        } = action_message
        {
            println!("Player {:?} Action: {}", player_name, action);

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
                    println!("Unknown action: {}", action);
                }
            }
        } else {
            println!("Received an unexpected message type: {:?}", action_message);
        }
    }

    pub async fn current_player_fold(&mut self) {
        if let Some(player) = self.find_player_by_name().await {
            player.fold();
            player.last_move = "Fold".to_string();
            println!("PLAYER FOLDED: {:?}", player.player_name);
        } else {
            println!("Player not found in the list.");
        }
    }

    pub async fn current_player_check(&mut self) {
        if let Some(player) = self.find_player_by_name().await {
            player.check();
            player.last_move = "Check".to_string();
        } else {
            println!("Player not found in the list.");
        }
    }

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

            let call_amount = self.highest_bet - player_put_in_pot;
            self.pot += call_amount;

            player.call(call_amount);
            player.last_move = "Call".to_string();
            println!("PLAYER CALLED: {:?}", player.player_name);
        } else {
            println!("Player not found in the list.");
        }
    }

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
    pub async fn new_discard_round(&mut self) {
        self.discard_cards_prompted = true;

        for player in &mut self.players {
            player.new_round_choices().await;
        }
    }

    pub async fn handle_player_discard_message(&mut self, discard_message: MessageType) {
        if let MessageType::DiscardAction {
            player_name,
            card_index,
        } = discard_message
        {
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
                                    "Player {:?} is SWAPPING {:?} for the new card {:?}",
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
    }

    //****************************************************************
    // SCORING ROUND
    //****************************************************************

    pub async fn new_scoring_round(&mut self) {
        self.rotate_dealer().await;
    }

    #[allow(clippy::comparison_chain)]
    pub async fn determine_winner_five_card(&mut self) {
        let mut winners: Vec<(Player, ScoredHand)> = Vec::new();
        let mut best_hand = ScoredHand::HighCard([Value::Two; 5]);

        //if self.get_remaining_player_count() <= 1 {
        //    for player in self.players.iter() {
        //        if !player.player_choices.contains_key("Fold") {
        //            winners.push((player.clone(), best_hand));
        //        }
        //    }
        //    self.winner = winners;
        //    return;
        //}
        //
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
    //
    //fn determine_winner_seven_card(&self) -> Vec<(u32, ScoredHand)> {
    //    let mut winners: Vec<(u32, ScoredHand)> = Vec::new();
    //    let mut best_hand = ScoredHand::HighCard([Value::Two; 5]);
    //
    //    if self.get_remaining_player_count() <= 1 {
    //        for (index, player) in self.players.iter().enumerate() {
    //            if !player.player_choices.contains_key("Fold") {
    //                winners.push((index as u32, best_hand));
    //            }
    //        }
    //        return winners;
    //    }
    //
    //    for (index, player) in self.players.iter().enumerate() {
    //        if !player.player_choices.contains_key("Fold") {
    //            let best_hand_from_player = get_best_from_7_card_hand(&player.player_hand);
    //
    //            if best_hand_from_player > best_hand {
    //                winners.clear();
    //                winners.push((index as u32, best_hand_from_player));
    //                best_hand = best_hand_from_player;
    //            } else if best_hand_from_player == best_hand {
    //                winners.push((index as u32, best_hand_from_player));
    //            }
    //        }
    //    }
    //    winners
    //}
    //
    //fn determine_winner_texas(&self) -> Vec<(u32, ScoredHand)> {
    //    let mut winners: Vec<(u32, ScoredHand)> = Vec::new();
    //    let mut best_hand = ScoredHand::HighCard([Value::Two; 5]);
    //
    //    if self.get_remaining_player_count() <= 1 {
    //        for (index, player) in self.players.iter().enumerate() {
    //            if !player.player_choices.contains_key("Fold") {
    //                winners.push((index as u32, best_hand));
    //            }
    //        }
    //        return winners;
    //    }
    //
    //    for (index, player) in self.players.iter().enumerate() {
    //        if !player.player_choices.contains_key("Fold") {
    //            let mut total_hand = Hand::new(7);
    //
    //            for card in self.community_cards.cards.clone() {
    //                total_hand.cheat(card);
    //            }
    //
    //            for card in player.player_hand.cards.clone() {
    //                total_hand.cheat(card);
    //            }
    //
    //            let best_hand_from_player = get_best_from_7_card_hand(&total_hand);
    //
    //            if best_hand_from_player > best_hand {
    //                winners.clear();
    //                winners.push((index as u32, best_hand_from_player));
    //                best_hand = best_hand_from_player;
    //            } else if best_hand_from_player == best_hand {
    //                winners.push((index as u32, best_hand_from_player));
    //            }
    //        }
    //    }
    //    winners
    //}

    pub async fn pay_winners(&mut self) {
        let winner_count = self.winner.len();

        if winner_count == 0 {
            println!("No winners found.");
            return;
        }

        let pot_share = self.pot / winner_count as u32;

        for (winner, _) in self.winner.clone() {
            for player in self.players.iter_mut() {
                if player.player_name == winner.player_name {
                    player.player_money += pot_share;
                    println!("{} won {} chips!", player.player_name, pot_share);
                }
            }
        }
        self.pot = 0;
    }
    //****************************************************************
    // DB FNS
    //****************************************************************

    pub async fn write_results_to_db(&self) {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();
        let _result = db_client.insert(self).await;

        for player in self.players.clone() {
            let _ = db_client.update_one(&player).await;
        }
    }

    //****************************************************************
    // MISC HELPER FUNCTIONS
    //****************************************************************
    pub fn get_game_state(&self) -> &Self {
        self
    }

    async fn find_player_by_name(&mut self) -> Option<&mut Player> {
        self.players
            .iter_mut()
            .find(|p| p.player_name == self.current_player.player_name)
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
            "deck": bson::to_bson(&self.deck).map_err(|e| e.to_string())?,
            "game_id": self.game_id as i64,
            "game_variant": self.game_variant.clone(),
            "max_players": self.max_players,
            "hand_size": self.hand_size as i64,
            "pot": self.pot as i64,
            "stage_number": self.stage_number as i64,
            "round_number": self.round_number as i64,
            "minimum_bet": self.minimum_bet as i64,
            "highest_bet": self.highest_bet as i64,
            "dealer": self.dealer as i64,
            "winner": bson::to_bson(&self.winner).map_err(|e| e.to_string())?,
            "current_player": bson::to_bson(&self.current_player).map_err(|e| e.to_string())?,
            "player_action": bson::to_bson(&self.player_action).map_err(|e| e.to_string())?,
            "community_cards": bson::to_bson(&self.community_cards).map_err(|e| e.to_string())?,
            "discard_cards_prompted": self.discard_cards_prompted,
            "current_action_string": self.current_action_string.clone(),
            "action_history": bson::to_bson(&self.action_history).map_err(|e| e.to_string())?,
            "raise_min_max": bson::to_bson(&self.raise_min_max).map_err(|e| e.to_string())?,
            "pending_actions": "",
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
            current_player: bson::from_bson(
                doc.get("current_player")
                    .cloned()
                    .unwrap_or(bson::Bson::Null),
            )
            .map_err(|e| e.to_string())?,
            player_action: bson::from_bson(
                doc.get("betting_actions")
                    .cloned()
                    .unwrap_or(bson::Bson::Null),
            )
            .map_err(|e| e.to_string())?,
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
            pending_actions: HashMap::new(),
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
