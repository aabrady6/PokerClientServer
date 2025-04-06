use crate::game_structures::PlayerChoice;
use crate::game_structures::GameState;

/// Displays the current state of the poker game to the console.
///
/// This function prints a formatted representation of the game board, including
/// game variant, current player's turn, recent actions, pot size, community cards
/// (for Texas Holdem), and details about each player's state.
///
/// # Parameters
/// * `game_state` - The current state of the game to display
/// * `viewer_username` - Username of the player viewing the game board
/// * `is_current_player` - Boolean indicating if the viewer is the current player whose turn it is
///
/// # Output
/// Prints game information to the console:
/// * Game variant and whose turn it is
/// * Last five actions taken
/// * Current pot size
/// * Community cards (for Texas Holdem)
/// * Information about each player:
///   * Player ID and name
///   * Special roles (Dealer, Small Blind, Big Blind)
///   * Current money and amount bet
///   * Cards in hand (only showing face-up cards or all cards for the viewer)
pub fn print_game_board(game_state: GameState, viewer_username: String, is_current_player: bool) {
    // Want to print in a line the players names, 
    let mut game_info_string = "".to_string();
    // This is the bad print, but executing for now for ease of use
    match is_current_player {
        true => game_info_string = format!("{}{}, Currently Your Turn\n", game_info_string, game_state.game_variant),
        false => game_info_string = format!("{}{}, Currently {}'s Turn\n", game_info_string, game_state.game_variant, game_state.current_player.player_name),
    }
    // get the last five items in the action_history, if there isnt then collect as many as possible: 
    // Vec<String> and convert them to "1. last action, 2. second last action, 3. third last action..."
    let last_actions = game_state.action_history.iter().rev().take(5).collect::<Vec<&String>>();

    let formatted_actions = last_actions
        .into_iter()
        .enumerate()
        .map(|(i, action)| format!("{}. {}", i + 1, action))
        .collect::<Vec<String>>()
        .join(", ");

    game_info_string = format!("{}Last five moves: {}\n", game_info_string, formatted_actions);
    game_info_string = format!("{}Current Pot: {}\n", game_info_string, game_state.pot);

    let community_card_display = game_state.community_cards.cards
            .iter()
            .map(|card| card.to_string(false))
            .collect::<Vec<String>>()
            .join(", ");

    if game_state.game_variant == "Texas Holdem" {
        game_info_string = format!("{}Community Cards: {}\n", game_info_string, community_card_display);
    }

    let mut player_strings: Vec<String> = Vec::new();
    for player in game_state.players {
        let mut player_string = "".to_string();
        match player.token.as_str() {
            "D" => player_string = format!("{}Dealer Player ID: {}\nPlayer name: {}\n", player_string, player.player_id, player.player_name),
            "SB" => player_string = format!("{}Small Blind Player ID: {}\nPlayer name: {}\n", player_string, player.player_id, player.player_name),
            "BB" => player_string = format!("{}Big Blind Player ID: {}\nPlayer name: {}\n", player_string, player.player_id, player.player_name),
            _ => player_string = format!("{}Player ID: {}\nPlayer name: {}\n", player_string, player.player_id, player.player_name),
        }

        let placed_in_pot = match player.player_choices.get("PlacedInPot") {
            Some(choice) => match choice {
                PlayerChoice::PlacedInPot(value) => value.clone(),
                _ => 0,
            }
            _ => 0,
        };

        player_string = format!("{}Money: {}\nMoney Bet: {}\n", player_string, player.player_money, placed_in_pot);
        let view_all = if player.player_name == viewer_username { true } else { false };
        // Convert Player hand of cards to consistent output, collapse the vector into a string with the card values seperated by, want ## 
        let print_hand = player.player_hand.cards
            .iter()
            .map(|card| card.to_string(view_all))
            .collect::<Vec<String>>()
            .join(", ");
        player_string = format!("{}Player Hand: {}\n", player_string, print_hand);
        // println!("Player Hand: {}", print_hand)

        player_strings.push(player_string);
    }

    // Printing the resulting strings here
    println!("{}", game_info_string);
    for string in player_strings {
        println!("{}", string); 
    }
}

/// Displays the winner(s) of a completed poker game.
///
/// This function prints information about who won the game and their winning hand.
/// It handles both single-winner scenarios and ties with multiple winners.
///
/// # Parameters
/// * `game_state` - The current state of the game with winner information
/// * `viewer_username` - Username of the player viewing the results
///
/// # Output
/// Prints winner information to the console:
/// * For a single winner: Shows the winner's name (or "you" if viewer is winner) and their winning hand
/// * For multiple winners: Lists all winners' names and shows the tied winning hand
pub fn print_winner(game_state: GameState, viewer_username: String) {
    let num_winners = game_state.winner.len();
    let winning_hand = game_state.winner[0].1;

    // Multiple winners here
    if num_winners > 1 {
        println!("{}", game_state.current_action_string);
        // println!("Each person took home {}", );
        print!("The winners are: ");
        for (winning_player, _) in game_state.winner {
            print!("{}, ", winning_player.player_name);
        }
        println!("");
    }
    else {
        let winner_name = game_state.winner[0].0.player_name.clone();
        if winner_name == viewer_username {
            println!("The sole winner was you with the winning hand: {}", winning_hand);
        }
        else {
            println!("The sole winner was {} with the winning hand: {}", winner_name, winning_hand);
        }
    }
}