use crate::game::base_rules::LoadState;
use crate::game::score_hands::ScoredHand;
use std::future::Future;

/// Traits to be implimented by each of the individual game types to ensure coherent function calling.
pub trait PokerGame {
    /// Constructor to return the game type back to the caller to execute further calls
    fn new(ls: LoadState) -> Self;

    fn new_round(&mut self) -> impl Future<Output = ()> + Send;

    /// Main core of the game to call other functions that then advance the game state
    fn game_run_through(&mut self) -> impl Future<Output = ()> + Send;

    fn betting_round(
        &mut self,
        starting_index: u32,
        min_bet: u32,
    ) -> impl Future<Output = ()> + Send;

    /// Extra function to help determine who the winner is, not being used currently
    fn determine_winner(&self) -> Vec<(u32, ScoredHand)>;

    fn pay_out(&mut self, winners: Vec<(u32, ScoredHand)>);

    fn save_to_db(&self) -> impl Future<Output = ()> + Send;

    fn post_round_menu(&self) -> bool;
}
