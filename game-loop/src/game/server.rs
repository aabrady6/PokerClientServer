//! # WebSocket Connection Handling
//!
//! This module defines the WebSocket connection actor and its associated logic for handling client connections, broadcasting messages to all clients, and managing active sessions.

use crate::db::auth::login_player;
use crate::db::dbclient::{DbClient, MONGO_URI};
use crate::game::client_messages::MessageType;
use crate::game::game_state::GameState;
use crate::game::player::Player;
use crate::game::player::PlayerChoice;
use actix::clock::timeout;
use actix::{Actor, Addr, AsyncContext, Handler, Message as ActMessage, Running, StreamHandler};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext};
use itertools::Itertools;
use serde_json::{from_str, json, Map, Value};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{broadcast, mpsc, watch, Mutex as tMutex};
use tokio::time::Duration;

/// allows games to tell server to broadcast game state to all clients
pub static BROADCAST_SENDER: OnceLock<broadcast::Sender<()>> = OnceLock::new();

/// Message struct used to broadcast a message to all WebSocket clients.
struct BroadcastMessage(String);

/// The result type of the message,
impl ActMessage for BroadcastMessage {
    type Result = ();
}

/// Handles `BroadcastMessage` by sending the provided message to the WebSocket client.
///
/// # Arguments
///
/// * `msg` - The `BroadcastMessage` containing the message to be sent to the client.
/// * `ctx` - The WebSocket context for the actor, used to send the message to the client.
impl Handler<BroadcastMessage> for WebSocketConnection {
    type Result = ();

    /// Send the broadcast message as a message to the client.
    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// A WebSocket connection actor that manages a single client connection,
/// handles broadcasting messages to all clients, and tracks active WebSocket sessions.
pub struct WebSocketConnection {
    /// A shared, thread-safe collection of active WebSocket sessions.
    /// This is used to track all connected clients and broadcast messages to them.
    pub sessions: Arc<Mutex<HashSet<Addr<WebSocketConnection>>>>,
    /// The sender for sending messages from this WebSocket connection.
    pub tx: mpsc::Sender<String>,
}

impl Actor for WebSocketConnection {
    /// The type of context used by this actor. `ws::WebsocketContext<Self>`
    /// is used for managing WebSocket-specific functionality.
    type Context = ws::WebsocketContext<Self>;

    /// Called when the WebSocket connection is started.
    /// It adds the connection address to the `sessions` set and prints the current number of active sessions.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the actor which provides access to the actor's address and other utilities.
    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(addr.clone());
        }
        println!(
            "Rust: WebSocket connection started. Total sessions: {}",
            self.sessions.lock().unwrap().len()
        );
    }

    /// Called when the WebSocket connection is stopping.
    /// It removes the session from the `sessions` set if it is no longer connected.
    ///
    /// # Arguments
    ///
    /// * `_ctx` - The context of the actor, though it is not used here.
    ///
    /// # Returns
    ///
    /// Returns `Running::Stop` to indicate the actor should stop.
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|s| s.connected());
        println!(
            "Rust: WebSocket connection stopping. Total sessions now: {}",
            sessions.len()
        );
        Running::Stop
    }
}

/// Implements the `StreamHandler` for handling incoming WebSocket messages.
/// This is used to process incoming messages from clients over the WebSocket connection.
impl StreamHandler<Result<Message, ProtocolError>> for WebSocketConnection {
    /// Handles messages received from WebSocket clients.
    ///
    /// It supports handling text messages, ping requests, and connection closure events.
    /// Text messages are forwarded to an MPSC (Multi-Producer, Single-Consumer) channel for further processing.
    /// Ping messages are responded to with a pong, and close messages print the closure reason.
    ///
    /// # Arguments
    ///
    /// * `msg` - A result of `Message` which can either be a valid WebSocket message or an error.
    /// * `ctx` - The WebSocket context, which is used to send responses like `pong` messages or handle connection closure.
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut WebsocketContext<Self>) {
        match msg {
            Ok(Message::Text(text)) => {
                //println!("WebSocket received message: {}", text);
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = tx.send(text.to_string()).await {
                        eprintln!("Failed to send message to MPSC channel: {}", e);
                    }
                });
                //let server_tx = tx.clone();
                //let msg = text.clone(); // Send the received message to the mpsc channel.
                //let _ = rx.send(msg).await;
            }
            Ok(Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(Message::Close(reason)) => {
                println!("WebSocket closing: {:?}", reason);
            }
            Err(err) => {
                eprintln!("WebSocket error: {:?}", err);
            }
            _ => (),
        }
    }
}

/// `Server` struct manages the game state, sessions, message passing, and broadcasting.
///
/// It is parameterized by a type `T` which implements the `PokerGame` trait. The `Server` struct holds the necessary data
/// for managing active game sessions, broadcasting updates to clients, and sending/receiving messages using MPSC channels.
///
/// # Fields
///
/// * `game_state`: A `tMutex<GameState>` representing the current game state, protected by a mutex for thread safety.
/// * `sessions`: A `Arc<Mutex<HashSet<Addr<WebSocketConnection>>>>` representing the set of active WebSocket sessions (clients).
/// * `broadcast_sender`: A `broadcast::Sender<()>` used to broadcast messages to all clients.
/// * `rx`: A `tMutex<mpsc::Receiver<String>>` for receiving messages from the MPSC channel.
/// * `mpsc_tx`: A `mpsc::Sender<String>` for sending messages through the MPSC channel.
pub struct Server {
    pub game_state: tMutex<GameState>,
    pub sessions: Arc<Mutex<HashSet<Addr<WebSocketConnection>>>>,
    pub broadcast_sender: broadcast::Sender<()>,
    pub rx: tMutex<mpsc::Receiver<String>>,
    pub mpsc_tx: mpsc::Sender<String>,
    pub joiners: tMutex<Vec<Player>>,
    pub game_active_tx: watch::Sender<bool>,
    pub game_active_rx: watch::Receiver<bool>,
}

impl Server {
    //****************************************************************
    // INITIALIZATION FUNCTIONS
    //****************************************************************

    /// Creates a new instance of the `Server` struct and initializes all necessary components.
    ///
    /// This function initializes a new `Server` instance with:
    /// - A `broadcast` channel (`broadcast::channel`) for broadcasting messages to all WebSocket clients.
    /// - An MPSC (Multi-Producer, Single-Consumer) channel (`mpsc::channel`) for message passing between different parts of the server.
    /// - A `game_state` initialized with a new `GameState` using a `tMutex` for thread safety.
    /// - A `sessions` field that holds active WebSocket client sessions in a `Mutex` to ensure thread-safe access.
    /// - The `broadcast_sender` field is set to the broadcast channel's sender to broadcast messages to clients.
    /// - The receiver part of the MPSC channel (`mpsc_rx`) is stored in the `rx` field, while the sender part (`mpsc_tx`) is stored in `mpsc_tx`.
    ///
    /// The `BROADCAST_SENDER` global static is also set to the broadcast sender to allow other components to broadcast messages.
    ///
    /// # Returns
    ///
    /// Returns an `Arc<Self>`, a reference-counted smart pointer to the newly created `Server` instance, which ensures the server is shared safely across threads.
    ///
    /// # Example
    ///
    /// ```rust
    /// let server = Server::new();
    /// ```
    ///
    /// # Notes
    ///
    /// - The server initializes the necessary components for managing game state, sessions, and communication between components.
    /// - The function is currently logging "Standard Poker initialize" to the console, which may be used for debugging or initialization tracking.
    pub fn new() -> Arc<Self> {
        println!("Standard Poker initalize");
        let (tx, _rx) = broadcast::channel(100); // Create a broadcast channel
        let (mpsc_tx, mpsc_rx) = mpsc::channel(100);
        BROADCAST_SENDER.set(tx.clone()).ok();
        let (game_active_tx, game_active_rx) = watch::channel(false);

        let server = Arc::new(Self {
            game_state: tMutex::new(GameState::new()),
            sessions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_sender: tx,
            rx: tMutex::new(mpsc_rx),
            mpsc_tx,
            joiners: tMutex::new(Vec::new()),
            game_active_tx,
            game_active_rx,
        });

        // // 🔹 Spawn the listener in a new task
        //let server_clone = server.clone();
        //tokio::spawn(async move {
        //    server_clone.listen_for_websocket_messages().await;
        //});

        server
    }

    //****************************************************************
    // BROADCAST FUNCTIONS
    //****************************************************************

    /// Listens for broadcast messages and triggers an update to all connected clients when a broadcast is received.
    ///
    /// This asynchronous function subscribes to the broadcast channel and continuously listens for new broadcast messages.
    /// When a broadcast is received, it triggers the `broadcast_update` function to send the latest game state to all connected WebSocket clients.
    ///
    /// The method runs an infinite loop and awaits messages on the broadcast channel. Upon receiving a message, it spawns a new asynchronous task to perform the broadcasting.
    ///
    /// # Example
    ///
    /// ```rust
    /// let server = Server::new();
    /// tokio::spawn(async move {
    ///     server.listen_for_broadcasts().await;
    /// });
    /// ```
    ///
    /// # Notes
    /// - This function runs indefinitely and is typically called at server startup to begin listening for broadcast messages.
    pub async fn listen_for_broadcasts(self: Arc<Self>) {
        let mut rx = self.broadcast_sender.subscribe();

        loop {
            match rx.recv().await {
                Ok(()) => {
                    println!("\nListen for broadcasts: Broadcast triggered! Updating clients...\n");

                    // Clone the Arc before moving into the spawn
                    let server_clone = self.clone();

                    // Spawn broadcast as separate task
                    tokio::spawn(async move {
                        Self::broadcast_update(&server_clone).await;
                    });
                }
                Err(e) => {
                    println!("Listen for broadcasts: Broadcast listener error: {}", e);
                }
            }
        }
    }

    /// Broadcasts the game state update to all connected clients.
    ///
    /// This function retrieves the current game state and serializes it into a JSON message, which is then sent to all connected WebSocket sessions.
    /// Each session will receive the update as a message to update the client-side game state.
    ///
    /// # Parameters
    /// - `data`: A reference-counted `Arc<Self>` pointing to the `Server` instance, which contains the game state and sessions.
    ///
    /// # Example
    ///
    /// ```rust
    /// let server = Server::new();
    /// tokio::spawn(async move {
    ///     server.broadcast_update(&server).await;
    /// });
    /// ```
    ///
    /// # Notes
    /// - This method serializes the `GameState` and sends it to all connected clients. If there's an error with locking the game state or serializing the state, an error message is printed.
    pub async fn broadcast_update(data: &Arc<Self>) {
        //println!("BROADCAST UPDATE CALLED");
        // 1. Get game state with async lock
        let message = match {
            let game_lock = data.game_state.try_lock();
            let game_lock = match game_lock {
                Ok(lock) => lock,
                Err(e) => {
                    println!("lock error: {e}");
                    return;
                }
            };
            //println!("got lock");
            serde_json::to_string(&*game_lock)
        } {
            Ok(msg) => msg,
            Err(e) => {
                println!("Broadcast serialization error: {}", e);
                return;
            }
        };

        //println!("Broadcast message prepared.");

        // 2. Get sessions with blocking lock (brief)
        let sessions = data.sessions.lock().unwrap().clone();

        // 3. Send to all connected sessions
        println!("Broadcasting to {} sessions", sessions.len());
        for session in sessions {
            if session.connected() {
                session.do_send(BroadcastMessage(message.clone()));
            }
        }
    }

    //****************************************************************
    // GAME TYPE FUNCTIONS
    //****************************************************************

    /// Handles the game lobby state, including resetting game data or retaining existing game state, and processes game actions.
    ///
    /// This function is responsible for managing the lobby and reacting to player actions in the game lobby.
    /// It will reset game data (except for the lobby and players) if there was a round winner, or retain the existing game state otherwise.
    /// It listens for a game action and processes it (e.g., ending a round), then broadcasts the updated game state to all connected players.
    ///
    /// # Parameters
    /// - `data`: A shared reference to the `Server` instance wrapped in `web::Data`, providing access to game state and game actions.
    ///
    /// # Workflow:
    /// - Resets the game data if a winner has been determined (except for the lobby and players).
    /// - Waits for a game action (e.g., `EndRound`).
    /// - Processes the received game action (e.g., handling a player's round-end action).
    /// - Updates the game state and broadcasts it to all players.
    ///
    /// # Example
    /// ```rust
    /// let server = web::Data::new(Server::new());
    /// server.game_lobby().await;
    /// ```
    ///
    /// # Notes:
    /// - If no game action is received in time, a message is logged indicating a timeout.
    pub async fn game_lobby(data: &web::Data<Server>) {
        let mut rx = data.game_active_rx.clone();
        while *rx.borrow() {
            rx.changed().await.unwrap();
            println!("Game is active.");
        }

        {
            let mut joiners_lock = data.joiners.lock().await;
            if !joiners_lock.is_empty() {
                let mut game_lock = data.game_state.lock().await;
                for player in joiners_lock.clone().iter() {
                    game_lock.lobby.push(player.clone());
                    joiners_lock.retain(|p| p.player_name != player.player_name);
                }
            }
        }

        Server::broadcast_update(data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            if game_lock.winner.len() > 0 {
                // reset, only maintain lobby and players
                println!("----------------------------");
                println!("Retaining lobby info...");
                println!("----------------------------");
                game_lock.game_state_retain_lobby_info();
            }
        }
        {
            let mut rx_lock = data.rx.lock().await;
            match Self::wait_for_game_action(&mut rx_lock).await {
                Some(game_action) => match game_action {
                    MessageType::EndRound {
                        option,
                        dealer_option,
                        player_name,
                    } => {
                        println!(
                            "Received GameAction: Option: {}, DealerOption: {}, PlayerName: {}",
                            option, dealer_option, player_name
                        );
                        Server::handle_join_game_message(&data, option, dealer_option, player_name)
                            .await;
                    }
                    _ => {
                        println!("Received an unexpected message type.");
                    }
                },
                None => {
                    println!("No GameAction received in time.");
                }
            }
        }
        Server::broadcast_update(data).await;
    }

    /// Runs the logic for a Five Card Draw game round, including dealing, betting, discarding, and determining the winner.
    ///
    /// This function handles the flow of a Five Card Draw game, including dealing cards, conducting betting rounds, allowing discards, and determining the winner.
    /// It orchestrates multiple rounds of the game, and after each round, it broadcasts the updated game state to all connected players.
    ///
    /// # Parameters
    /// - `data`: A shared reference to the `Server` instance wrapped in `web::Data`, providing access to game state and player actions.
    ///
    /// # Workflow:
    /// - Initializes a new round of the game by dealing cards, setting up the players, and starting the betting rounds.
    /// - Players participate in a discard round, followed by another betting round.
    /// - The game state is updated and broadcasted after each stage of the game (dealing, betting, discarding, etc.).
    /// - The winner is determined, and the results are written to the database and broadcasted.
    ///
    /// # Example
    /// ```rust
    /// let server = web::Data::new(Server::new());
    /// server.five_card_game().await;
    /// ```
    ///
    /// # Notes:
    /// - The `five_card_game` function assumes that all necessary game mechanics (such as dealing and betting logic) are implemented in the `GameState` structure.
    #[allow(unused_assignments)]
    pub async fn five_card_game(data: web::Data<Server>) {
        {
            let mut game_lock = data.game_state.lock().await;

            game_lock.new_five_card_round().await;
            tokio::task::yield_now().await;
        }
        tokio::task::yield_now().await;

        {
            let mut game_lock = data.game_state.lock().await;
            let player_index = game_lock.get_blinds_starting_player_index().await;
            game_lock.set_current_player(player_index).await;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            let player_index = game_lock.get_starting_player_left_of_dealer().await;
            game_lock.set_current_player(player_index).await;
            game_lock.new_discard_round().await;
        }
        Server::broadcast_update(&data).await;
        Server::discard_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            let player_index = game_lock.get_starting_player_left_of_dealer().await;
            game_lock.set_current_player(player_index).await;
            game_lock.demo_mode = "complete".to_string();
            game_lock.highest_bet = 0;
        }

        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.determine_winner_five_card().await;
            if game_lock.winner.len() > 1 {
                game_lock.current_action_string = format!(
                    "Split Pot {} ways. Each win ${} Highest hand {}",
                    game_lock.winner.len(),
                    game_lock.pot / game_lock.winner.len() as u32,
                    game_lock.winner[0].1
                );
            } else {
                game_lock.current_action_string = format!(
                    "{} wins ${} with hand {}",
                    game_lock.winner[0].0.player_name, game_lock.pot, game_lock.winner[0].1
                );
            }
            game_lock.pay_winners().await;
            game_lock.write_results_to_db().await;
            game_lock.rotate_dealer().await;
        }

        Server::broadcast_update(&data).await;
        let mut num_players_in_lobby = 0;
        {
            let game_lock = data.game_state.lock().await;
            num_players_in_lobby = game_lock.lobby.len();
        }
        println!(
            "{} players still in the lobby to talk to.",
            num_players_in_lobby
        );

        Server::deactivate_game(&data).await;

        for _ in 0..num_players_in_lobby {
            let server_data_clone = data.clone();
            tokio::spawn(async move {
                Server::game_lobby(&server_data_clone).await;
            });
        }
        println!("end of five card draw");
    }

    /// Runs the logic for a Seven Card Stud game round, including dealing, betting, and determining the winner.
    ///
    /// This function manages the full flow of a Seven Card Stud game, including the dealing of cards (face-up and face-down),
    /// conducting multiple betting rounds, and determining the winner. It performs actions for each player and broadcasts the updated
    /// game state after each significant step (betting, dealing, determining winners, etc.).
    ///
    /// # Parameters
    /// - `data`: A shared reference to the `Server` instance wrapped in `web::Data`, providing access to the game state and actions.
    ///
    /// # Workflow:
    /// - The function starts by dealing the cards, conducting betting rounds, and updating the game state.
    /// - It proceeds with the game by dealing additional cards (face-up), updating the current player, and handling betting rounds.
    /// - After all betting rounds and dealing of cards, the winner is determined, and the results are written to the database and broadcasted.
    ///
    /// # Example
    /// ```rust
    /// let server = web::Data::new(Server::new());
    /// server.seven_card_game().await;
    /// ```
    ///
    /// # Notes:
    /// - The game progresses through multiple stages of betting and card dealing.
    #[allow(unused_assignments)]
    pub async fn seven_card_game(data: web::Data<Server>) {
        {
            let mut game_lock = data.game_state.lock().await;

            game_lock.new_seven_card_round().await;
            tokio::task::yield_now().await;
        }
        tokio::task::yield_now().await;

        {
            let mut game_lock = data.game_state.lock().await;
            let player_index = game_lock.determine_lowest_door().await;

            game_lock.set_current_player(player_index).await;
            game_lock.seven_card_antes(player_index).await;

            game_lock.highest_bet = game_lock.minimum_bet;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.deal_face_up(1).await;

            let player_index = game_lock.determine_highest_door().await;
            game_lock.set_current_player(player_index).await;
            game_lock.new_betting_round().await;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.deal_face_up(1).await;

            let player_index = game_lock.determine_highest_door().await;
            game_lock.set_current_player(player_index).await;
            game_lock.new_betting_round().await;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.deal_face_up(1).await;

            let player_index = game_lock.determine_highest_door().await;
            game_lock.set_current_player(player_index).await;
            game_lock.new_betting_round().await;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.deal_face_down(1).await;

            let player_index = game_lock.determine_highest_door().await;
            game_lock.set_current_player(player_index).await;
            game_lock.demo_mode = "complete".to_string();
            game_lock.highest_bet = 0;
            game_lock.new_betting_round().await;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.determine_winner_seven_card().await;
            if game_lock.winner.len() > 1 {
                game_lock.current_action_string = format!(
                    "Split Pot {} ways. Each win ${} Highest hand {}",
                    game_lock.winner.len(),
                    game_lock.pot / game_lock.winner.len() as u32,
                    game_lock.winner[0].1
                );
            } else {
                game_lock.current_action_string = format!(
                    "{} wins ${} with hand {}",
                    game_lock.winner[0].0.player_name, game_lock.pot, game_lock.winner[0].1
                );

                if game_lock.get_remaining_player_count() == 1 {
                    game_lock.current_action_string = format!(
                        "{} wins ${} due to all players folding",
                        game_lock.winner[0].0.player_name, game_lock.pot
                    );

                    println!(
                        "{} wins ${} due to everyone folding",
                        game_lock.winner[0].0.player_name, game_lock.pot
                    );
                } else {
                    println!(
                        "{} wins ${} with hand {}",
                        game_lock.winner[0].0.player_name, game_lock.pot, game_lock.winner[0].1
                    );
                }
            }

            game_lock.pay_winners().await;
            game_lock.write_results_to_db().await;
            game_lock.rotate_dealer().await;
        }

        Server::broadcast_update(&data).await;
        let mut num_players_in_lobby = 0;
        {
            let game_lock = data.game_state.lock().await;
            num_players_in_lobby = game_lock.lobby.len();
        }
        println!(
            "{} players still in the lobby to talk to.",
            num_players_in_lobby
        );

        Server::deactivate_game(&data).await;

        for _ in 0..num_players_in_lobby {
            let server_data_clone = data.clone();
            tokio::spawn(async move {
                Server::game_lobby(&server_data_clone).await;
            });
        }
        println!("end of seven card stud");
    }

    /// Runs the logic for a Texas Holdem game round, including dealing, betting, and determining the winner.
    ///
    /// This function manages the full flow of a Texas Holdem game, including the dealing of cards (face-down and community cards),
    /// conducting multiple betting rounds, and determining the winner. It performs actions for each player and broadcasts the updated
    /// game state after each significant step (betting, dealing, determining winners, etc.).
    ///
    /// # Parameters
    /// - `data`: A shared reference to the `Server` instance wrapped in `web::Data`, providing access to the game state and actions.
    ///
    /// # Workflow:
    /// - The function starts by dealing the cards, conducting betting rounds, and updating the game state.
    /// - It proceeds with the game by dealing additional cards (community face-up), updating the current player, and handling betting rounds.
    /// - After all betting rounds and dealing of cards, the winner is determined, and the results are written to the database and broadcasted.
    ///
    /// # Example
    /// ```rust
    /// let server = web::Data::new(Server::new());
    /// server.texas_card_game().await;
    /// ```
    ///
    /// # Notes:
    /// - The game progresses through multiple stages of betting and card dealing.    
    #[allow(unused_assignments)]
    pub async fn texas_card_game(data: web::Data<Server>) {
        {
            let mut game_lock = data.game_state.lock().await;

            game_lock.new_texas_holdem_round().await;
            tokio::task::yield_now().await;
        }
        tokio::task::yield_now().await;

        {
            let mut game_lock = data.game_state.lock().await;
            let player_index = game_lock.get_blinds_starting_player_index().await;
            game_lock.set_current_player(player_index).await;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.deal_community_cards(3).await;
        }

        Server::broadcast_update(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            let player_index = game_lock.get_blinds_starting_player_index().await;
            game_lock.set_current_player(player_index).await;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.deal_community_cards(1).await;
        }
        Server::broadcast_update(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            let player_index = game_lock.get_blinds_starting_player_index().await;
            game_lock.set_current_player(player_index).await;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.deal_community_cards(1).await;
            game_lock.demo_mode = "complete".to_string();
            game_lock.highest_bet = 0;
        }
        Server::broadcast_update(&data).await;

        {
            let mut game_lock = data.game_state.lock().await;
            let player_index = game_lock.get_blinds_starting_player_index().await;
            game_lock.set_current_player(player_index).await;
        }
        Server::broadcast_update(&data).await;
        Server::betting_round(&data).await;

        {
            let mut broke_game = false;
            let mut game_lock = data.game_state.lock().await;
            game_lock.determine_winner_texas().await;
            if game_lock.winner.len() > 1 {
                game_lock.current_action_string = format!(
                    "Split Pot {} ways. Each win ${} Highest hand {}",
                    game_lock.winner.len(),
                    game_lock.pot / game_lock.winner.len() as u32,
                    game_lock.winner[0].1
                );
            } else if game_lock.winner.len() == 1 {
                game_lock.current_action_string = format!(
                    "{} wins ${} with hand {}",
                    game_lock.winner[0].0.player_name, game_lock.pot, game_lock.winner[0].1
                );

                if game_lock.get_remaining_player_count() == 1 {
                    game_lock.current_action_string = format!(
                        "{} wins ${} due to all players folding",
                        game_lock.winner[0].0.player_name, game_lock.pot
                    )
                }
            } else {
                game_lock.current_action_string = "No one won somehow :)".to_string();
                game_lock.winner = Vec::new();
                broke_game = true;
            }
            if !broke_game {
                game_lock.pay_winners().await;
                game_lock.write_results_to_db().await;
                game_lock.rotate_dealer().await;
            }
        }

        Server::broadcast_update(&data).await;
        let mut num_players_in_lobby = 0;
        {
            let game_lock = data.game_state.lock().await;
            num_players_in_lobby = game_lock.lobby.len();
        }
        println!(
            "{} players still in the lobby to talk to.",
            num_players_in_lobby
        );

        Server::deactivate_game(&data).await;

        for _ in 0..num_players_in_lobby {
            let server_data_clone = data.clone();
            tokio::spawn(async move {
                Server::game_lobby(&server_data_clone).await;
            });
        }
        println!("end of texas");
    }

    //****************************************************************
    // GAME SPECIFIC FUNCTIONS
    //****************************************************************

    /// Handles a single betting round in the Texas Hold'em game.
    ///
    /// This function controls the entire process of a betting round in Texas Hold'em, including player actions (bet, fold, etc.), broadcasting game updates, and managing demo mode. It ensures that all players take their turns in order and progresses the game to the next stage once all players have acted or the round is complete.
    ///
    /// # Parameters
    /// - `data`: A reference to the server data (`web::Data<Server>`), which contains the current game state, player data, and message handling for player actions.
    ///
    /// # Process Overview
    /// 1. **Initial Setup**:  
    ///    The function starts by checking if only one player remains in the game. If so, the round is marked as complete immediately. The function also prepares the game for a new betting round if it isn't the first round.
    ///
    /// 2. **Player Actions**:
    ///    - If the demo mode is active, the function will handle demo mode actions.
    ///    - The function enters a loop where it waits for each player to take their action (bet, fold, etc.).
    ///    - The `broadcast_update` method is called regularly to inform all players of the current state of the game.
    ///    
    /// 3. **Betting Process**:
    ///    - If the round is not complete, the function will continually check for player actions and update the current player's status.
    ///    - Players who have already folded are skipped, and the current action is displayed to all players.
    ///    - Player actions are processed, such as betting or folding, and the game state is updated accordingly.
    ///
    /// 4. **Demo Mode Handling**:
    ///    - If demo mode is active, special handling is performed for player actions, and the round may exit early to simulate demo behavior.
    ///    
    /// 5. **End of Round**:
    ///    - The round is marked as complete when all players have made their choices or when only one player remains.
    ///    - The round ends after all actions have been processed, and the function moves to the next round or concludes the game.
    #[allow(clippy::comparison_chain)]
    #[allow(unused_assignments)]
    #[allow(unused_mut)]
    pub async fn betting_round(data: &web::Data<Server>) {
        let mut round_complete = false;
        {
            let mut game_lock = data.game_state.lock().await;

            round_complete = game_lock.get_remaining_player_count() <= 1;

            game_lock.new_betting_round().await;

            if game_lock.round_number > 0 {
                game_lock.current_player.new_round_choices().await;
            }
        }

        while !round_complete {
            Server::broadcast_update(data).await;

            let mut demo_mode;
            {
                let game_lock = data.game_state.lock().await;
                demo_mode = game_lock.demo_mode.clone();
            }

            if demo_mode == *"active" {
                Self::handle_demo_mode_bet(data).await;
                return;
            }

            {
                let mut game_lock = data.game_state.lock().await;
                round_complete = game_lock.is_betting_round_complete().await;
                if round_complete {
                    game_lock.round_number += 1;
                    game_lock.new_betting_round().await;
                    break;
                }

                if game_lock.get_remaining_player_count() == 1 {
                    return;
                }
                while game_lock.current_player.player_choices.contains_key("Fold") {
                    game_lock.update_current_player().await;
                    let current_player_name = game_lock.current_player.player_name.clone();
                    game_lock.current_action_string =
                        format!("{} is currently betting", current_player_name);
                }

                game_lock.get_available_player_actions().await;

                let current_player_name = game_lock.current_player.player_name.clone();
                game_lock.current_action_string =
                    format!("{} is currently betting", current_player_name);
            }

            Server::broadcast_update(data).await;

            let mut game_round;
            {
                let mut rx = data.rx.lock().await;
                let mut game_lock = data.game_state.lock().await;

                game_round = game_lock.round_number;

                match Self::wait_for_player_action(&mut rx).await {
                    Some(player_action) => match player_action {
                        MessageType::PlayerAction {
                            player_name: _,
                            action,
                            bet_amount,
                        } => {
                            println!(
                                "Received PlayerAction: Action: {}, BetAmount: {}",
                                action, bet_amount
                            );
                            game_lock
                                .handle_player_betting_message(action, bet_amount)
                                .await;
                        }
                        MessageType::DemoMode {
                            player_name: _,
                            status,
                        } => {
                            println!("Received DemoMode action");
                            game_lock.handle_demo_mode_message(status).await;
                        }
                        _ => {
                            println!("Received an unexpected message type.");
                        }
                    },
                    None => {
                        println!("No PlayerAction received in time.");
                        game_lock.current_player_fold().await;
                    }
                }
                game_lock.update_current_player().await;
            }
            Server::broadcast_update(data).await;

            // this dumb block is to allow the big bling to bet in first round. gross
            if game_round == 0 {
                let mut broadcast_flag = false;
                {
                    let mut game_lock = data.game_state.lock().await;

                    if game_lock.get_remaining_player_count() == 1 {
                        return;
                    }

                    let index_big =
                        ((game_lock.dealer + 2) % game_lock.players.len() as u32) as usize;

                    if game_lock.current_player.player_name
                        != game_lock.players[index_big].player_name
                        || game_lock.highest_bet != game_lock.minimum_bet
                    {
                        continue;
                    }
                    game_lock.get_available_player_actions().await;
                    broadcast_flag = true;

                    let current_player_name = game_lock.current_player.player_name.clone();
                    game_lock.current_action_string =
                        format!("{}'s is currently betting", current_player_name);
                }
                if broadcast_flag {
                    Server::broadcast_update(data).await;
                    let mut game_lock = data.game_state.lock().await;
                    let mut rx = data.rx.lock().await;
                    match Self::wait_for_player_action(&mut rx).await {
                        Some(player_action) => match player_action {
                            MessageType::PlayerAction {
                                player_name: _,
                                action,
                                bet_amount,
                            } => {
                                println!(
                                    "Received PlayerAction: Action: {}, BetAmount: {}",
                                    action, bet_amount
                                );
                                game_lock
                                    .handle_player_betting_message(action, bet_amount)
                                    .await;
                            }
                            MessageType::DemoMode {
                                player_name: _,
                                status,
                            } => {
                                println!("Received DemoMode action");
                                game_lock.handle_demo_mode_message(status).await;
                            }
                            _ => {
                                println!("Received an unexpected message type.");
                            }
                        },
                        None => {
                            println!("No PlayerAction received in time.");
                            game_lock.current_player_fold().await;
                        }
                    }

                    game_lock.update_current_player().await;
                    Server::broadcast_update(data).await;
                }
            }
        }
        Server::broadcast_update(data).await;
    }

    /// Handles a discard round in the Texas Hold'em game.
    ///
    /// This function is responsible for managing the discard phase of the game, where players can choose to discard one or more cards. It ensures that the game progresses through player actions, handles demo mode behavior, and broadcasts updates to all players. If only one player remains, the discard phase is skipped. The function also waits for and processes discard actions from players.
    ///
    /// # Parameters
    /// - `data`: A reference to the server data (`web::Data<Server>`), which contains the current game state, player data, and message handling for player actions.
    ///
    /// # Process Overview
    /// 1. **Initial Setup**:  
    ///    The function checks the number of remaining players. If only one player is left, the discard phase ends immediately. It then proceeds by updating the current player’s action status to indicate that the player is currently discarding their cards.
    ///
    /// 2. **Player Actions**:
    ///    - The function enters a loop where it waits for discard actions from each player. The action is either discarding a card or folding.
    ///    - If demo mode is active, the discard phase will end immediately without processing player discard actions.
    ///
    /// 3. **Handling Discard Actions**:
    ///    - For each player, the function waits for a discard action (which card to discard). It processes the discard action by calling the `handle_player_discard_message` method.
    ///    - If no discard action is received in time, the current player is forced to fold.
    ///
    /// 4. **Broadcasting Updates**:
    ///    - The game state is broadcasted to all players after each action or update, keeping everyone informed about the current state of the discard round.
    ///
    /// 5. **End of Round**:
    ///    - After all players have taken their discard actions, or if demo mode is active, the discard round is concluded. The function then moves to the next stage of the game or continues based on the game rules.
    #[allow(unused_assignments)]
    pub async fn discard_round(data: &web::Data<Server>) {
        let remaining_players;
        {
            let mut game_lock = data.game_state.lock().await;
            game_lock.new_discard_round().await;
            remaining_players = game_lock.get_remaining_player_count();
            if game_lock.get_remaining_player_count() == 1 {
                return;
            }

            while game_lock.current_player.player_choices.contains_key("Fold") {
                game_lock.update_current_player().await;
                let current_player_name = game_lock.current_player.player_name.clone();
                game_lock.current_action_string =
                    format!("{} is currently betting", current_player_name);
            }

            let current_player_name = game_lock.current_player.player_name.clone();
            game_lock.current_action_string =
                format!("{} is currently discarding", current_player_name);
        }

        Server::broadcast_update(data).await;

        for _ in 0..remaining_players {
            {
                let game_lock = data.game_state.lock().await;
                if game_lock.demo_mode == *"active".to_string() {
                    return;
                }
            }

            {
                let mut rx = data.rx.lock().await;
                let mut game_lock = data.game_state.lock().await;

                if game_lock.get_remaining_player_count() == 1 {
                    return;
                }

                match Self::wait_for_discard_action(&mut rx).await {
                    Some(discard_action) => match discard_action {
                        MessageType::DiscardAction {
                            player_name,
                            card_index,
                        } => {
                            println!("Received DiscardAction: Action: {:?}", card_index);
                            game_lock
                                .handle_player_discard_message(player_name, card_index)
                                .await;
                        }
                        MessageType::DemoMode {
                            player_name: _,
                            status,
                        } => {
                            println!("Received DemoMode action");
                            game_lock.handle_demo_mode_message(status).await;
                        }
                        _ => {
                            println!("Received an unexpected message type.");
                        }
                    },
                    None => {
                        println!("No PlayerAction received in time.");
                        game_lock.current_player_fold().await;
                    }
                }
                game_lock.update_current_player().await;

                let current_player_name = game_lock.current_player.player_name.clone();
                game_lock.current_action_string =
                    format!("{} is currently discarding", current_player_name);
            }
            Server::broadcast_update(data).await;
        }
    }

    /// Handles the betting logic during demo mode, including checking, calling, and folding.
    ///
    /// This function simulates the betting actions of players in demo mode. It processes players' choices (check, call, fold) based on their available money and current bet, then updates the game state accordingly. The function groups players into three categories: those who can check, call, or must fold, and handles each group sequentially. It updates the game state after each group action and proceeds to the next betting round.
    ///
    /// # Parameters
    /// - `data`: A reference to the server data (`web::Data<Server>`) that contains the game state, including player data, available actions, and current bets.
    ///
    /// # Process Overview
    /// 1. **Initialize Action Vectors**:  
    ///    The function starts by initializing three vectors: `check_vec`, `call_vec`, and `fold_vec`, which represent players who will check, call, or fold, respectively. It determines the player's actions based on their current bet and available money relative to the highest bet in the round.
    ///
    /// 2. **Categorizing Players**:
    ///    - **Check**: Players who have placed the highest bet and can check (no additional action) are added to `check_vec`.
    ///    - **Call**: Players who can match the highest bet with their available money are added to `call_vec`.
    ///    - **Fold**: Players who cannot afford to call the highest bet or have already folded are added to `fold_vec`.
    ///
    /// 3. **Executing Actions**:
    ///    - The function then processes each vector in turn:
    ///        - **Check**: Players in `check_vec` will check the round (no additional bet).
    ///        - **Call**: Players in `call_vec` will call the round (match the highest bet).
    ///        - **Fold**: Players in `fold_vec` will fold (exit the round).
    ///    
    /// 4. **Proceeding to Next Round**:
    ///    After all players have taken their actions (check, call, or fold), the function updates the round number and initiates a new betting round.
    pub async fn handle_demo_mode_bet(data: &web::Data<Server>) {
        let mut game_lock = data.game_state.lock().await;
        let mut check_vec: Vec<usize> = vec![];
        let mut call_vec: Vec<usize> = vec![];
        let mut fold_vec: Vec<usize> = vec![];

        let highest_bet = game_lock.highest_bet;

        for (idx, player) in game_lock.players.iter_mut().enumerate() {
            if player.player_choices.contains_key("Fold") {
                continue;
            }

            let placed_in_pot = match player.player_choices.get("PlacedInPot") {
                Some(PlayerChoice::PlacedInPot(amount)) => *amount,
                _ => 0,
            };

            if placed_in_pot == highest_bet {
                check_vec.push(idx);
            } else if player.player_money + placed_in_pot >= highest_bet {
                call_vec.push(idx);
            } else {
                fold_vec.push(idx);
            }
        }

        println!("CHECK VEC: {:?}", check_vec);
        if !check_vec.is_empty() {
            for idx in check_vec {
                game_lock.set_current_player(idx).await;
                game_lock.current_player_check().await;
            }
        }

        println!("CALL VEC: {:?}", call_vec);
        if !call_vec.is_empty() {
            for idx in call_vec {
                game_lock.set_current_player(idx).await;
                game_lock.current_player_call().await;
            }
        }

        println!("FOLD VEC: {:?}", fold_vec);
        if !fold_vec.is_empty() {
            for idx in fold_vec {
                game_lock.set_current_player(idx).await;
                game_lock.current_player_fold().await;
            }
        }

        game_lock.round_number += 1;
        game_lock.new_betting_round().await;
    }

    //****************************************************************
    // SERVER SIDE GAME LISTENERS FUNCTIONS
    //****************************************************************

    /// Waits for a game action message, specifically looking for `EndRound` messages.
    ///
    /// This function listens for incoming WebSocket messages and processes them until a valid `EndRound` message is received. It handles invalid message formats and other types of messages by ignoring them. If the WebSocket channel is closed, the function will return `None`.
    ///
    /// # Parameters
    /// - `rx`: A mutable reference to a `Receiver<String>`, which is the message queue to receive WebSocket messages.
    ///
    /// # Returns
    /// - `Some(MessageType)` if a valid `EndRound` message is received.
    /// - `None` if the WebSocket channel is closed.
    ///
    /// # Example Usage
    /// ```rust
    /// if let Some(game_action) = wait_for_game_action(&mut rx).await {
    ///     // Handle the game action
    /// }
    /// ```
    pub async fn wait_for_game_action(rx: &mut Receiver<String>) -> Option<MessageType> {
        loop {
            match rx.recv().await {
                Some(msg) => {
                    //println!("Received WebSocket message: {}", msg);

                    match from_str::<MessageType>(&msg) {
                        Ok(MessageType::EndRound { .. }) => {
                            println!("Valid Game Variety Found!");
                            return Some(from_str(&msg).unwrap());
                        }
                        Ok(_) => {
                            println!("Ignoring non-GameVariety message...");
                            continue;
                        }
                        Err(e) => {
                            println!("Invalid message format: {}. Ignoring...", e);
                            continue;
                        }
                    }
                }
                None => {
                    println!("MPSC WebSocket channel closed.");
                    return None;
                }
            }
        }
    }

    /// Waits for a player action message, specifically looking for `PlayerAction` or `DemoMode` messages.
    ///
    /// This function listens for WebSocket messages and processes them until a valid `PlayerAction` or `DemoMode` message is received. It handles invalid message formats and other types of messages by ignoring them. The function also includes a timeout mechanism that returns `None` if no message is received within 30 seconds.
    ///
    /// # Parameters
    /// - `rx`: A mutable reference to a `Receiver<String>`, which is the message queue to receive WebSocket messages.
    ///
    /// # Returns
    /// - `Some(MessageType)` if a valid `PlayerAction` or `DemoMode` message is received.
    /// - `None` if the WebSocket channel is closed or the timeout expires.
    ///
    /// # Example Usage
    /// ```rust
    /// if let Some(player_action) = wait_for_player_action(&mut rx).await {
    ///     // Handle the player action
    /// }
    /// ```
    pub async fn wait_for_player_action(rx: &mut Receiver<String>) -> Option<MessageType> {
        let timeout_duration = Duration::from_secs(30);

        loop {
            match timeout(timeout_duration, rx.recv()).await {
                Ok(Some(msg)) => {
                    //println!("Received WebSocket message: {}", msg);

                    match from_str::<MessageType>(&msg) {
                        Ok(MessageType::PlayerAction { .. }) => {
                            println!("Valid PlayerAction received!");
                            return Some(from_str(&msg).unwrap());
                        }
                        Ok(MessageType::DemoMode { .. }) => {
                            println!("Moving to demo mode");
                            return Some(from_str(&msg).unwrap());
                        }
                        Ok(_) => {
                            println!("Ignoring non-PlayerAction message...");
                            continue;
                        }
                        Err(e) => {
                            println!("Invalid message format: {}. Ignoring...", e);
                            continue;
                        }
                    }
                }
                Ok(None) => {
                    println!("MPSC WebSocket channel closed.");
                    return None;
                }
                Err(_) => {
                    println!("Timeout: No PlayerAction received in 30 seconds.");
                    return None;
                }
            }
        }
    }

    /// Waits for a discard action message, specifically looking for `DiscardAction` or `DemoMode` messages.
    ///
    /// This function listens for WebSocket messages and processes them until a valid `DiscardAction` or `DemoMode` message is received. It handles invalid message formats and other types of messages by ignoring them. The function also includes a timeout mechanism that returns `None` if no message is received within 30 seconds.
    ///
    /// # Parameters
    /// - `rx`: A mutable reference to a `Receiver<String>`, which is the message queue to receive WebSocket messages.
    ///
    /// # Returns
    /// - `Some(MessageType)` if a valid `DiscardAction` or `DemoMode` message is received.
    /// - `None` if the WebSocket channel is closed or the timeout expires.
    ///
    /// # Example Usage
    /// ```rust
    /// if let Some(discard_action) = wait_for_discard_action(&mut rx).await {
    ///     // Handle the discard action
    /// }
    /// ```
    pub async fn wait_for_discard_action(rx: &mut Receiver<String>) -> Option<MessageType> {
        let timeout_duration = Duration::from_secs(30);

        loop {
            match timeout(timeout_duration, rx.recv()).await {
                Ok(Some(msg)) => {
                    //println!("Received WebSocket message: {}", msg);

                    match from_str::<MessageType>(&msg) {
                        Ok(MessageType::DiscardAction { .. }) => {
                            println!("Valid DiscardAction received!");
                            return Some(from_str(&msg).unwrap());
                        }
                        Ok(MessageType::DemoMode { .. }) => {
                            println!("Moving to demo mode");
                            return Some(from_str(&msg).unwrap());
                        }
                        Ok(_) => {
                            println!("Ignoring non-DiscardAction message...");
                            continue;
                        }
                        Err(e) => {
                            println!("Invalid message format: {}. Ignoring...", e);
                            continue;
                        }
                    }
                }
                Ok(None) => {
                    println!("MPSC WebSocket channel closed.");
                    return None;
                }
                Err(_) => {
                    println!("Timeout: No PlayerAction received in 30 seconds.");
                    return None;
                }
            }
        }
    }

    /// Handles a statistics request message, processing it based on the specified `stats_menu_type`.
    ///
    /// This function processes the received stats request message and queries the database for relevant statistics data. The response is formatted in JSON and returned as a `Value`. The function supports three types of statistics menu:
    /// 1. **Player statistics** - Retrieves data related to players (e.g., player names, money, win percentages, etc.).
    /// 2. **Game statistics** - Retrieves data related to games (e.g., game variants, winners, players, etc.).
    /// 3. **Single game statistics** - Retrieves detailed information for a single game using its ID.
    ///
    /// The response contains the requested data in a JSON format, or an error message if there are issues with database retrieval or data formatting.
    ///
    /// # Parameters
    /// - `stats_message`: The `MessageType` enum representing the statistics request message. It contains the `stats_menu_type` (e.g., "player", "games", or "single_game") and an optional `selected_option` for a specific game ID.
    ///
    /// # Returns
    /// - A `Value` containing the statistics data in JSON format for the requested menu type (`player_data`, `games_data`, or `single_game_data`).
    /// - `Value::Null` if an unexpected message type is received or if an error occurs during database operations.
    ///
    /// # Example Usage
    /// ```rust
    /// let stats_message = MessageType::StatsMenu {
    ///     stats_menu_type: "player".to_string(),
    ///     selected_option: "".to_string(),
    /// };
    /// let response = handle_stats_request_message(stats_message).await;
    /// println!("{:?}", response);
    /// ```
    pub async fn handle_stats_request_message(stats_message: MessageType) -> Value {
        if let MessageType::StatsMenu {
            stats_menu_type,
            selected_option,
        } = stats_message
        {
            let uri = &MONGO_URI;
            let db_client = DbClient::new(uri).await.unwrap();

            let mut response = Map::new();

            match stats_menu_type.as_str() {
                "player" => {
                    let result = db_client.query_all::<Player>().await;

                    match result {
                        Ok(players) => {
                            if players.is_empty() {
                                println!("No players found in the database.");
                            } else {
                                let mut players_json_arr = Vec::new();
                                let mut players_json = Map::new();
                                let mut playerids = Vec::new();
                                let mut usernames = Vec::new();
                                let mut money = Vec::new();
                                let mut hands_played = Vec::new();
                                let mut win_percentages = Vec::new();
                                let mut wins = Vec::new();
                                let mut losses = Vec::new();
                                let mut total_earnings = Vec::new();
                                for player in players {
                                    let win_percent = if player.get_games() == 0 {
                                        0.0
                                    } else {
                                        (player.get_wins() as f64 / player.get_games() as f64)
                                            * 100.0
                                    };
                                    let win_percent_formatted = format!("{:.2}", win_percent);
                                    playerids.push(player.player_id);
                                    usernames.push(player.player_name);
                                    money.push(player.player_money);
                                    hands_played.push(player.total_games);
                                    win_percentages.push(win_percent_formatted.clone());
                                    wins.push(player.total_wins);
                                    losses.push(player.total_losses);
                                    total_earnings.push(player.total_earnings);
                                }
                                for (i, _) in playerids.clone().iter().enumerate() {
                                    players_json.insert(
                                        "player_id".to_owned(),
                                        Value::Number(playerids[i].into()),
                                    );
                                    players_json.insert(
                                        "username".to_owned(),
                                        Value::String(usernames[i].clone()),
                                    );
                                    players_json
                                        .insert("money".to_owned(), Value::Number(money[i].into()));
                                    players_json.insert(
                                        "hands_played".to_owned(),
                                        Value::Number(hands_played[i].into()),
                                    );
                                    players_json.insert(
                                        "win_percentage".to_owned(),
                                        Value::String(win_percentages[i].clone()),
                                    );
                                    players_json
                                        .insert("wins".to_owned(), Value::Number(wins[i].into()));
                                    players_json.insert(
                                        "losses".to_owned(),
                                        Value::Number(losses[i].into()),
                                    );
                                    players_json.insert(
                                        "total_earnings".to_owned(),
                                        Value::Number(total_earnings[i].into()),
                                    );
                                    players_json_arr.push(players_json.clone());
                                }
                                response.insert(
                                    "player_data".to_owned(),
                                    Value::String(
                                        serde_json::to_string(&players_json_arr)
                                            .expect("could not serialize players json"),
                                    ),
                                );
                                return Value::Object(response);
                            }
                        }
                        Err(e) => {
                            println!("Error retrieving players: {}", e);
                        }
                    }
                }
                "games" => {
                    let result = db_client.query_all::<GameState>().await;

                    match result {
                        Ok(games) => {
                            if games.is_empty() {
                                println!("No games found in the database.");
                            } else {
                                let mut games_json_arr = Vec::new();
                                let mut games_json = Map::new();
                                let mut gameids = Vec::new();
                                let mut variants = Vec::new();
                                let mut players = Vec::new();
                                let mut winners = Vec::new();
                                let mut dealer = Vec::new();
                                for game in games {
                                    gameids.push(game.game_id);
                                    variants.push(game.game_variant);
                                    players.push(game.players.len());
                                    winners.push(game.winner);
                                    dealer.push(game.dealer);
                                }
                                for (i, _) in gameids.clone().iter().enumerate() {
                                    games_json.insert(
                                        "game_id".to_owned(),
                                        Value::Number(gameids[i].into()),
                                    );
                                    games_json.insert(
                                        "game_variant".to_owned(),
                                        Value::String(variants[i].clone()),
                                    );
                                    games_json.insert(
                                        "num_players".to_owned(),
                                        Value::Number(players[i].into()),
                                    );
                                    games_json.insert(
                                        "winners".to_owned(),
                                        Value::Array(
                                            winners[i]
                                                .iter()
                                                .map(|(w, _)| {
                                                    Value::String(w.get_name().to_string())
                                                })
                                                .collect(),
                                        ),
                                    );
                                    games_json.insert(
                                        "dealer_idx".to_owned(),
                                        Value::Number(dealer[i].into()),
                                    );
                                    games_json_arr.push(games_json.clone());
                                }
                                response.insert(
                                    "games_data".to_owned(),
                                    Value::String(
                                        serde_json::to_string(&games_json_arr)
                                            .expect("could not serialize games json"),
                                    ),
                                );
                                return Value::Object(response);
                            }
                        }
                        Err(e) => {
                            println!("Error retrieving games: {}", e);
                        }
                    }
                }
                "single_game" => {
                    let user_search_game =
                        GameState::new_dummy_game_state(selected_option.parse::<u32>().unwrap());
                    let result = db_client.query_one(&user_search_game).await;

                    match result {
                        Ok(Some(retrieved_game)) => {
                            // let mut searched_game: Vec<GameState> = Vec::new();
                            // searched_game.push(retrieved_game);
                            let mut single_game_json_arr = Vec::new();
                            let mut single_game_json = Map::new();
                            let mut playerids = Vec::new();
                            let mut playernames = Vec::new();
                            let mut hands = Vec::new();
                            let mut total_wagereds = Vec::new();
                            let mut chips_won = Vec::new();
                            for player in retrieved_game.players {
                                playerids.push(player.player_id);
                                playernames.push(player.player_name);
                                hands.push(player.player_hand);
                                total_wagereds.push(player.total_wagered_per_game);
                                chips_won.push(player.round_win);
                            }
                            for (i, _) in playerids.clone().iter().enumerate() {
                                single_game_json.insert(
                                    "player_id".to_owned(),
                                    Value::Number(playerids[i].into()),
                                );
                                single_game_json.insert(
                                    "player_name".to_owned(),
                                    Value::String(playernames[i].clone()),
                                );
                                single_game_json.insert(
                                    "hand".to_owned(),
                                    Value::Array(
                                        hands[i]
                                            .cards
                                            .iter()
                                            .map(|c| Value::String(c.to_string()))
                                            .collect(),
                                    ),
                                );
                                single_game_json.insert(
                                    "total_wagered".to_owned(),
                                    Value::Number(total_wagereds[i].into()),
                                );
                                single_game_json.insert(
                                    "chips_won".to_owned(),
                                    Value::Number(chips_won[i].into()),
                                );
                                single_game_json_arr.push(single_game_json.clone());
                            }
                            response.insert(
                                "single_game_data".to_owned(),
                                Value::String(
                                    serde_json::to_string(&single_game_json_arr)
                                        .expect("could not serialize single game json"),
                                ),
                            );
                            return Value::Object(response);
                        }
                        Ok(None) => {
                            println!("\nGame not found.");
                        }
                        Err(e) => {
                            println!("Error retrieving Game: {}", e);
                        }
                    }
                }
                _ => {
                    println!("Unknown stats menu type: {}", stats_menu_type);
                }
            }
        } else {
            println!("Received an unexpected message type: {:?}", stats_message);
        }
        Value::Null
    }

    /// Handles a player's action to join, spectate, or leave a game table.
    ///
    /// This function processes messages related to a player joining a game table, spectating a game, or leaving a table.
    /// Depending on the `option` provided, the following actions are taken:
    /// - **"Join table"**: A player joins the game, either by being a new player or by rejoining an existing game. The function updates the player's information and the game state. If the player is the dealer, they may reset the game with a new game variant.
    /// - **"Spectate game"**: A player chooses to spectate the game, joining the list of spectators and ensuring they are properly added to the `dealer_choice_spectators` list if needed. The player is removed from the active players list.
    /// - **"Leave table"**: A player leaves the table, which involves removing the player from the `players`, `spectators`, and `lobby` lists. If the lobby is empty after the player leaves, the game state is reset, including the dealer position.
    ///
    /// # Parameters
    /// - `data`: The shared server state, including game state and player data.
    /// - `option`: The action the player wants to take (e.g., "Join table", "Spectate game", "Leave table").
    /// - `dealer_option`: The game variant chosen by the player if they are joining as the dealer (e.g., "Five-Card Draw", "Texas Hold 'Em").
    /// - `player_name`: The name of the player performing the action.
    ///
    /// # Returns
    /// This function doesn't return any values. It modifies the game state based on the player's action.
    ///
    /// # Example Usage
    /// ```rust
    /// handle_join_game_message(&server_data, "Join table".to_string(), "Texas Hold 'Em".to_string(), "Alice".to_string()).await;
    /// ```
    pub async fn handle_join_game_message(
        data: &web::Data<Server>,
        option: String,
        dealer_option: String,
        player_name: String,
    ) {
        match option.as_str() {
            "Join table" => {
                let mut game_lock = data.game_state.lock().await;

                if game_lock
                    .players
                    .iter()
                    .all(|p| p.get_name() != &player_name)
                {
                    let mut new_player = Player::new(&player_name);
                    let uri = &MONGO_URI;
                    let db_client = DbClient::new(uri).await.unwrap();
                    let result = db_client.query_one::<Player>(&new_player).await;
                    match result {
                        Ok(Some(p)) => {
                            new_player = p;
                        }
                        Ok(None) => {
                            println!("\nPlayer not found in db.");
                        }
                        Err(e) => {
                            println!("Error retrieving player: {}", e);
                        }
                    };
                    new_player.token = "".to_string();

                    match game_lock.insert_player(&new_player) {
                        Ok(()) => println!("Player added successfully!"),
                        Err(e) => println!("Error inserting player: {}", e),
                    };
                } else {
                    let this_player_idx = game_lock
                        .players
                        .iter()
                        .find_position(|p| p.get_name() == &player_name);
                    match this_player_idx {
                        Some((idx, _)) => {
                            // if you are the dealer, and this is NOT a brand new lobby
                            game_lock.players[idx].token = "".to_string();
                            if dealer_option != "" {
                                game_lock.winner.clear();
                                // clear all players but yourself
                                game_lock
                                    .players
                                    .retain(|player| player.player_name == player_name);
                                game_lock
                                    .spectators
                                    .retain(|player| player.player_name == player_name);
                                // set dealer index to be 0, since you are now in the first place as the dealer
                                game_lock.dealer = 0;
                            }
                        }
                        None => println!("Error: player {} is missing!!", player_name),
                    };
                }

                match dealer_option.as_str() {
                    "Five-Card Draw" => game_lock.game_variant = "5 Card Draw".to_string(),
                    "Seven-Card Stud" => game_lock.game_variant = "7 Card Stud".to_string(),
                    "Texas Hold 'Em" => game_lock.game_variant = "Texas Hold'em".to_string(),
                    _ => println!("Invalid/no game variant chosen."),
                };
                println!("successfully joined table");
            }
            "Spectate game" => {
                println!("Player {} chose to spectate game.", player_name);
                let mut game_lock = data.game_state.lock().await;
                game_lock
                    .players
                    .retain(|player| player.player_name != player_name);
                if game_lock
                    .dealer_choice_spectators
                    .iter()
                    .all(|p| p.get_name() != &player_name)
                {
                    let new_player = Player::new(&player_name);

                    match game_lock.insert_player_to_dealer_choice_spectators(&new_player) {
                        Ok(()) => println!("Dealer choice spectator added successfully!"),
                        Err(e) => println!("Error inserting player: {}", e),
                    };
                }
                if game_lock
                    .spectators
                    .iter()
                    .all(|p| p.get_name() != &player_name)
                {
                    let new_player = Player::new(&player_name);

                    match game_lock.insert_player_to_spectators(&new_player) {
                        Ok(()) => println!("Spectator added successfully!"),
                        Err(e) => println!("Error inserting player: {}", e),
                    };
                }
                // ensures there is a dealer
                println!(
                    "Player length: {}, dealer index: {}",
                    (game_lock.players.len() as u32),
                    game_lock.dealer
                );
                loop {
                    if game_lock.dealer == 0 {
                        break;
                    }
                    if game_lock.players.len() <= usize::try_from(game_lock.dealer).unwrap() {
                        game_lock.dealer -= 1;
                    } else {
                        break;
                    }
                }
            }
            "Leave table" => {
                println!("Player {} chose to leave the table.", player_name);
                let mut game_lock = data.game_state.lock().await;
                game_lock
                    .players
                    .retain(|player| player.player_name != player_name);
                game_lock
                    .spectators
                    .retain(|player| player.player_name != player_name);
                game_lock
                    .lobby
                    .retain(|player| player.player_name != player_name);
                // check if lobby is empty, and if so, reset entire game state
                // ensures there is a dealer
                println!(
                    "Player length: {}, dealer index: {}",
                    (game_lock.players.len() as u32),
                    game_lock.dealer
                );
                loop {
                    if game_lock.dealer == 0 {
                        break;
                    }
                    if game_lock.players.len() <= usize::try_from(game_lock.dealer).unwrap() {
                        game_lock.dealer -= 1;
                    } else {
                        break;
                    }
                }
                if game_lock.lobby.len() == 0 {
                    let new_game_state = GameState::new();
                    *game_lock = new_game_state;
                }
            }
            _ => println!("Unknown join game message: {option} {dealer_option} {player_name}"),
        };
    }

    /// Marks the game as active by setting the `game_active` flag to `true`.
    ///
    /// This function sends a `true` signal through the `game_active_tx` watch channel
    /// in the [`Server`] instance, notifying all listeners that the game has started.
    pub async fn activate_game(data: &web::Data<Server>) {
        println!("Game is ACTIVE");
        let _ = data.game_active_tx.send(true);
    }

    /// Marks the game as deactive by setting the `game_active` flag to `false`.
    ///
    /// This function sends a `true` signal through the `game_active_tx` watch channel
    /// in the [`Server`] instance, notifying all listeners that the game has started.
    pub async fn deactivate_game(data: &web::Data<Server>) {
        println!("Game is NOT ACTIVE");
        let _ = data.game_active_tx.send(false);
    }
}

/// WebSocket handler for establishing a WebSocket connection with clients.
///
/// This handler is triggered when a client sends a request to the `/ws/` endpoint.
/// It initializes the WebSocket connection, using the sessions and transmission
/// channels provided by the server state, and sets up the communication between
/// the client and the server.
///
/// # Arguments
///
/// * `req` - The HTTP request object that contains details about the WebSocket connection request.
/// * `stream` - The payload stream that will carry data between the client and the server.
/// * `data` - Shared server data containing the state of the application, including active sessions
///           and the message-passing transmission channel.
///
/// # Returns
///
/// Returns a `Responder` that either establishes the WebSocket connection or handles any errors
/// that may occur during the connection setup.
#[get("/ws/")]
async fn websocket(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    data: web::Data<Server>,
) -> impl Responder {
    let sessions = data.sessions.clone();
    let tx = data.mpsc_tx.clone();
    let resp = ws::start(WebSocketConnection { sessions, tx }, &req, stream);
    println!("Rust: WebSocket client connected!");
    resp
}

/// HTTP handler for retrieving statistics based on query parameters.
///
/// This handler listens to the `/stats` endpoint and processes the query parameters sent by the client.
/// It retrieves statistics based on the provided message type and returns a JSON response containing
/// player data, game data, or single game data as needed.
///
/// # Arguments
///
/// * `req` - The HTTP request object, which contains the query string sent by the client.
///
/// # Returns
///
/// Returns a JSON response containing statistics based on the query. If no valid statistics are found,
/// it returns an empty response. The response is structured with the keys `player_data`, `games_data`,
/// and `single_game_data`.
#[allow(clippy::needless_return)]
#[get("/stats")]
async fn stats(req: HttpRequest) -> impl Responder {
    println!("Rust: Getting stats from query: {:?}", req.query_string());

    let params = web::Query::<MessageType>::from_query(req.query_string());

    // TODO: do better error handling on if it does not match a message type

    let response_json = Server::handle_stats_request_message(params.unwrap().into_inner()).await;

    return match response_json {
        Value::Null => {
            let mut map = Map::new();
            map.insert("player_data".to_string(), Value::String("".to_string()));
            map.insert("games_data".to_string(), Value::String("".to_string()));
            map.insert(
                "single_game_data".to_string(),
                Value::String("".to_string()),
            );
            // println!("Sending back stats reponse: {:#?}", map);
            HttpResponse::Ok().json(map)
        }
        _ => {
            // println!("Sending back stats reponse: {:#?}", response_json);
            HttpResponse::Ok().json(response_json)
        }
    };
}

/// Starts a new game based on the chosen game variant.
///
/// This handler listens for a GET request to the `/startgame` endpoint. When the request is made, it
/// retrieves the game variant currently set in the `game_state` and moves players who chose to spectate
/// into the appropriate vector. It then starts a new game using the selected game type in a new async task.
///
/// # Arguments
///
/// * `data` - Shared server data containing the state of the application, which includes information
///           about the current game state and players.
///
/// # Returns
///
/// Returns an HTTP response with a success message. The response is a JSON object with an empty string.
#[get("/startgame")]
#[allow(unused_assignments)]
async fn startgame(data: web::Data<Server>) -> impl Responder {
    println!("Rust: Starting game");

    let mut game_type = "".to_string();

    {
        let mut game_lock = data.game_state.lock().await;
        game_type = game_lock.game_variant.clone();
        // move all those who chose to spectate into the correct vector
        game_lock.spectators = game_lock.dealer_choice_spectators.clone();
        game_lock.dealer_choice_spectators.clear();
    }

    println!("Starting game now!!");
    let server_data_clone = data.clone();

    tokio::spawn(async move {
        match game_type.as_str() {
            "5 Card Draw" => {
                Server::activate_game(&server_data_clone).await;
                Server::five_card_game(server_data_clone).await
            }
            "7 Card Stud" => {
                Server::activate_game(&server_data_clone).await;
                Server::seven_card_game(server_data_clone).await
            }
            "Texas Hold'em" => {
                Server::activate_game(&server_data_clone).await;
                Server::texas_card_game(server_data_clone).await
            }
            _ => {
                println!("Cannot start game of unknown type")
            }
        }
    });
    HttpResponse::Ok().json("{}")
}

/// Handles player login requests.
///
/// This handler listens for POST requests to the `/login/{player_name}` endpoint. It verifies the player's
/// login credentials and either successfully adds the player to the lobby or returns an error if the credentials
/// are invalid or if the username in the path doesn't match the one in the request body.
///
/// # Arguments
///
/// * `data` - Shared server data containing the state of the application, including the game state and lobby.
/// * `player_name` - The player name extracted from the request URL path.
/// * `login_message` - The message containing login credentials (username and password).
///
/// # Returns
///
/// Returns an HTTP response indicating whether the login was successful or not. If successful, the player
/// is added to the lobby and an update is broadcasted. If unsuccessful, an error message is returned.
#[post("/login/{player_name}")]
#[allow(unused_assignments)]
async fn player_login(
    data: web::Data<Server>,
    player_name: web::Path<String>,
    login_message: web::Json<MessageType>,
) -> impl Responder {
    let player_name = player_name.into_inner();
    let login_message = login_message.into_inner();

    if let MessageType::UserLogin { username, password } = login_message {
        if player_name != username {
            return HttpResponse::BadRequest().json(json!({
                "error": "Username mismatch",
                "message": "Username in path does not match the username in the request body."
            }));
        }

        match login_player(username.clone(), password.clone()).await {
            Ok(player) => {
                {
                    let game_lock = data.game_state.try_lock();
                    match game_lock {
                        Ok(mut game_lock) => {
                            if let Some(existing_player) = game_lock
                                .lobby
                                .iter()
                                .find(|p| p.player_name == player.player_name)
                            {
                                println!(
                                    "Player '{:?}' is being replaced in the lobby.",
                                    existing_player.player_name
                                );

                                game_lock
                                    .lobby
                                    .retain(|p| p.player_name != player.player_name);
                            } else {
                                println!(
                                    "Player '{}' is joining the lobby for the first time.",
                                    player.player_name
                                );
                            }

                            {
                                let mut joiners_lock = data.joiners.lock().await;
                                joiners_lock.push(player.clone());
                                println!("LOGING JOINER {:?}", joiners_lock);
                            }
                        }
                        Err(..) => {
                            let mut joiners_lock = data.joiners.lock().await;
                            joiners_lock.push(player.clone());
                        }
                    }
                }

                println!(
                    "Login Successful: {:?}. Adding to Lobby",
                    player.player_name
                );
                Server::broadcast_update(&data).await;

                let server_data_clone = data.clone();

                tokio::spawn(async move {
                    Server::game_lobby(&server_data_clone).await;
                });
                HttpResponse::Ok().json(json!({
                    "message": "Login successful",
                    "username": player.player_name
                }))
            }
            Err(error_message) => {
                println!("Login Unsuccessful: {error_message}");
                HttpResponse::Unauthorized().json(json!({
                    "error": error_message
                }))
            }
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "error": "Invalid request format"
        }))
    }
}

/// Handles player registration requests.
///
/// This handler listens for POST requests to the `/register/{player_name}` endpoint. It processes the registration
/// of a new player by validating the provided username and password, checking if the username already exists in the
/// database, and if not, adding the new player to the database and the game lobby.
///
/// # Arguments
///
/// * `data` - Shared server data containing the state of the application, including the current game state and player lobby.
/// * `player_name` - The player name extracted from the request URL path.
/// * `register_message` - The message containing registration details, including the username and password.
///
/// # Returns
///
/// Returns an HTTP response indicating whether the registration was successful or not:
/// - If the registration is successful, the player is added to the lobby, and a success message is returned.
/// - If the registration fails (e.g., username already exists), an error message is returned.
#[post("/register/{player_name}")]
#[allow(unused_assignments)]
async fn player_register(
    data: web::Data<Server>,
    player_name: web::Path<String>,
    register_message: web::Json<MessageType>,
) -> impl Responder {
    let player_name = player_name.into_inner();
    let register_message = register_message.into_inner();

    if let MessageType::UserRegistration { username, password } = register_message {
        if player_name != username {
            return HttpResponse::BadRequest().json(json!({
                "error": "Username mismatch",
                "message": "Username in path does not match the username in the request body."
            }));
        }

        let mut query_player = Player::empty();

        query_player.player_name = username.clone();

        let db_client = DbClient::new(&MONGO_URI).await.unwrap();

        let result = db_client.query_one(&query_player).await;

        match result {
            Ok(Some(_retrieved_player)) => {
                println!("Username already exists in DB. Registration unsuccessful.");
                HttpResponse::Unauthorized().json(json!({
                "error": "User already exists in db"
                }))
            }
            Ok(None) => {
                let new_id = db_client.get_new_player_id().await;

                let inserted_player = match Player::new_player(username, password, new_id) {
                    Ok(player) => player,
                    Err(err) => {
                        return HttpResponse::InternalServerError().json(json!({
                            "error": format!("Failed to create player: {}", err)
                        }))
                    }
                };
                let insert_result = db_client.insert(&inserted_player).await;

                match insert_result {
                    Ok(Some(p)) => {
                        println!(
                            "Registration Successful: {:?}. Adding to Lobby",
                            p.player_name
                        );
                        {
                            let mut joiners_lock = data.joiners.lock().await;
                            joiners_lock.push(p);
                            println!("{:?}", joiners_lock);
                        }
                        Server::broadcast_update(&data).await;

                        let server_data_clone = data.clone();

                        tokio::spawn(async move {
                            Server::game_lobby(&server_data_clone).await;
                        });
                        HttpResponse::Ok().json(json!({
                            "message": "Login successful",
                            "username": inserted_player.player_name
                        }))
                    }
                    Ok(None) => {
                        println!("Player was not inserted for an unknown reason.");
                        HttpResponse::InternalServerError().json(json!({
                            "error": "Unknown insertion failure"
                        }))
                    }
                    Err(error_message) => {
                        println!("Error inserting into DB. Registration unsuccessful.");
                        HttpResponse::Unauthorized().json(json!({
                        "error": error_message
                        }))
                    }
                }
            }
            Err(e) => HttpResponse::BadRequest().json(json!({
            "error": e
            })),
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "error": "Invalid request format"
        }))
    }
}

/// Asynchronous function that listens for admin commands via standard input.
///
/// This function continuously reads lines from `stdin` and processes admin commands:
///
/// - `"stop"`: Shuts down the server immediately.
/// - `"reset"`: Resets game and player statistics in the database.
///
/// # Behavior
/// - Runs indefinitely, processing user input in a loop.
/// - Uses `tokio::io::BufReader` to handle asynchronous input.
/// - Interacts with a MongoDB database via `DbClient` to reset statistics.
pub async fn admin_controls() {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    while let Some(line_result) = lines.next_line().await.transpose() {
        match line_result {
            Ok(line) => match line.trim() {
                "stop" => {
                    println!("Shutting down server...");
                    std::process::exit(0);
                }
                "reset" => {
                    let db_client = DbClient::new(&MONGO_URI).await.unwrap();
                    let game = db_client.reset_game_stats().await;
                    match game {
                        Ok(_) => println!("Game stats are reset"),
                        Err(e) => println!("Error Deleting Games: {}", e),
                    }

                    let players = db_client.reset_player_stats().await;
                    match players {
                        Ok(_) => println!("Player stats are reset"),
                        Err(e) => println!("Error Deleting Player Stats: {}", e),
                    }
                }
                _ => {
                    println!("Unknown command: {}. \n Available Commands: \nstop - shuts down the server\nreset - resets the db stats", line);
                }
            },
            Err(e) => {
                eprintln!("Error reading line: {}", e);
            }
        }
    }
}
