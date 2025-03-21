use crate::game::poker_game::PokerGame;
use std::io;
use std::io::Write;
/// Define a generic Server struct, which is parameterized by a type T that implements the PokerGame trait.
pub struct Server<T: PokerGame> {
    pub game_type: T,
}
/// Implement methods for the Server struct, which operates on a type that implements the PokerGame trait.
impl<T: PokerGame> Server<T> {
    /// Constructor function to initialize a new server instance with a specific Poker game type.
    pub fn new(game_type: T) -> Self {
        println!("Standard Poker initalize");
        Self { game_type }
    }

    /// The main game loop for playing the game. It repeatedly starts a new round and runs the game
    /// until the user decides to stop playing.
    pub async fn play_game(&mut self) {
        loop {
            //self.game_type.start_round();
            self.game_type.game_run_through().await;
            if Self::ask_another_round().await == 1 {
                break;
            }
            self.game_type.new_round().await;
        }
    }

    /// This function asks the user if they would like to play another round. It returns 2 if the user chooses to continue
    /// and 1 if they choose to stop playing. It validates user input and ensures they enter 'Y/y' or 'N/n'.
    pub async fn ask_another_round() -> u8 {
        let mut input = String::new();
        loop {
            println!("Would you like to stay in this game and play another round (Y/N)?");
            io::stdout().flush().unwrap();
            input.clear();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            match input.trim().parse::<char>() {
                Ok(result) => {
                    // Check if bet is within the valid range
                    if result == 'Y' || result == 'N' || result == 'y' || result == 'n' {
                        if result == 'Y' || result == 'y' {
                            return 2;
                        }
                        return 1;
                    } else {
                        println!("Result must be Y/N or y/n");
                    }
                }
                Err(_) => {
                    println!("Result must be Y/N or y/n");
                }
            }
        }
    }
}
