
//! WebSocket-based game client implementation for a card game
//! 
//! This module provides a client implementation for connecting to a card game server
//! via WebSockets, handling authentication, game state updates, and player actions.

use std::io::{self, Write};
use std::sync::Arc;
// use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
// use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::select;
use tokio::time::timeout;
// use tokio::task::spawn_blocking;
use futures_util::sink::SinkExt;
use futures_util::stream::{SplitSink, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
// use serde::{Deserialize, Serialize};
// use serde_json::{json, Value};
use reqwest::Client;
use std::collections::HashSet;
// use futures_util::FutureExt;
use crate::mpsc::error::TryRecvError;
use client_cli::game_structures::GameState;
use client_cli::game_structures::PlayerChoice;
use client_cli::message_types::*;
use dotenv::dotenv;
use std::env;

/// Main client structure for handling game interactions
/// 
/// Manages the connection to the game server, tracks player state,
/// and handles user input and game state updates.
struct GameClient{
    /// Flag to indicate if current round should be discarded
    discard_round: bool,
    /// Flag to indicate if player can start a new game
    can_click_start_game: bool,
    /// Flag to indicate if player is in spectator mode
    is_spectator: bool,
    /// Flag to indicate if player is the dealer
    is_dealer: bool,
    /// Player's username
    username: String,
    /// HTTP client for REST API calls
    http_client: Client,
    /// Base URL for the game server
    server_url: String,
    /// Channel for cancelling pending input operations
    input_cancel_tx: Option<oneshot::Sender<()>>,
    /// Counter for broadcast messages received
    broadcasts_received: u32,
    /// Counter for display refresh requests
    refreshes_count: u32,
}

/// Implementation of Clone trait for GameClient
/// 
/// Note: The input_cancel_tx field is not cloned since oneshot::Sender<()>
/// doesn't implement Clone. It's set to None in the cloned instance.
impl Clone for GameClient {
    fn clone(&self) -> Self {
        GameClient {
            discard_round: self.discard_round,
            can_click_start_game: self.can_click_start_game,
            is_spectator: self.is_spectator,
            is_dealer: self.is_dealer,
            username: self.username.clone(),
            http_client: self.http_client.clone(),
            server_url: self.server_url.clone(),
            input_cancel_tx: None, // Can't clone `oneshot::Sender<()>`, so just set it to None
            broadcasts_received: self.broadcasts_received,
            refreshes_count: self.refreshes_count,
        }
    }
}

impl GameClient{
    /// Creates a new GameClient instance
    /// 
    /// # Arguments
    /// 
    /// * `server_url` - Base URL of the game server
    /// 
    /// # Returns
    /// 
    /// A new GameClient instance with default values
    fn new(server_url: String) -> Self {
        GameClient {
            discard_round: false,
            can_click_start_game: false,
            is_spectator: false,
            is_dealer: false,
            username: String::new(),
            http_client: Client::new(),
            server_url,
            input_cancel_tx: None,
            broadcasts_received: 0,
            refreshes_count: 0,
        }
    }

    /// Reads user input with support for cancellation
    /// 
    /// This function displays a prompt and waits for user input, but can be cancelled
    /// if a new game state is received while waiting.
    /// 
    /// # Arguments
    /// 
    /// * `prompt` - The text prompt to display to the user
    /// * `cancel_rx` - Oneshot receiver that triggers cancellation when a message is received
    /// 
    /// # Returns
    /// 
    /// * `Some(String)` - The user's input if successful
    /// * `None` - If the input operation was cancelled
    async fn read_input_with_cancel(prompt: &str, cancel_rx: &mut oneshot::Receiver<()>) -> Option<String> {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        
        // Create a future for reading input
        let input_future = tokio::task::spawn_blocking(move || {
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => input.trim().to_string(),
                Err(_) => String::new(),
            }
        });
        
        // Wait for either input completion or cancellation
        select! {
            result = input_future => {
                match result {
                    Ok(input) => Some(input),
                    Err(_) => None,
                }
            },
            _ = cancel_rx => {
                // Input was cancelled due to a new game state
                println!("\nInput interrupted - new game state received!");
                None
            },
        }
    }

    /// Registers a new player with the game server
    /// 
    /// Prompts the user for username and password, then sends a registration
    /// request to the server.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If registration was successful or input was cancelled
    /// * `Err(...)` - If registration failed
    async fn register_player(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        self.input_cancel_tx = Some(cancel_tx);
        // print!("Enter username: ");
        let username = match Self::read_input_with_cancel("Enter username: ", &mut cancel_rx).await {
            Some(input) => {
                // Check if just Enter was pressed (refresh request)
                if input.is_empty() {
                    self.refreshes_count += 1;
                    println!("\nRefreshing display... (Refresh count: {}, Broadcasts received: {})", 
                             self.refreshes_count, self.broadcasts_received);
                    return Ok(());
                }
                input
            },
            None => return Ok(()), // Input was cancelled
        };
        
        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        self.input_cancel_tx = Some(cancel_tx);
        // print!("Enter password: ");
        let password = match Self::read_input_with_cancel("Enter password: ", &mut cancel_rx).await {
            Some(input) => {
                // Check if just Enter was pressed (refresh request)
                if input.is_empty() {
                    self.refreshes_count += 1;
                    println!("\nRefreshing display... (Refresh count: {}, Broadcasts received: {})", 
                             self.refreshes_count, self.broadcasts_received);
                    return Ok(());
                }
                input
            },
            None => return Ok(()), // Input was cancelled
        };

        let url = format!("{}/register/{}", self.server_url, username);
        
        let request = MessageType::UserRegistration {
            username: username.clone(),
            password: password.clone(),
        };

        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            println!("Registration successful: {}", result);
            self.username = username;
            Ok(())
        } else {
            let error = response.text().await?;
            println!("Registration failed: {}", error);
            Err("Registration failed".into())
        }
    }

    /// Logs in an existing player to the game server
    /// 
    /// Prompts the user for username and password, then sends a login
    /// request to the server.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If login was successful or input was cancelled
    /// * `Err(...)` - If login failed
    async fn login_player(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        self.input_cancel_tx = Some(cancel_tx);
        // print!("Enter username: ");
        let username = match Self::read_input_with_cancel("Enter username: ", &mut cancel_rx).await {
            Some(input) => {
                // Check if just Enter was pressed (refresh request)
                if input.is_empty() {
                    self.refreshes_count += 1;
                    println!("\nRefreshing display... (Refresh count: {}, Broadcasts received: {})", 
                             self.refreshes_count, self.broadcasts_received);
                    return Ok(());
                }
                input
            },
            None => return Ok(()), // Input was cancelled
        };
        
        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        self.input_cancel_tx = Some(cancel_tx);
        
        // print!("Enter password: ");
        let password = match Self::read_input_with_cancel("Enter password: ", &mut cancel_rx).await {
            Some(input) => {
                // Check if just Enter was pressed (refresh request)
                if input.is_empty() {
                    self.refreshes_count += 1;
                    println!("\nRefreshing display... (Refresh count: {}, Broadcasts received: {})", 
                             self.refreshes_count, self.broadcasts_received);
                    return Ok(());
                }
                input
            },
            None => return Ok(()), // Input was cancelled
        };

        let url = format!("{}/login/{}", self.server_url, username);
        
        let request = MessageType::UserLogin {
            username: username.clone(),
            password: password.clone(),
        };

        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            println!("Login successful: {}", result);
            self.username = username;
            Ok(())
        } else {
            let error = response.text().await?;
            println!("Login failed: {}", error);
            Err("Login failed".into())
        }
    }

    /// Main entry point for the game client
    /// 
    /// Sets up the WebSocket connection, handles authentication,
    /// and manages the game loop for processing state updates and player actions.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the client exits cleanly
    /// * `Err(...)` - If there was an error during operation
    async fn handle_game_client(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let game_state_arc = Arc::new(Mutex::new(GameState::default()));
        let game_client_arc = Arc::new(Mutex::new(GameClient::new(self.server_url.clone())));
        let (game_state_tx, mut game_state_rx) = mpsc::channel::<GameState>(32);
        let (auth_tx, mut auth_rx) = mpsc::channel::<()>(1);
        let server_ip = env::var("RUST_SERVER_IP").unwrap_or_else(|_| "localhost".to_string());
        let server_port = env::var("RUST_SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
        
        let url = format!("ws://{}:{}/ws/", server_ip, server_port);
        println!("Connecting to {}...", url);

        let (ws_stream, _) = connect_async(url).await?;
        let (writer, mut reader) = ws_stream.split();

    // Shared writer for other tasks
        let writer_arc = Arc::new(Mutex::new(writer));

        // Handle incoming game state updates
        let handle = tokio::spawn({
            let game_state_tx = game_state_tx.clone();
            let game_client_arc = game_client_arc.clone();
            async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Text(data)) => {
                        if let Ok(received_game_state) = serde_json::from_str::<GameState>(&data) {
                            println!("\nReceived new game state update from server, please press enter to refresh");
                            
                            // Increment broadcasts counter
                            {
                                let mut client_lock = game_client_arc.lock().await;
                                client_lock.broadcasts_received += 1;
                            }
                            
                            // Send the new game state to the main loop
                            if let Err(_) = game_state_tx.send(received_game_state).await {
                                println!("Error sending game state update, receiver dropped");
                                break;
                            }
                        } else {
                            println!("\nReceived message: {}", data);
                        }

                    }
                    Ok(Message::Close(_)) => {
                        println!("\nWebSocket connection closed by server");
                        break;
                    }
                    _ => {}  // Ignore other message types
                }
            }
        }
        });

        let auth = tokio::spawn({
            let game_client = game_client_arc.clone();
            let auth_tx = auth_tx.clone();
            async move {
            loop{
                println!("\nMain Menu:");
                println!("1. Login");
                println!("2. Register New Player");
                println!("3. Exit");
                let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
                game_client.lock().await.input_cancel_tx = Some(cancel_tx);
                
                let option = match timeout(Duration::from_secs(1000), async {
                    let prompt = "Enter option: ";
                    Self::read_input_with_cancel(prompt, &mut cancel_rx).await
                }).await {
                    Ok(Some(opt)) => opt.trim().to_string(),
                    _ => continue,
                };
        
                match option.as_str() {
                    "1" => {
                        let mut client_lock = game_client.lock().await;
                        match client_lock.login_player().await {
                            Ok(_) => {
                                auth_tx.send(()).await.unwrap();
                                break;
                            },
                            Err(e) => eprintln!("Error logging in player: {}", e),
                        }
                    },
                    "2" => {
                        let mut client_lock = game_client.lock().await;
                        match client_lock.register_player().await {
                            Ok(_) => {
                                auth_tx.send(()).await.unwrap();
                                break;
                            },
                            Err(e) => eprintln!("Error registering player: {}", e),
                        }
                    },
                    "3" => break,
                    _ => continue,
                }
            }
        }
        });

        auth_rx.recv().await;
        println!("Authentication completed. Starting game loop.");
        // let mut action_in_progress = false;
        loop {
            // Check for new game state updates
            // println!("TEST");
            // select! {
                match game_state_rx.try_recv() {
                    Ok(new_game_state) => {
                        println!("Received new game state update");
                        // Cancel any pending IO operations
                        if let Some(cancel_tx) = {
                            let mut client_lock = game_client_arc.lock().await;
                            client_lock.input_cancel_tx.take()
                        } {
                            println!("Cancelling current input operation");
                            let _ = cancel_tx.send(());
                        }
                        
                        // Update the game state
                        {
                            let mut game_state = game_state_arc.lock().await;
                            *game_state = new_game_state.clone();
                        }
                        
                        // Handle the new game state
                        // println!("Handle the new game state");
                        Self::handle_game_state(&new_game_state, &game_client_arc).await;
                        // let mut writer_lock = writer_arc.lock().await;
                        // if let Err(e) = Self::process_player_actions(&new_game_state, &game_client_arc, &mut writer_lock).await {
                        //     eprintln!("Error processing player actions: {}", e);
                        // }
                        // println!("new game state before lock after player actions");
                        // drop(writer_lock);
                        // println!("new game state after lock after player actions");
                        continue;
                    },
                    Err(TryRecvError::Empty) => {
                        // Get the current game state
                        // println!("Getting current game state in alternate process actions");
                        let current_game_state = {
                            let game_state = game_state_arc.lock().await;
                            (*game_state).clone()
                        };
                        // println!("Got current game state in alternate process actions");
                        
                        // Process player actions based on game state
                        let mut writer_lock = writer_arc.lock().await;
                        Self::process_player_actions(&current_game_state, &game_client_arc, &mut writer_lock).await?;
                        drop(writer_lock);
                        // Small delay to prevent CPU spinning
                        // println!("tokio select before lock after player actions");
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    },
                    Err(TryRecvError::Disconnected) => {
                        println!("Game state channel disconnected");
                        break;
                    }
                }
            // }
        }

        // handle.await.unwrap();

        handle.await?;
        auth.await?;

        // game_loop.await?;

        Ok(())
    }

    async fn user_logged_in(game_state: &GameState, username: String) -> bool{
        if username == ""{
            return false;
        }
        if game_state.lobby.iter().any(|p| p.player_name == username){
            return true;
        }
        if game_state.players.iter().any(|p| p.player_name == username){
            return true;
        }
        if game_state.spectators.iter().any(|p| p.player_name == username){
            return true;
        }

        false
    }

    async fn handle_game_state(game_state: &GameState, game_client: &Arc<Mutex<GameClient>>) {
        println!("\n========= Game State Update =========");
        
        // Get client information
        let game_lock = game_client.lock().await;
        let username = game_lock.username.clone();
        println!("Broadcasts received: {}, Refreshes: {}", 
                 game_lock.broadcasts_received, game_lock.refreshes_count);
        drop(game_lock);
        
        // Check if user is logged in
        let logged_in = Self::user_logged_in(game_state, username.clone()).await;
        if !logged_in {
            println!("You are not currently logged into the game.");
            println!("Press Enter at any prompt to refresh the display");
            return;
        }
        
        // Determine user's status and game state
        let game_running = game_state.game_id != 0;
        let in_lobby = game_state.lobby.iter().any(|p| p.player_name == username);
        let in_game = game_state.players.iter().any(|p| p.player_name == username);
        let is_spectator = game_state.spectators.iter().any(|p| p.player_name == username);
        
        let is_dealer = if game_running {
            game_state.players.get(game_state.dealer as usize)
                .map_or(false, |p| p.player_name == username)
        } else {
            game_state.lobby.get(game_state.dealer as usize)
                .map_or(false, |p| p.player_name == username)
        };
        
        // Update client state
        {
            let mut client = game_client.lock().await;
            client.is_dealer = is_dealer;
            client.is_spectator = is_spectator;
            client.can_click_start_game = is_dealer && in_lobby && game_state.lobby.len() >= 2;
            client.discard_round = game_state.discard_cards_prompted;
        }
        // 1. Check state after logging in
        if logged_in {
            println!("You are logged in as: {}", username);
            
            // 2. Just logged in status
            if is_dealer {
                println!("You are the dealer!");
                if in_lobby && game_state.lobby.len() >= 2 {
                    println!("You can start the game when ready.");
                }
            } else if game_running {
                println!("A game is currently running: {}", game_state.game_variant);
            } else {
                println!("Waiting in lobby for game to start.");
            }
        }
        
        // Display current game state
        if game_running && !game_state.winner.len() == 0{
            // Find player in the game
            if in_game || is_spectator {
                // let client = game_client.lock().await;
                Self::display_table(game_state, username.to_string(), false);
            }
        } else if in_lobby {
            println!("You are in the lobby waiting for the game to start.");
            Self::display_players(game_state);
        }
        
        println!("======================================");
        println!("Press Enter at any prompt to refresh the display");
    }

    /// Processes player actions based on the current game state.
    /// 
    /// This function handles various scenarios where the player needs to make decisions:
    /// - Starting a game (for dealer)
    /// - Taking turns during active gameplay
    /// - Handling lobby actions between games
    /// - Viewing statistics
    /// 
    /// The function determines the appropriate actions available to the player based on their
    /// role (dealer, player, spectator) and the current game state.
    ///
    /// # Arguments
    /// * `game_state` - Current state of the poker game
    /// * `game_client` - Shared reference to the game client with player information
    /// * `writer` - WebSocket writer to send messages to the server
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Success or error
    async fn process_player_actions(
        game_state: &GameState, 
        game_client: &Arc<Mutex<GameClient>>, 
        writer: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // As soon as the dealer chooses to make a new table it makes a new winner vector, 
        // Until the dealer makes an action everything remains the same as before
        // Not everyone in the lobby can be the dealer, some are spectating, try to work around this
        // println!("Entered into Process player actions");
        let game_lock = game_client.lock().await;
        let username = game_lock.username.clone();
        drop(game_lock);
        // Self::display_players(game_state);
        // Check if user is logged in
        let logged_in = Self::user_logged_in(game_state, username.clone()).await;
        if !logged_in {
            return Ok(());
        }
        let game_running = game_state.pot != 0;
        // let in_lobby = game_state.lobby.iter().any(|p| p.player_name == username);
        let in_game = game_state.players.iter().any(|p| p.player_name == username);
        let is_current_player = game_state.current_player.player_name == username;
        let is_spectator = game_state.spectators.iter().any(|p| p.player_name == username);
        
        let is_dealer = if game_state.players.len() > 0 {
            game_state.players.get(game_state.dealer as usize)
                .map_or(false, |p| p.player_name == username)
        } else {
            game_state.lobby.get(game_state.dealer as usize)
                .map_or(false, |p| p.player_name == username)
        };
        
        // let discard_round = game_state.discard_cards_prompted;
        let end_of_round = game_state.winner.len() > 0;
        let can_click_start_game = is_dealer && game_state.players.len() >= 2 && game_state.winner.len() == 0;

        
        // Handle different scenarios
        // println!("Entering into process player actions");
        // Check if we need to prompt for actions
        let has_actions_to_process = 
            (is_dealer && can_click_start_game && !game_running) ||
            (game_running && in_game && is_current_player) ||
            (!game_running && logged_in) || end_of_round;

        if game_running{ Self::display_table(game_state, username, is_spectator); }
       
        // Always prompt for Enter to refresh, regardless of other actions
        if has_actions_to_process  {
            // println!("You can press Enter to refresh the display at any time, or choose an action:");
        } else {
            // No actions to perform, so we can present them with stats instead.
            return Self::process_player_stats(game_client).await;
        };
        
        
        
        // Scenario 1: Dealer can start game
        if is_dealer && can_click_start_game && !game_running {
            println!("\nDo you want to start the game? (y/n)");
            
            let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
            {
                let mut client = game_client.lock().await;
                client.input_cancel_tx = Some(cancel_tx);
            }
            
            if let Some(input) = {
                Self::read_input_with_cancel("Enter choice: ", &mut cancel_rx).await
            } {
                // Check if user just pressed Enter to refresh
                if input.is_empty() {
                    // Increment refresh counter
                    {
                        let mut client = game_client.lock().await;
                        client.refreshes_count += 1;
                    }
                    println!("Refreshing display...");
                    return Ok(());
                }
                
                if input.to_lowercase() == "y" {
                    // Send start game message
                    println!("Got y for start the game");
                    {
                        let client = game_client.lock().await;
                        let url = format!("{}/startgame", client.server_url);
                        println!("URL: {}", url);
                        let response = client.http_client
                            .get(&url)
                            .send()
                            .await?;
                        if response.status().is_success() {
                            println!("Game start request sent successfully");
                        } else {
                            println!("Failed to start game: {}", response.status());
                        }
                        // println!("Sent start game");
                    }
                    return Ok(());
                }
            }
            return Ok(());
        }
        
        // Scenario 2-5: In-game actions
        if game_running && in_game && is_current_player && !end_of_round{
            // Handle in-game actions
            return Self::process_player_game_action(game_state, game_client, writer).await;
        }
        
        // Handle lobby actions, should be triggered for end of game as well
        if (!game_running && logged_in) || end_of_round{
            return Self::process_player_lobby_action(game_state, game_client, writer, is_dealer).await;
        }
        
        Ok(())
    }

    
    /// Handles in-game player actions during an active game.
    /// 
    /// This function manages two primary scenarios:
    /// 1. Card discard rounds (selecting cards to discard)
    /// 2. Betting rounds (fold, check, call, raise)
    ///
    /// The function presents appropriate options based on the current game state and player's 
    /// position, then sends the chosen action to the server.
    ///
    /// # Arguments
    /// * `game_state` - Current state of the poker game
    /// * `game_client` - Shared reference to the game client with player information
    /// * `writer` - WebSocket writer to send messages to the server
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Success or error
    async fn process_player_game_action(
        game_state: &GameState, 
        game_client: &Arc<Mutex<GameClient>>, 
        writer: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let game_lock = game_client.lock().await;
        let username = game_lock.username.clone();
        drop(game_lock);
        let discard_round = game_state.discard_cards_prompted;

        if discard_round {
            // Handle discard round
            let mut selected_indices = Vec::new();
            
            let player = game_state.players.iter()
                .find(|p| p.player_name == username)
                .unwrap();
            
            println!("\nSelect cards to discard (space-separated indices, or 'none'):");
            println!("Your hand:");
            
            for (i, card) in player.player_hand.cards.iter().enumerate() {
                println!("{}. {}", i+1, card.to_string(true));
            }
            
            let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
            {
                let mut client = game_client.lock().await;
                client.input_cancel_tx = Some(cancel_tx);
            }
            
            if let Some(input) = {
                Self::read_input_with_cancel("Enter indices to discard: ", &mut cancel_rx).await
            } {
                // Check if user just pressed Enter to refresh
                if input.is_empty() {
                    // Increment refresh counter
                    {
                        let mut client = game_client.lock().await;
                        client.refreshes_count += 1;
                    }
                    println!("Refreshing display...");
                    return Ok(());
                }
                
                if input.trim().to_lowercase() != "none" {
                    for idx_str in input.split_whitespace() {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            if idx > 0 && idx <= player.player_hand.cards.len() {
                                selected_indices.push(idx - 1);
                            }
                        }
                    }
                }
                // removing duplicates and keeping it orderly
                let remove_dup: HashSet<usize> = selected_indices.into_iter().collect();
                selected_indices = remove_dup.into_iter().collect();
                selected_indices.sort();
                
                // Send discard action
                let message = serde_json::to_string(&MessageType::DiscardAction { 
                    player_name: username, 
                    card_index: selected_indices,
                })?;
                writer.send(Message::Text(message)).await?;
            }
            return Ok(());
        } else {
            // Handle regular game actions
            println!("\nYour turn! Choose action:");
            let mut options = Vec::new();
            
            if game_state.demo_mode == "inactive" {
                options.push("Demo Mode");
            }
            
            options.push("Fold");
            
            let placed_in_pot = match game_state.current_player.player_choices.get("PlacedInPot") {
                Some(PlayerChoice::PlacedInPot(value)) => *value,
                _ => 0,
            };
            
            if game_state.highest_bet == 0 || placed_in_pot == game_state.highest_bet {
                options.push("Check");
            }
            
            if game_state.highest_bet > 0 && 
            game_state.current_player.player_money >= game_state.highest_bet - placed_in_pot &&
            placed_in_pot < game_state.highest_bet {
                options.push("Call");
            }
            // let mut raise_string = "".to_string();
            if game_state.current_player.player_money > 0 &&
            game_state.current_player.player_money + placed_in_pot >= game_state.raise_min_max.0 {
                options.push("Raise");         
            }
            for (idx, string) in options.iter().enumerate(){
                println!("{}. {}", idx + 1, string);
            }
            
            let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
            {
                let mut client = game_client.lock().await;
                client.input_cancel_tx = Some(cancel_tx);
            }
            
            if let Some(selection) = {
                Self::read_input_with_cancel("Enter choice: ", &mut cancel_rx).await
            } {
                // Check if user just pressed Enter to refresh
                if selection.is_empty() {
                    // Increment refresh counter
                    {
                        let mut client = game_client.lock().await;
                        client.refreshes_count += 1;
                    }
                    println!("Refreshing display...");
                    return Ok(());
                }
                
                let index = selection.parse::<usize>().unwrap_or(0);
                if index > 0 && index <= options.len() {
                    let action = options[index - 1].to_string();
                    
                    let mut amount = 0;
                    if action == "Demo Mode" {
                        {
                            let mut client = game_client.lock().await;
                            client.refreshes_count += 1;
                        }
                        let message = serde_json::to_string(&MessageType::DemoMode { 
                            player_name: username.clone(),
                            status: "active".to_string(),
                        })?;
                        writer.send(Message::Text(message)).await?;
                        return Ok(());
                    }
                    else if action == "Raise" {
                        let raise_min = game_state.raise_min_max.0;
                        let raise_max = game_state.raise_min_max.1;
                        loop{
                            let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
                            {
                                let mut client = game_client.lock().await;
                                client.input_cancel_tx = Some(cancel_tx);
                            }
                            // this raise should not be interupted if the actual raise is brought up
                            if let Some(amount_str) = {
                                Self::read_input_with_cancel(
                                    &format!("Enter raise amount ({}-{}): ", 
                                    raise_min, 
                                    raise_max), 
                                    &mut cancel_rx).await
                            } {
                                // Check if user just pressed Enter to refresh
                                if amount_str.is_empty() {
                                    // Increment refresh counter
                                    {
                                        let mut client = game_client.lock().await;
                                        client.refreshes_count += 1;
                                    }
                                    println!("Refreshing display...");
                                    return Ok(());
                                }
                                
                                let parsed_amount = amount_str.parse::<u32>().unwrap_or(game_state.raise_min_max.0);
                                if parsed_amount <= game_state.raise_min_max.1 && parsed_amount >= game_state.raise_min_max.0{
                                    amount = parsed_amount;
                                    break;
                                }
                                else{
                                    println!("Enter in a correct raise amount");
                                }
                                // amount = parsed_amount.max(game_state.raise_min_max.0).min(game_state.raise_min_max.1);
                            } else {
                                return Ok(()); // Input was cancelled
                            }
                        }

                    }
                    
                    // Send game action
                    {
                        let mut client = game_client.lock().await;
                        client.refreshes_count += 1;
                    }
                    let message = serde_json::to_string(&MessageType::PlayerAction { 
                        action: action.clone(), 
                        player_name: username,
                        bet_amount: amount 
                    })?;
                    writer.send(Message::Text(message)).await?;
                    
                }
            }
            return Ok(());
        }
    
        // Ok(())
    }

    
    /// Handles player actions in the lobby between games.
    /// 
    /// This function presents different options based on the player's role:
    /// - For dealers: selecting game variants (Five Card Draw, Seven Card Stud, etc.)
    /// - For regular players: joining table, spectating, or leaving
    ///
    /// The function also handles end-of-round scenarios where players decide their next action.
    ///
    /// # Arguments
    /// * `game_state` - Current state of the poker game
    /// * `game_client` - Shared reference to the game client with player information
    /// * `writer` - WebSocket writer to send messages to the server
    /// * `is_dealer` - Boolean indicating whether the current player is the dealer
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Success or error
    async fn process_player_lobby_action(
        game_state: &GameState, 
        game_client: &Arc<Mutex<GameClient>>, 
        writer: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        is_dealer: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let game_lock = game_client.lock().await;
        let username = game_lock.username.clone();
        drop(game_lock);
        let end_of_round = game_state.winner.len() > 0;
        let in_game = game_state.players.iter().any(|p| p.player_name == username);
        if in_game && !end_of_round{
            // If no actions are available, just wait for refresh
            let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
            {
                let mut client = game_client.lock().await;
                client.input_cancel_tx = Some(cancel_tx);
            }
            
            if let Some(_) = {
                Self::read_input_with_cancel("Press Enter to refresh: ", &mut cancel_rx).await
            } {
                // Any input (including empty) will refresh
                {
                    let mut client = game_client.lock().await;
                    client.refreshes_count += 1;
                }
                println!("Refreshing display...");

                // Handle stats here and printing it to the client with cancelation
            }
            return Ok(());
        } else {
            if is_dealer{
                println!("\nLobby Options:");
                println!("1. Five Card Draw");
                println!("2. Seven Card Stud");
                println!("3. Texas Hold 'Em");
                println!("4. Spectate game");
                println!("5. Leave table");
                let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
                // println!("Game Selection before game_client_lock");
                {
                    let mut client = game_client.lock().await;
                    client.input_cancel_tx = Some(cancel_tx);
                }
                // println!("Game Selection after game_client_lock");
                if let Some(input) = {
                    Self::read_input_with_cancel("Enter choice: ", &mut cancel_rx).await
                } {
                    // println!("Game Selection Got input");
                    let mut action = "".to_string();
                    let mut dealer_choice = "".to_string();
                    match input.trim() {
                        "1" => {
                            action = "Join table".to_string();
                            dealer_choice = "Five-Card Draw".to_string();},
                        "2" => {
                            action = "Join table".to_string();
                            dealer_choice = "Seven-Card Stud".to_string();},
                        "3" => {
                            action = "Join table".to_string();
                            dealer_choice = "Texas Hold 'Em".to_string();},
                        "4" => action = "Spectate game".to_string(),
                        "5" => action = "Leave table".to_string(),
                        _ => (),
                    };
                    
                    if !action.is_empty() {
                        {
                            let mut client = game_client.lock().await;
                            client.refreshes_count += 1;
                        }
                        let message = serde_json::to_string(&MessageType::EndRound { 
                            option: action, 
                            dealer_option: dealer_choice, 
                            player_name: username
                        })?;
                        writer.send(Message::Text(message)).await?;
                    }
                }
                // println!("Game Selection return ok");
                return Ok(());  
            }
            else if game_state.game_variant != ""{
                // Not in lobby, show join options
                println!("\nLobby Options:");
                println!("1. Join table");
                println!("2. Spectate game");
                println!("3. Leave table");
                
                let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
                {
                    let mut client = game_client.lock().await;
                    client.input_cancel_tx = Some(cancel_tx);
                }
                if let Some(input) = {
                    Self::read_input_with_cancel("Enter choice: ", &mut cancel_rx).await
                } {
                    let action = match input.trim() {
                        "1" => "Join table".to_string(),
                        "2" => "Spectate game".to_string(),
                        "3" => "Leave table".to_string(),
                        _ => "".to_string(),
                    };
                    
                    if !action.is_empty() {
                        {
                            let mut client = game_client.lock().await;
                            client.refreshes_count += 1;
                        }
                        let message = serde_json::to_string(&MessageType::EndRound { 
                            option: action,
                            dealer_option: "".to_string(), // Non-dealer doesn't specify game type
                            player_name: username
                        })?;
                        writer.send(Message::Text(message)).await?;
                    }
                }
                return Ok(());
            }
        }
        Ok(())
    }

    /// Retrieves and displays player statistics from the server.
    /// 
    /// This function provides a stats menu with different views:
    /// 1. All Player stats - Shows stats for all players
    /// 2. All Games stats - Shows information about all games played
    /// 3. Specific Game stats - Shows detailed information about a specific game ID
    ///
    /// The function sends HTTP requests to fetch the requested statistics and displays
    /// them in formatted tables.
    ///
    /// # Arguments
    /// * `game_client` - Shared reference to the game client with connection information
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Success or error
    async fn process_player_stats(
        game_client: &Arc<Mutex<GameClient>>, 
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut options: Vec<&str> = Vec::new();
        options.push("player");
        options.push("games");
        options.push("single_game");

        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        {
            let mut client = game_client.lock().await;
            client.input_cancel_tx = Some(cancel_tx);
        }
        println!("Stats menu request: ");
        println!("1. All Player");
        println!("2. All Games");
        println!("3. Specific Game");
        
        if let Some(selection) = {
            Self::read_input_with_cancel("Enter choice: ", &mut cancel_rx).await
        } {
            // Check if user just pressed Enter to refresh
            if selection.is_empty() {
                // Increment refresh counter
                {
                    let mut client = game_client.lock().await;
                    client.refreshes_count += 1;
                }
                println!("Refreshing display...");
                return Ok(());
            }
            let mut stats_menu_type = "".to_string();
            let mut selected_option = "".to_string();
            match selection.trim(){
                "1" => stats_menu_type = "player".to_string(),
                "2" => stats_menu_type = "games".to_string(),
                "3" => {stats_menu_type = "single_game".to_string();
                selected_option = Self::read_input_with_cancel("Enter game ID: ", &mut cancel_rx).await.unwrap();}
                _ => (),
            }

            // let message = MessageType::StatsMenu { 
            //     stats_menu_type: stats_menu_type.clone(),
            //     selected_option,
            // };
            
            let game_client_lock = game_client.lock().await;
            // let url = format!("{}/stats?type=StatsMenu&stats_menu_type=${}&selected_option=${}", game_client_lock.server_url, stats_menu_type, selected_option);
            // println!("Stats url: {}", url);
            // let response = game_client_lock.http_client
            //     .post(&url)
            //     // .json(&message)
            //     .send()
            //     .await?;
            let url = format!(
                "{}/stats?type=StatsMenu&stats_menu_type={}&selected_option={}", 
                game_client_lock.server_url, 
                stats_menu_type, 
                selected_option
            );
            
            // println!("Fetching stats from: {}", url);
            
            // Using GET instead of POST since the server endpoint is defined as GET
            let response = game_client_lock.http_client
                .get(&url)
                .send()
                .await?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                return Self::handle_stats_response(result).await;
                // return Ok(());
            } else {
                let error = response.text().await?;
                println!("Request failed: {}", error);
            }
        }
        
        Ok(())
    }

    /// Parses and displays statistics data received from the server.
    /// 
    /// This function formats and displays three types of statistics:
    /// 1. Player statistics - Shows player performance data like wins, losses, earnings
    /// 2. Game statistics - Shows information about games that have been played
    /// 3. Single game statistics - Shows detailed information about a specific game
    ///
    /// # Arguments
    /// * `response_json` - JSON response from the server containing statistics data
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Success or error
    async fn handle_stats_response(response_json: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Handling player data
        if let Some(player_data_str) = response_json.get("player_data").and_then(|v| v.as_str()) {
            if !player_data_str.is_empty() {
                let player_stats: Vec<serde_json::Value> = serde_json::from_str(player_data_str)?;
                println!("\n=== PLAYER STATS ===");
                println!("{:<6} {:<15} {:<10} {:<12} {:<15} {:<6} {:<6} {:<15}", 
                    "ID", "Username", "Money", "Hands", "Win %", "Wins", "Losses", "Earnings");
                
                for player in player_stats {
                    println!("{:<6} {:<15} {:<10} {:<12} {:<15} {:<6} {:<6} {:<15}",
                        player["player_id"].as_i64().unwrap_or_default(),
                        player["username"].as_str().unwrap_or_default(),
                        player["money"].as_i64().unwrap_or_default(),
                        player["hands_played"].as_i64().unwrap_or_default(),
                        player["win_percentage"].as_str().unwrap_or_default(),
                        player["wins"].as_i64().unwrap_or_default(),
                        player["losses"].as_i64().unwrap_or_default(),
                        player["total_earnings"].as_i64().unwrap_or_default()
                    );
                }
            } else {
                println!("No player stats available.");
            }
        }

        // Handling games data
        if let Some(games_data_str) = response_json.get("games_data").and_then(|v| v.as_str()) {
            if !games_data_str.is_empty() {
                let games_stats: Vec<serde_json::Value> = serde_json::from_str(games_data_str)?;
                println!("\n=== GAME STATS ===");
                println!("{:<6} {:<15} {:<12} {:<20} {:<10}", 
                    "ID", "Variant", "Players", "Winners", "Dealer");
                
                for game in games_stats {
                    let winners = if let Some(winners_array) = game["winners"].as_array() {
                        winners_array.iter()
                            .filter_map(|w| w.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    } else {
                        String::from("None")
                    };
                    
                    println!("{:<6} {:<15} {:<12} {:<20} {:<10}",
                        game["game_id"].as_i64().unwrap_or_default(),
                        game["game_variant"].as_str().unwrap_or_default(),
                        game["num_players"].as_i64().unwrap_or_default(),
                        winners,
                        game["dealer_idx"].as_i64().unwrap_or_default()
                    );
                }
            } else {
                println!("No game stats available.");
            }
        }

        // Handling single game data
        if let Some(single_game_data_str) = response_json.get("single_game_data").and_then(|v| v.as_str()) {
            if !single_game_data_str.is_empty() {
                let single_game_stats: Vec<serde_json::Value> = serde_json::from_str(single_game_data_str)?;
                println!("\n=== SPECIFIC GAME DETAILS ===");
                println!("{:<6} {:<15} {:<30} {:<15} {:<10}", 
                    "ID", "Player", "Hand", "Wagered", "Won");
                
                for player_data in single_game_stats {
                    let hand = if let Some(cards) = player_data["hand"].as_array() {
                        cards.iter()
                            .filter_map(|c| c.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    } else {
                        String::from("No cards")
                    };
                    
                    println!("{:<6} {:<15} {:<30} {:<15} {:<10}",
                        player_data["player_id"].as_i64().unwrap_or_default(),
                        player_data["player_name"].as_str().unwrap_or_default(),
                        hand,
                        player_data["total_wagered"].as_i64().unwrap_or_default(),
                        player_data["chips_won"].as_i64().unwrap_or_default()
                    );
                }
            } else {
                println!("No specific game data available.");
            }
        }

        Ok(())
    }
}

impl GameClient{
    /// Displays all players in the current game state, including lobby, active players, and spectators.
    /// 
    /// # Arguments
    /// * `game_state` - Reference to the current GameState
    pub fn display_players(game_state: &GameState) {
        // println!("Game type: {}, Game ID: {}", game_state.game_variant, game_state.game_id);
        // println!("Game Winners: {:?}", game_state.winner);
        if game_state.spectators.len() > 0{
            println!("Spectators: ");
            for (idx, player) in game_state.spectators.iter().enumerate(){
                println!("{}: {}", idx, player.player_name);
            }
        }
        if game_state.lobby.len() > 0{
            // println!("Dealer in lobby: {}", game_state.lobby[game_state.dealer as usize].player_name);
            println!("Lobby: ");
            for (idx, player) in game_state.lobby.iter().enumerate(){
                println!("{}: {}", idx, player.player_name);
            }
        }

        if game_state.players.len() > 0{
            // println!("Dealer in players: {}", game_state.players[game_state.dealer as usize].player_name);
            println!("All Players: ");
            for (idx, player) in game_state.players.iter().enumerate(){
                println!("{}: {}", idx, player.player_name);
            }
        }

        
    }

    /// Displays the current game table state, including player information, community cards,
    /// and game-specific details.
    /// 
    /// # Arguments
    /// * `game_state` - Reference to the current GameState
    /// * `player_name` - Name of the current player viewing the table
    /// * `spectator` - Boolean indicating if the viewer is a spectator
    pub fn display_table(game_state: &GameState, player_name: String, spectator: bool) {
        // Clear screen
        // print!("\x1B[2J\x1B[1;1H");
        
        // Display game action
        println!("{}", game_state.current_action_string);
        println!();
        
        // Display game variant and pot
        println!("Game: {} | Pot: ${}", 
            game_state.game_variant, 
            game_state.pot.to_string()
        );

        if game_state.raise_min_max.0 > 0 && game_state.raise_min_max.1 > 0{
            println!("Raise To: {}, From: {}", game_state.raise_min_max.0, game_state.raise_min_max.1);
        }
        
        // Display community cards
        if !game_state.community_cards.cards.is_empty() {
            let print_hand = game_state.community_cards.cards
                .iter()
                .map(|card| card.to_string(spectator))
                .collect::<Vec<String>>()
                .join(", ");
            println!("\nCommunity Cards: {}", print_hand)
        }
        
        // Display players
        println!("\nPlayers:");
        for (i, player) in game_state.players.iter().enumerate() {
            let is_dealer = i == game_state.dealer as usize;
            let is_current = if game_state.current_player.player_name == player_name.to_string() {true} else {false};
            
            let mut player_display = format!("{}{}{}: ${}", 
                if is_dealer { "D " } else { "" },
                if is_current { "→ " } else { "  " },
                player.player_name,
                player.player_money
            );
            let placed_in_pot = match player.player_choices.get("PlacedInPot"){
                Some(choice_value) => { match choice_value{
                    PlayerChoice::PlacedInPot(value) => value.clone(),
                    _ => 0,
                }},
                None => 0,
            };
            if placed_in_pot > 0 {
                player_display.push_str(&format!(" (bet: ${})", placed_in_pot));
            }
            
            if &player.last_move != ""{
                player_display.push_str(&format!(" | Last move: {}", player.last_move));
            }
            
            println!("{}", player_display);
            
            // if !player.face_up_cards.is_empty() {
            //     println!("  Face up: {}", display_cards(&player.face_up_cards));
            // }
            
            if player.player_name == player_name {
                let print_hand = player.player_hand.cards
                    .iter()
                    .map(|card| card.to_string(true))
                    .collect::<Vec<String>>()
                    .join(", ");
                println!("  Your hand: {}", print_hand);
            } else{
                let print_hand = player.player_hand.cards
                    .iter()
                    .map(|card| card.to_string(spectator))
                    .collect::<Vec<String>>()
                    .join(", ");
                println!("  Their hand: {}", print_hand);
            }
        }
        
        // Display winner if any
        if !game_state.winner.is_empty() {
            print!("{}", Self::print_winners(&game_state, player_name.to_string()));
            // println!("\n{}", format!("Winners: {}", game_state.winner.join(", ")));
        }
    }

    /// Formats and returns a string containing winner information for the current game.
    /// 
    /// # Arguments
    /// * `game_state` - Reference to the current GameState
    /// * `viewer_username` - Username of the current viewer
    /// 
    /// # Returns
    /// A formatted string containing information about the winners
    pub fn print_winners(game_state: &GameState, viewer_username: String) -> String{
        let num_winners = game_state.winner.len();
        let winning_hand = game_state.winner[0].1;
        let mut return_string = "".to_string();
        // Multiple winners here
        if num_winners > 1{
            return_string = format!("{}{}\nThe winners are: ", return_string, game_state.current_action_string);
            // println!("{}", game_state.current_action_string);
            // println!("Each person took home {}", );
            // print!("The winners are: ");
            for (winning_player, _) in &game_state.winner{
                return_string = format!("{}{}, ", return_string, winning_player.player_name);
                // print!("{}, ", winning_player.player_name);
            }
            return_string = format!("{}\n", return_string);
            // println!("");
        }
        else{
            let winner_name = game_state.winner[0].0.player_name.clone();
            if winner_name == viewer_username{
                return_string = format!("{}The sole winner was you with the winning hand: {}\n", return_string, winning_hand);
                // println!("The sole winner was you with the winning hand: {}", winning_hand);
            }
            else{
                return_string = format!("{}The sole winner was {} with the winning hand: {}\n", return_string, winner_name, winning_hand);
                // println!("The sole winner was {} with the winning hand: {}", winner_name, winning_hand);
            }
            
        }

        return_string
    }
}

/// Main entry point for the poker game client application.
/// Sets up the game client and initiates the client connection.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let server_ip = env::var("RUST_SERVER_IP").unwrap_or_else(|_| "localhost".to_string());
    let server_port = env::var("RUST_SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    
    println!("Using server: {}:{}", server_ip, server_port);
    let mut game_client = GameClient {
        can_click_start_game: false,
        discard_round: false,
        is_dealer: false,
        is_spectator: false,
        username: String::new(),
        http_client: Client::new(),
        server_url: format!("http://{}:{}", server_ip, server_port),
        input_cancel_tx: None,
        broadcasts_received: 0,
        refreshes_count: 0,
    };
    let _ = game_client.handle_game_client().await;

    Ok(())
}