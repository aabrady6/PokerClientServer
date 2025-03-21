// $5/$10 7 Card Stud RULES:
//
//  PLAYERS:
//      2-7
//
//  ANTES:
//      Before dealing all players post an ante
//      $2
//
//  DEALING:
//      Each player is dealt 3 cards on dealers left in this order:
//          2 face down(hole cards)
//          1 face up(door card)
//
//  GAME LOOP:
//      players must ante - $2
//      deal - 2 face down, 1 face up (3 cards)
//      betting round - low bet (min $5)
//          lowest face up card starts round
//          initial min bet to call (bring in bet) $5
//      deal - 1 face up (4 cards: 2 down, 2 up)
//      betting round low bet (min $5)
//          best hand showing starts round (highest face up card)
//      deal - 1 face up (5 cards: 2 down, 3 up)
//      betting round high bet (min $10)
//          best hand showing starts roung
//      deal - 1 face up (6 cards: 2 down, 4 up)
//      betting round high bet (min $10)
//          best hand showing starts round
//      deal - 1 face down (7 cards: 3 down, 4 up)
//      betting round high bet (min $10)
//      DEALER BUTTON MOVES LEFT 1 SPOT
//
//  HOUSE RULES:
//      Lowest and Highest value for bettings rounds is based off the single value card,
//          i.e. P1 = {QS QH}, P2 = {KH, 2S} - P2 starts
//
use crate::db::dbclient::DbClient;
use crate::game::base_rules::GameState;
use crate::game::base_rules::LoadState;
use crate::game::base_rules::TestInput;
use crate::game::card::Value;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::poker_game::PokerGame;
use crate::game::score_hands::get_best_from_7_card_hand;
use crate::game::score_hands::ScoredHand;

#[derive(Debug)]
pub struct SevenCardStud {
    pub game_state: GameState,
    pub lower_bet: u32,
    pub higher_bet: u32,
}

impl PokerGame for SevenCardStud {
    fn new(_ls: LoadState) -> Self {
        let mut game_state = GameState::new_game(0);

        game_state.game_variant = "7 Card Stud".to_string();
        game_state.hand_size = 7;
        game_state.max_players = 7;
        game_state.minimum_bet = 5;

        let lower = 5;
        let higher = 10;
        SevenCardStud {
            game_state,
            lower_bet: lower,
            higher_bet: higher,
        }
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
            player.player_hand = Hand::new(7);
            player.infinite_money();
        }
    }

    async fn game_run_through(&mut self) {
        let mut starting_index;
        loop {
            self.new_round().await;

            // initial antes are taken care of in the first betting round logic

            self.deal_face_down(2);
            self.deal_face_up(1);

            starting_index = self.determine_lowest_door();
            self.betting_round(starting_index, self.lower_bet).await;

            self.deal_face_up(1);
            starting_index = self.determine_highest_door();
            self.betting_round(starting_index, self.higher_bet).await;

            self.deal_face_up(1);
            starting_index = self.determine_highest_door();
            self.betting_round(starting_index, self.higher_bet).await;

            self.deal_face_up(1);
            starting_index = self.determine_highest_door();
            self.betting_round(starting_index, self.higher_bet).await;

            self.deal_face_down(1);
            starting_index = self.determine_highest_door();
            self.betting_round(starting_index, self.higher_bet).await;

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
                let best_hand_from_player = get_best_from_7_card_hand(&player.player_hand);

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

impl SevenCardStud {
    pub fn put_antes(&mut self) {
        for player in self.game_state.players.clone() {
            self.game_state.put_into_pot(player.player_id, 2);
        }
    }

    pub fn first_betting_round(&mut self) {
        for player in &mut self.game_state.players {
            player.raise(2);
            self.game_state.pot += 2;
        }
        self.game_state.highest_bet = self.lower_bet;
    }

    pub fn deal_face_down(&mut self, num_cards: u32) {
        if self.game_state.get_remaining_player_count() <= 1 {
            return;
        }

        for _ in 0..num_cards {
            for player in &mut self.game_state.players {
                if !player.player_choices.contains_key("Fold") {
                    let _ = player.player_hand.draw(&mut self.game_state.deck, 1);
                }
            }
        }
    }

    pub fn deal_face_up(&mut self, num_cards: u32) {
        if self.game_state.get_remaining_player_count() <= 1 {
            return;
        }

        for _ in 0..num_cards {
            for player in &mut self.game_state.players {
                if !player.player_choices.contains_key("Fold") {
                    let _ = player
                        .player_hand
                        .draw_face_up(&mut self.game_state.deck, 1);
                }
            }
        }
    }

    pub fn determine_lowest_door(&self) -> u32 {
        let mut lowest = self.game_state.players[0].clone();
        let mut lowest_index = 0;

        for (index, player) in self.game_state.players.iter().enumerate() {
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

    pub fn determine_highest_door(&self) -> u32 {
        let mut highest = self.game_state.players[0].clone();
        let mut highest_index = 0;

        for (index, player) in self.game_state.players.iter().enumerate() {
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
}
