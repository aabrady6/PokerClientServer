use crate::db::dbclient::DbClient;
use crate::game::base_rules::GameState;
use crate::game::player::Player;
use prettytable::{row, Table};
use std::io;

/// Displays a welcome screen and handles user input for logging in or creating a new user.
///
/// Inputs:
/// - `db_client`: A reference to the database client.
///
/// Outputs:
/// - Returns a `Player` instance upon successful login or creation.
pub async fn welcome(db_client: &DbClient) -> Player {
    println!("\n\n\n                     Welcome to\n                   ");
    println!(".------..------..------..------..------..------..------.");
    println!("|B.--. ||A.--. ||L.--. ||A.--. ||T.--. ||R.--. ||O.--. |");
    println!("| :(): || (\\/) || :/\\: || (\\/) || :/\\: || :(): || :/\\: |");
    println!("| ()() || :\\/: || (__) || :\\/: || (__) || ()() || :\\/: |");
    println!("| '--'B|| '--'A|| '--'L|| '--'A|| '--'T|| '--'R|| '--'O|");
    println!("`------'`------'`------'`------'`------'`------'`------'\n\n\n");
    println!("House Rules:");
    println!("* Players have inifinite money, if they run out, refill to $1000");
    println!("* After the initial betting round, the player to the left of the small blind will begin the betting round");
    println!("* Potentially others... House rules are House rules.\n");

    println!("Choose an option to play!");
    loop {
        println!("1. Login with username");
        println!("2. Create a new user");
        let mut user_buf = String::new();
        let _ = io::stdin().read_line(&mut user_buf);
        let user_input = user_buf.trim();

        match user_input {
            "1" => match login(db_client).await {
                Ok(Some(player)) => return Ok::<Player, String>(player).unwrap(),
                Ok(None) => println!("Player not found, please try again."),
                Err(e) => println!("Error: {}", e),
            },
            "2" => match create_user(db_client).await {
                Ok(Some(player)) => return Ok::<Player, String>(player).unwrap(),
                Ok(None) => println!("Player not found, please try again."),
                Err(e) => println!("Error: {}", e),
            },
            _ => {
                println!("\nEnter a valid input\n");
                continue;
            }
        };
    }
}

/// Logs the user in by checking their username in the database.
///
/// Inputs:
/// - `db_client`: A reference to the database client.
///
/// Outputs:
/// - `Ok(Some(Player))` if login is successful.
/// - `Ok(None)` if the user is not found.
/// - `Err(String)` if an error occurs.
pub async fn login(db_client: &DbClient) -> Result<Option<Player>, String> {
    println!("\nEnter username (example: Mr.Monopoly):");
    let mut user_buf = String::new();
    let _ = io::stdin().read_line(&mut user_buf);
    let user = user_buf.trim();

    let query_player = Player::new(&user.to_string());

    match db_client.query_one(&query_player).await {
        Ok(Some(player)) => {
            println!("{} has logged in!", player.player_name);
            Ok(Some(player))
        }
        Ok(None) => {
            println!("\nPlayer not found.");
            Ok(None)
        }
        Err(e) => {
            println!("Error retrieving player: {}", e);
            Err(e)
        }
    }
}

/// Creates a new user by checking if the username exists, then inserting it into the database.
///
/// Inputs:
/// - `db_client`: A reference to the database client.
///
/// Outputs:
/// - `Ok(Some(Player))` if the user is created successfully.
/// - `Err(String)` if an error occurs.
pub async fn create_user(db_client: &DbClient) -> Result<Option<Player>, String> {
    loop {
        println!("\n Creating a New User");
        println!("Enter a username:");
        let mut user_buf = String::new();
        let _ = io::stdin().read_line(&mut user_buf);
        let user = user_buf.trim();

        let query_player = Player::new(&user.to_string());

        match db_client.query_one(&query_player).await {
            Ok(Some(player)) => {
                println!("{} already exists, use a unique name!", player.player_name);
            }
            Ok(None) => match db_client.insert(&query_player).await {
                Ok(Some(mut player)) => {
                    let new_id = db_client.get_new_player_id().await;
                    player.set_id(new_id);
                    db_client.update_one(&player).await.unwrap();
                    println!("{} created!", user);
                    return Ok(Some(player));
                }
                Ok(None) => {
                    return Err("Unknown error during player creation.".to_string());
                }
                Err(e) => {
                    println!("Error inserting player: {}", e);
                    return Err(e);
                }
            },
            Err(e) => {
                println!("Error creating player: {}", e);
                return Err(e);
            }
        }
    }
}

/// Displays the main menu options for the game.
///
/// Outputs:
/// - Returns a `String` representing the user's menu selection.
pub async fn main_menu() -> String {
    println!("\n\n");
    println!(".------..------..------..------.");
    println!("|M.--. ||E.--. ||N.--. ||U.--. |");
    println!("| (\\/) || (\\/) || :(): || (\\/) |");
    println!("| :\\/: || :\\/: || ()() || :\\/: |");
    println!("| '--'M|| '--'E|| '--'N|| '--'U|");
    println!("`------'`------'`------'`------'\n");
    println!("Menu Options: Enter an Option:");
    loop {
        println!("1. Five Card Draw");
        println!("2. Seven Card Stud");
        println!("3. Texas Hold'em");
        println!("4. Player Stats");
        println!("5. Previous Game Stats");
        println!("6. Reset All Stats");
        println!("7. Exit the Game");
        let mut user_buf = String::new();
        let _ = io::stdin().read_line(&mut user_buf);
        let user_input = user_buf.trim();

        match user_input {
            "1" => return "five-card-draw".to_string(),
            "2" => return "seven-card-stud".to_string(),
            "3" => return "texas-hold-em".to_string(),
            "4" => return "player-stats".to_string(),
            "5" => return "game-stats".to_string(),
            "6" => return "reset-stats".to_string(),
            "7" => return "exit".to_string(),
            _ => {
                println!("\nEnter a valid input\n");
                continue;
            }
        };
    }
}

/// Prints a table of player statistics.
///
/// Inputs:
/// - `players`: A vector containing player data.
pub fn print_player_table(players: Vec<Player>) {
    let mut table = Table::new();

    table.add_row(row![
        "PlayerID",
        "Username",
        "Money",
        "Hands Played",
        "Win %",
        "Wins",
        "Losses",
        "Total Earnings"
    ]);

    for player in players {
        let win_percent = if player.get_games() == 0 {
            0.0
        } else {
            (player.get_wins() as f64 / player.get_games() as f64) * 100.0
        };

        let win_percent_formatted = format!("{:.2}", win_percent);

        table.add_row(row![
            player.get_id(),
            player.player_name,
            player.player_money,
            player.get_games(),
            win_percent_formatted,
            player.get_wins(),
            player.get_losses(),
            player.get_earnings(),
        ]);
    }
    table.printstd();
}

/// Displays the player stats menu and allows the user to search for specific player statistics.
///
/// Inputs:
/// - `db_client`: A reference to the database client.
pub async fn player_stats_menu(db_client: &DbClient) {
    println!("\n\n");
    println!(".------..------..------..------..------..------..------.");
    println!("|P.--. ||L.--. ||A.--. ||Y.--. ||E.--. ||R.--. ||S.--. |");
    println!("| :/\\: || :/\\: || (\\/) || (\\/) || (\\/) || :(): || :/\\: |");
    println!("| (__) || (__) || :\\/: || :\\/: || :\\/: || ()() || :\\/: |");
    println!("| '--'P|| '--'L|| '--'A|| '--'Y|| '--'E|| '--'R|| '--'S|");
    println!("`------'`------'`------'`------'`------'`------'`------'\n");
    println!("Player Stats:");

    let result = db_client.query_all::<Player>().await;

    match result {
        Ok(players) => {
            if players.is_empty() {
                println!("No players found in the database.");
            } else {
                print_player_table(players);
            }
        }
        Err(e) => {
            println!("Error retrieving players: {}", e);
        }
    }

    println!("User Stats Options: Enter a number:");
    loop {
        println!("1. Search for a Player by Username:");
        println!("2. Return to Main Menu");

        let mut user_buf = String::new();
        let _ = io::stdin().read_line(&mut user_buf);
        let user_input = user_buf.trim();

        match user_input {
            "1" => {
                println!("Enter the username you want to see the stats for:");
                let mut user_buf = String::new();
                let _ = io::stdin().read_line(&mut user_buf);
                let user_search_input = user_buf.trim();

                let user_search_player = Player::new(user_search_input);

                let result = db_client.query_one(&user_search_player).await;

                match result {
                    Ok(Some(retrieved_player)) => {
                        let mut searched_player: Vec<Player> = Vec::new();
                        searched_player.push(retrieved_player);
                        print_player_table(searched_player);
                    }
                    Ok(None) => {
                        println!("\nPlayer not found.");
                    }
                    Err(e) => {
                        println!("Error retrieving player: {}", e);
                    }
                }
            }
            "2" => break,
            _ => {
                println!("\nEnter a valid input\n");
                continue;
            }
        };
    }
}

/// Formats the winner name for display.
///
/// Inputs:
/// - `winners`: A vector of `Player` instances representing the winners.
///
/// Outputs:
/// - A `String` representation of the winner(s).
fn format_winners(winners: &Vec<Player>) -> String {
    if winners.is_empty() {
        return "No Winner".to_string();
    }

    if winners.len() > 1 {
        "Push".to_string()
    } else {
        winners[0].get_name().to_string()
    }
}

/// Prints a general table of game statistics.
///
/// Inputs:
/// - `games`: A vector containing `Game_State` instances.
pub fn print_general_games_table(games: Vec<GameState>) {
    let mut table = Table::new();

    table.add_row(row![
        "GameID",
        "Game Variant",
        "Players",
        "Winner",
        "Dealer",
    ]);

    for game in games {
        table.add_row(row![
            game.get_game_id(),
            game.get_variant(),
            game.players().len(),
            format_winners(&game.winner),
            game.dealer,
        ]);
    }
    table.printstd();
}

/// Prints a specific table of game statistics, showing detailed information.
///
/// Inputs:
/// - `games`: A vector containing `Game_State` instances.
pub fn print_specific_game_table(games: Vec<GameState>) {
    let mut table = Table::new();

    table.add_row(row![
        "GameID",
        "Game Variant",
        "Players",
        "Winner",
        "Dealer Seat",
    ]);

    for game in games {
        table.add_row(row![
            game.get_game_id(),
            game.get_variant(),
            game.players().len(),
            format_winners(&game.winner),
            game.dealer,
        ]);

        table.add_row(row!["~~", "~~", "~Players~", "~~", "~~",]);
        table.add_row(row![
            "Player ID",
            "Player Name",
            "Hand",
            "Total Wagered",
            "Chips Won",
        ]);
        for player in game.players {
            table.add_row(row![
                player.get_id(),
                player.get_name(),
                player.player_hand,
                player.total_wagered_per_game,
                player.round_win,
            ]);
        }
    }
    table.printstd();
}

/// Displays the game stats menu and allows the user to search for specific game statistics.
///
/// Inputs:
/// - `db_client`: A reference to the database client.
pub async fn game_stats_menu(db_client: &DbClient) {
    println!("\n\n");
    println!(".------..------..------..------..------.");
    println!("|G.--. ||A.--. ||M.--. ||E.--. ||S.--. |");
    println!("| :/\\: || (\\/) || (\\/) || (\\/) || :/\\: |");
    println!("| :\\/: || :\\/: || :\\/: || :\\/: || :\\/: |");
    println!("| '--'G|| '--'A|| '--'M|| '--'E|| '--'S|");
    println!("`------'`------'`------'`------'`------'\n");
    println!("Game Stats:");

    let result = db_client.query_all::<GameState>().await;

    match result {
        Ok(games) => {
            if games.is_empty() {
                println!("No games found in the database.");
            } else {
                print_general_games_table(games);
            }
        }
        Err(e) => {
            println!("Error retrieving games: {}", e);
        }
    }

    println!("User Stats Options: Enter a number:");
    loop {
        println!("1. Search for a Game by Game ID:");
        println!("2. Return to Main Menu");

        let mut user_buf = String::new();
        let _ = io::stdin().read_line(&mut user_buf);
        let user_input = user_buf.trim();

        match user_input {
            "1" => {
                println!("Enter the Game ID you want to see the stats for:");
                let mut user_buf = String::new();
                let _ = io::stdin().read_line(&mut user_buf);
                let user_search_input = user_buf.trim();

                let mut search_id = 0;
                match user_search_input.parse::<i64>() {
                    Ok(game_id) => search_id = game_id,
                    Err(_) => {
                        println!("Invalid input. Please enter a valid number.");
                    }
                }

                let user_search_game = GameState::empty_constructor(search_id as u32);

                let result = db_client.query_one(&user_search_game).await;

                match result {
                    Ok(Some(retrieved_game)) => {
                        let mut searched_game: Vec<GameState> = Vec::new();
                        searched_game.push(retrieved_game);
                        print_specific_game_table(searched_game);
                    }
                    Ok(None) => {
                        println!("\nGame not found.");
                    }
                    Err(e) => {
                        println!("Error retrieving Game: {}", e);
                    }
                }
            }
            "2" => break,
            _ => {
                println!("\nEnter a valid input\n");
                continue;
            }
        };
    }
}

/// Resets all game statistics, removing all stored game and player data.
///
/// Inputs:
/// - `db_client`: A reference to the database client.
/// - `player`: A mutable reference to the current player.
pub async fn reset_stats_menu(db_client: &DbClient, player: &mut Player) {
    println!("\n\n");
    println!(".------..------..------..------..------..------..------..------..------..------.");
    println!("|R.--. ||E.--. ||S.--. ||E.--. ||T.--. ||S.--. ||T.--. ||A.--. ||T.--. ||S.--. |");
    println!(
        "| :(): || (\\/) || :/\\: || (\\/) || :/\\: || :/\\: || :/\\: || (\\/) || :/\\: || :/\\: |"
    );
    println!(
        "| ()() || :\\/: || :\\/: || :\\/: || (__) || :\\/: || (__) || :\\/: || (__) || :\\/: |"
    );
    println!("| '--'R|| '--'E|| '--'S|| '--'E|| '--'T|| '--'S|| '--'T|| '--'A|| '--'T|| '--'S|");
    println!("`------'`------'`------'`------'`------'`------'`------'`------'`------'`------'\n");
    println!("Reset Game Stats:");
    println!("\n\nAre you sure you want to reset all stats? Will remove all games and reset player stats to 0...");
    loop {
        println!("1. Confirm the Reset... There is no going back...");
        println!("2. Return to Main Menu");

        let mut user_buf = String::new();
        let _ = io::stdin().read_line(&mut user_buf);
        let user_input = user_buf.trim();

        match user_input {
            "1" => {
                let game = db_client.reset_game_stats().await;
                match game {
                    Ok(_) => println!("Game stats are reset"),
                    Err(e) => println!("Error Deleting Games: {}", e),
                }

                let players = db_client.reset_player_stats().await;
                match players {
                    Ok(_) => println!("Player stats are reset"),
                    Err(e) => println!("Error Deleting Players: {}", e),
                }

                player.set_earnings(0);
                player.set_games(0);
                player.set_wins(0);
                player.set_losses(0);
                break;
            }
            "2" => break,
            _ => {
                println!("\nEnter a valid input\n");
                continue;
            }
        };
    }
}
