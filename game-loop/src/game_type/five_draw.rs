// $2/$5 5 Card Draw RULES:
//
//  PLAYERS:
//      2-5
//
//  BLINDS:
//     Small Blind - Left of dealers
//     Big Blind - Left of small blind
//
//  DEALING:
//      Each player is dealt 5 cards on dealers left
//
//  DISCARD:
//      Each player can discard up to 5 cards and redraw
//
//  GAME LOOP:
//      big and small blind put into pot
//      deal - 5 face down cards
//      betting round - left of big blind starts betting
//          initial min bet to call (big blind) $5
//      discard round
//      betting round
//      showdown
//      DEALER BUTTON MOVES LEFT 1 SPOT
//
//  HOUSE RULES:
//      None

use crate::db::dbclient::DbClient;
use crate::game::base_rules::GameState;
use crate::game::base_rules::LoadState;
use crate::game::base_rules::TestInput;
use crate::game::card::Value;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::poker_game::PokerGame;
use crate::game::score_hands::get_best_5_card_hand;
use crate::game::score_hands::ScoredHand;
use std::collections::HashSet;
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct FiveCardDraw {
    /// Holding the game
    pub game_state: GameState,
}

impl PokerGame for FiveCardDraw {
    fn new(_ls: LoadState) -> Self {
        let mut game_state = GameState::new_game(0);

        game_state.game_variant = "5 Card Draw".to_string();
        game_state.hand_size = 5;
        game_state.max_players = 5;
        game_state.minimum_bet = 5;

        FiveCardDraw { game_state }
    }

    async fn new_round(&mut self) {
        let uri = "mongodb://localhost:27017";
        let db_client = DbClient::new(uri).await.unwrap();
        let game_id = db_client.get_new_game_id().await;

        self.game_state.game_id = game_id;
        self.game_state.deck = Deck::new();
        self.game_state.pot = 0;
        self.game_state.stage_number = 1;
        self.game_state.round_number = 0;
        self.game_state.highest_bet = 0;
        self.game_state.winner = vec![];
        self.game_state.rotate_blinds();

        for player in &mut self.game_state.players {
            player.reset_choices();
            player.total_wagered_per_game = 0;
            player.round_win = 0;
            player.player_hand = Hand::new(5);
            player.infinite_money();
        }
    }

    async fn game_run_through(&mut self) {
        loop {
            self.new_round().await;

            self.print_roles();
            self.game_state.deal_hands();

            // blinds are taken care of in the first betting round
            let start_index = self.get_starting_player_index();
            self.betting_round(start_index, self.game_state.minimum_bet)
                .await;

            self.player_redraw(&mut TestInput::Empty).await;
            self.betting_round(0, self.game_state.minimum_bet).await;

            let winners = self.determine_winner();
            self.pay_out(winners);
            self.save_to_db().await;

            if !self.post_round_menu() {
                break;
            }
        }
    }

    async fn betting_round(&mut self, starting_index: u32, _min_bet: u32) {
        if self.game_state.get_remaining_player_count() <= 1 {
            return;
        }

        self.game_state.new_betting_round();

        if self.game_state.round_number == 0 {
            self.first_betting_round();
        }

        self.game_state
            .betting_loop(starting_index, &mut TestInput::Empty, None)
            .await;
    }

    fn determine_winner(&self) -> Vec<(u32, ScoredHand)> {
        let mut winners: Vec<(u32, ScoredHand)> = Vec::new();
        let mut best_hand = ScoredHand::HighCard([Value::Two; 5]);

        if self.game_state.get_remaining_player_count() <= 1 {
            for (index, player) in self.game_state.players.iter().enumerate() {
                if !player.player_choices.contains_key("Fold") {
                    winners.push((index as u32, best_hand));
                }
            }
            return winners;
        }

        for (index, player) in self.game_state.players.iter().enumerate() {
            if !player.player_choices.contains_key("Fold") {
                let best_hand_from_player = get_best_5_card_hand(&player.player_hand);

                if best_hand_from_player > best_hand {
                    winners.clear();
                    winners.push((index as u32, best_hand_from_player));
                    best_hand = best_hand_from_player;
                } else if best_hand_from_player == best_hand {
                    winners.push((index as u32, best_hand_from_player));
                }
            }
        }

        winners
    }

    fn pay_out(&mut self, winners: Vec<(u32, ScoredHand)>) {
        self.game_state.pay_out(winners);
    }

    async fn save_to_db(&self) {
        self.game_state.write_results_to_db().await;
    }

    fn post_round_menu(&self) -> bool {
        self.game_state.post_round_menu()
    }
}

impl FiveCardDraw {
    pub fn deal_initial_hands(&mut self) {
        if self.game_state.get_remaining_player_count() <= 1 {
            return;
        }

        for _ in 0..5 {
            for player in &mut self.game_state.players {
                if !player.player_choices.contains_key("Fold") {
                    let _ = player.player_hand.draw(&mut self.game_state.deck, 1);
                }
            }
        }
    }

    pub fn first_betting_round(&mut self) {
        self.game_state.put_blinds();
    }

    pub fn get_starting_player_index(&self) -> u32 {
        (self.game_state.dealer + 3) % self.game_state.players.len() as u32
    }

    pub fn print_roles(&self) {
        println!("Roles: ");
        for i in 0..3 {
            if i == 0 {
                let index =
                    ((self.game_state.dealer) % self.game_state.players.len() as u32) as usize;
                println!("Dealer: {}", self.game_state.players[index].get_name(),)
            }
            if i == 1 {
                let index =
                    ((self.game_state.dealer + 1) % self.game_state.players.len() as u32) as usize;
                println!("Small Blind: {}", self.game_state.players[index].get_name(),)
            }
            if i == 2 {
                let index =
                    ((self.game_state.dealer + 2) % self.game_state.players.len() as u32) as usize;
                println!("Big Blind: {}\n", self.game_state.players[index].get_name(),)
            }
        }
    }

    /// Handles the player redraw phase, allowing players to discard cards and draw new ones.
    /// This function is run asynchronously and involves multiple steps:
    /// 1. Displaying the player's hand and asking them which cards they want to discard.
    /// 2. Ensuring the input is valid (only discards between 1 and 5 are allowed, no duplicates).
    /// 3. Discarding the specified cards and adjusting indices for proper handling of the deck.
    /// 4. Drawing new cards to replace the discarded ones.
    ///
    /// Example:
    /// ```rust
    /// game.player_redraw().await;
    /// ```
    async fn player_redraw(&mut self, test_input: &mut TestInput) {
        if self.game_state.get_remaining_player_count() <= 1 {
            return;
        }

        let mut players_remaining = 0;
        for player in &self.game_state.players {
            if !player.player_choices.contains_key("Fold") {
                players_remaining += 1;
            }
        }
        for index in 0..self.game_state.players.len() {
            let player_search = (self.game_state.dealer + index as u32 + 1)
                % (self.game_state.players.len() as u32);
            if players_remaining == 0 {
                break;
            }
            let player = &mut self.game_state.players[player_search as usize];

            // for player in &mut self.game_state.players {
            if !player.player_choices.contains_key("Fold") {
                players_remaining -= 0;
                println!("Player {}; Hand: {}", player.get_name(), player.player_hand);
                println!("What cards would you like to discard?");
                println!("Enter the cards you want to discard like 1, 2, 4, 5 or 2,3,4, enter 0 if no discards are wanted");
                loop {
                    let hand_vec = player.player_hand.get_hand_cards().clone();
                    for index in 0..5 {
                        println!("{}: {}", index + 1, hand_vec[index]); // Show card options (index + 1 for human-friendly input)
                    }
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

                    let mut discard_indices: Vec<u8> = input
                        .trim()
                        .split(',')
                        .filter_map(|s| s.trim().parse::<u8>().ok())
                        .collect();

                    // Handle cases
                    if discard_indices.is_empty() {
                        println!("Please enter a value.");
                    } else if discard_indices.contains(&0) {
                        // If 0 is entered, check if it's the only value.
                        if discard_indices.len() == 1 {
                            println!("No cards will be discarded.");
                            break; // Exit loop after handling '0' input alone
                        } else {
                            println!("You cannot enter other numbers with 0. Please try again.");
                        }
                    } else {
                        if discard_indices.len() > 5 {
                            println!("Please enter up to five values.");
                        } else if discard_indices.iter().any(|&x| x < 1 || x > 5) {
                            println!("Please enter values between 1 and 5.");
                        } else if discard_indices.iter().collect::<HashSet<_>>().len()
                            != discard_indices.len()
                        {
                            // Check for duplicates
                            println!("Numbers must not be repeated.");
                        } else {
                            // Valid input, process the discard
                            discard_indices.sort();
                            println!("Sorted Indicies: {:?}", discard_indices);
                            // Discard the cards and adjust indices after each discard
                            let mut adjusted_indices = Vec::new();
                            let mut handled_index = 1;
                            for index in &discard_indices.clone() {
                                let adjusted_index = index - handled_index; // Convert to zero-based index
                                if let Err(e) = player.player_hand.discard(adjusted_index as u8) {
                                    println!("Error discarding card {}: {}", index, e);
                                } else {
                                    adjusted_indices.push(adjusted_index);
                                    // After discarding, all indices in the vector need to be adjusted down
                                    discard_indices = discard_indices
                                        .into_iter()
                                        .filter(|&x| x > index.clone())
                                        .map(|x| x - 1)
                                        .collect();
                                }
                                handled_index += 1;
                            }
                            // Now draw new cards for the player
                            for _ in 0..handled_index - 1 {
                                // let _ = player.draw_card(&mut self.game_state.deck);
                                if let Err(e) = player.draw_card(&mut self.game_state.deck) {
                                    println!("Error drawing cards: {}", e);
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

////#[cfg(test)]
////mod tests {
////    use super::*;
////    use crate::game::hand::Hand;
////
////    #[tokio::test]
////    async fn test_betting_loop_pre_draw() {
////        let mut loading_state = GameState::empty_constructor(0);
////        let mut player1 = Player::new_player(5 as u8, "P1".to_string(), 1);
////        let mut player2 = Player::new_player(5 as u8, "P2".to_string(), 2);
////        let mut player3 = Player::new_player(5 as u8, "P3".to_string(), 3);
////        let mut player4 = Player::new_player(5 as u8, "P5".to_string(), 4);
////
////        player1.set_money(100);
////        player2.set_money(100);
////        player3.set_money(100);
////        player4.set_money(100);
////
////        loading_state.insert_player(&player1).unwrap();
////        loading_state.insert_player(&player2).unwrap();
////        loading_state.insert_player(&player3).unwrap();
////        loading_state.insert_player(&player4).unwrap();
////
////        let mut game = FiveDraw::new(1, LoadState::GameExist(loading_state));
////        game.game_state.put_blinds();
////        game.game_state.deal_hands();
////
////        let mut simulated_input = TestInput::Input(vec![
////            "1".to_string(),
////            "1".to_string(),
////            "1".to_string(),
////            "1".to_string(),
////        ]);
////        // Simulate pre-draw betting loop
////        game.pre_draw(&mut simulated_input).await;
////        for player in &game.game_state.players {
////            println!("Player {}, Money {}", player.get_name(), player.get_money());
////            player.print_enums();
////        }
////        assert_eq!(
////            game.game_state.players[0]
////                .player_choices
////                .contains_key("Fold"),
////            true
////        );
////        assert_eq!(
////            game.game_state.players[1]
////                .player_choices
////                .contains_key("Fold"),
////            true
////        );
////        assert_eq!(*game.game_state.players[2].get_money(), 105 as u32);
////        assert_eq!(
////            game.game_state.players[3]
////                .player_choices
////                .contains_key("Fold"),
////            true
////        );
////        // assert!(result); // Assuming the loop returns true when betting completes successfully
////    }
////
////    // Test that the post-draw phase works with card discards and redraws
////    #[tokio::test]
////    async fn test_betting_loop_post_draw() {
////        let mut loading_state = GameState::empty_constructor(0);
////        let mut player1 = Player::new_player(5 as u8, "P1".to_string(), 1);
////        let mut player2 = Player::new_player(5 as u8, "P2".to_string(), 2);
////        let mut player3 = Player::new_player(5 as u8, "P3".to_string(), 3);
////        let mut player4 = Player::new_player(5 as u8, "P5".to_string(), 4);
////
////        player1.set_money(100);
////        player2.set_money(100);
////        player3.set_money(100);
////        player4.set_money(100);
////
////        loading_state.insert_player(&player1).unwrap();
////        loading_state.insert_player(&player2).unwrap();
////        loading_state.insert_player(&player3).unwrap();
////        loading_state.insert_player(&player4).unwrap();
////
////        let mut game = FiveDraw::new(1, LoadState::GameExist(loading_state));
////        game.game_state.put_blinds();
////        game.game_state.deal_hands();
////        println!("Deck size {}", game.game_state.deck().clone().count());
////
////        let mut simulated_input = TestInput::Input(vec![
////            "1, 2, 3".to_string(),
////            "1, 2".to_string(),
////            "0".to_string(),
////            "2,3".to_string(),
////            "2".to_string(),
////            "2".to_string(),
////            "3".to_string(),
////            "20".to_string(),
////            "1".to_string(),
////            "2".to_string(),
////            "2".to_string(),
////            "2".to_string(),
////        ]);
////        for player in &game.game_state.players {
////            println!(
////                "Player {}, Money {}, Hand {}",
////                player.get_name(),
////                player.get_money(),
////                player.player_hand
////            );
////            player.print_enums();
////        }
////        // Simulate pre-draw betting loop
////        game.post_draw(&mut simulated_input).await;
////        for player in &game.game_state.players {
////            println!(
////                "Player {}, Money {}, Hand {}",
////                player.get_name(),
////                player.get_money(),
////                player.player_hand
////            );
////            player.print_enums();
////        }
////        assert_eq!(
////            game.game_state.players[0]
////                .player_choices
////                .contains_key("Call"),
////            true
////        );
////        assert_eq!(*game.game_state.players[0].get_money(), 80 as u32);
////        assert_eq!(
////            game.game_state.players[1]
////                .player_choices
////                .contains_key("Raise"),
////            true
////        );
////        assert_eq!(*game.game_state.players[1].get_money(), 75 as u32);
////        assert_eq!(
////            game.game_state.players[2]
////                .player_choices
////                .contains_key("Fold"),
////            true
////        );
////        assert_eq!(*game.game_state.players[2].get_money(), 90 as u32);
////        assert_eq!(
////            game.game_state.players[3]
////                .player_choices
////                .contains_key("Call"),
////            true
////        );
////        assert_eq!(*game.game_state.players[3].get_money(), 80 as u32);
////    }
////
////    #[tokio::test]
////    async fn test_redraw() {
////        let mut loading_state = GameState::empty_constructor(0);
////        let mut player1 = Player::new_player(5 as u8, "P1".to_string(), 1);
////        let mut player2 = Player::new_player(5 as u8, "P2".to_string(), 2);
////        let mut player3 = Player::new_player(5 as u8, "P3".to_string(), 3);
////        let mut player4 = Player::new_player(5 as u8, "P4".to_string(), 4);
////
////        player1.set_money(100);
////        player2.set_money(100);
////        player3.set_money(100);
////        player4.set_money(100);
////
////        loading_state.insert_player(&player1).unwrap();
////        loading_state.insert_player(&player2).unwrap();
////        loading_state.insert_player(&player3).unwrap();
////        loading_state.insert_player(&player4).unwrap();
////
////        let mut game = FiveDraw::new(1, LoadState::GameExist(loading_state));
////        game.game_state.put_blinds();
////        game.game_state.deal_hands();
////        let mut pre_discard_hands: Vec<Hand> = Vec::new();
////        for player in &game.game_state.players {
////            pre_discard_hands.push(player.player_hand.clone());
////        }
////        let mut simulated_input = TestInput::Input(vec![
////            "1, 2, 3".to_string(),
////            "1".to_string(),
////            "0".to_string(),
////            "10".to_string(),
////            "-1".to_string(),
////            "1, 2, 3, 4, 5, 6".to_string(),
////            "2".to_string(),
////        ]);
////
////        game.player_redraw(&mut simulated_input).await;
////        // for player in &game.game_state.players{
////        //     println!("Player {}, Money {}, Hand {}", player.get_name(), player.get_money(), player.player_hand);
////        //     player.print_enums();
////        // }
////        assert_ne!(game.game_state.players[0].player_hand, pre_discard_hands[0]);
////
////        assert_ne!(game.game_state.players[1].player_hand, pre_discard_hands[1]);
////
////        assert_ne!(game.game_state.players[2].player_hand, pre_discard_hands[2]);
////
////        assert_eq!(game.game_state.players[3].player_hand, pre_discard_hands[3]);
////    }
////}
