use crate::db::dbclient::DbClient;
use crate::game::client_messages::MessageType;
use crate::game::game_state::GameState;
use crate::game::player::Player;
use crate::game::player::PlayerChoice;
use actix::clock::timeout;
use actix::{Actor, Addr, AsyncContext, Handler, Message as ActMessage, Running, StreamHandler};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext};
use serde_json::{from_str, Map, Value};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{broadcast, mpsc, Mutex as tMutex};
use tokio::time::Duration;

// allows games to tell server to broadcast game state to all clients
pub static BROADCAST_SENDER: OnceLock<broadcast::Sender<()>> = OnceLock::new();

// Custom message for broadcasting updates.
struct BroadcastMessage(String);

impl ActMessage for BroadcastMessage {
    type Result = ();
}

impl Handler<BroadcastMessage> for WebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// WebSocket actor to manage client connections.
pub struct WebSocketConnection {
    pub sessions: Arc<Mutex<HashSet<Addr<WebSocketConnection>>>>,
    pub tx: mpsc::Sender<String>,
}

impl Actor for WebSocketConnection {
    type Context = ws::WebsocketContext<Self>;

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

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|s| !s.connected());
        println!(
            "Rust: WebSocket connection stopping. Total sessions now: {}",
            sessions.len()
        );
        Running::Stop
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for WebSocketConnection {
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
/// Define a generic Server struct, which is parameterized by a type T that implements the PokerGame trait.
///
pub struct Server {
    pub game_state: tMutex<GameState>,
    pub sessions: Arc<Mutex<HashSet<Addr<WebSocketConnection>>>>,
    pub broadcast_sender: broadcast::Sender<()>,
    pub rx: tMutex<mpsc::Receiver<String>>,
    pub mpsc_tx: mpsc::Sender<String>,
}

impl Server {
    //****************************************************************
    // INITIALIZATION FUNCTIONS
    //****************************************************************

    pub fn new() -> Arc<Self> {
        println!("Standard Poker initalize");
        let (tx, _rx) = broadcast::channel(100); // Create a broadcast channel
        let (mpsc_tx, mpsc_rx) = mpsc::channel(100);
        BROADCAST_SENDER.set(tx.clone()).ok();

        let server = Arc::new(Self {
            game_state: tMutex::new(GameState::new()),
            sessions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_sender: tx,
            rx: tMutex::new(mpsc_rx),
            mpsc_tx,
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

    /// Listener task to watch for broadcasts and trigger `broadcast_update`
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

    pub async fn five_card_game(data: web::Data<Server>) {
        {
            let mut game_lock = data.game_state.lock().await;

            game_lock.new_five_card_round().await;
            tokio::task::yield_now().await;

            //TODO: REMOVE THIS, just have it here for testing purposes
            for i in 0..game_lock.players.len() {
                game_lock.players[i].player_id = i as u32;
            }
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
        }

        Server::broadcast_update(&data).await;

        // write to db
    }

    pub async fn seven_card_game(data: web::Data<Server>) {
        {
            let mut game_lock = data.game_state.lock().await;

            game_lock.new_seven_card_round().await;
            tokio::task::yield_now().await;

            //TODO: REMOVE THIS, just have it here for testing purposes
            for i in 0..game_lock.players.len() {
                game_lock.players[i].player_id = i as u32;
            }
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
        }

        Server::broadcast_update(&data).await;

        // db write
    }

    pub async fn texas_card_game(data: web::Data<Server>) {
        {
            let mut game_lock = data.game_state.lock().await;

            game_lock.new_texas_holdem_round().await;
            tokio::task::yield_now().await;

            //TODO: REMOVE THIS, just have it here for testing purposes
            for i in 0..game_lock.players.len() {
                game_lock.players[i].player_id = i as u32;
            }
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
            let mut game_lock = data.game_state.lock().await;
            game_lock.determine_winner_texas().await;
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
                    )
                }
            }
            game_lock.pay_winners().await;
        }

        Server::broadcast_update(&data).await;

        // db write
    }

    //****************************************************************
    // GAME SPECIFIC FUNCTIONS
    //****************************************************************

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

    pub async fn handle_stats_request_message(stats_message: MessageType) -> Value {
        if let MessageType::StatsMenu {
            stats_menu_type,
            selected_option,
        } = stats_message
        {

            let uri = "mongodb://localhost:27017";
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
                                        (player.get_wins() as f64 / player.get_games() as f64) * 100.0
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
                                    players_json.insert("player_id".to_owned(), Value::Number(playerids[i].into()));
                                    players_json.insert("username".to_owned(), Value::String(usernames[i].clone()));
                                    players_json.insert("money".to_owned(), Value::Number(money[i].into()));
                                    players_json.insert("hands_played".to_owned(), Value::Number(hands_played[i].into()));
                                    players_json.insert("win_percentage".to_owned(), Value::String(win_percentages[i].clone()));
                                    players_json.insert("wins".to_owned(), Value::Number(wins[i].into()));
                                    players_json.insert("losses".to_owned(), Value::Number(losses[i].into()));
                                    players_json.insert("total_earnings".to_owned(), Value::Number(total_earnings[i].into()));
                                    players_json_arr.push(players_json.clone());
                                }
                                response.insert("player_data".to_owned(), Value::String(serde_json::to_string(&players_json_arr).expect("could not serialize players json")));
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
                                    games_json.insert("game_id".to_owned(), Value::Number(gameids[i].into()));
                                    games_json.insert("game_variant".to_owned(), Value::String(variants[i].clone()));
                                    games_json.insert("num_players".to_owned(), Value::Number(players[i].into()));
                                    games_json.insert("winners".to_owned(), Value::Array(winners[i].iter().map(|(w, _)| Value::String(w.get_name().to_string())).collect()));
                                    games_json.insert("dealer_idx".to_owned(), Value::Number(dealer[i].into()));
                                    games_json_arr.push(games_json.clone());
                                }
                                response.insert("games_data".to_owned(), Value::String(serde_json::to_string(&games_json_arr).expect("could not serialize games json")));
                                return Value::Object(response);
                            }
                        }
                        Err(e) => {
                            println!("Error retrieving games: {}", e);
                        }
                    }
                }
                "single_game" => {
                    let user_search_game = GameState::new_dummy_game_state(selected_option.parse::<u32>().unwrap());
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
                                single_game_json.insert("player_id".to_owned(), Value::Number(playerids[i].into()));
                                single_game_json.insert("player_name".to_owned(), Value::String(playernames[i].clone()));
                                single_game_json.insert("hand".to_owned(), Value::Array(hands[i].cards.iter().map(|c| Value::String(c.to_string())).collect()));
                                single_game_json.insert("total_wagered".to_owned(), Value::Number(total_wagereds[i].into()));
                                single_game_json.insert("chips_won".to_owned(), Value::Number(chips_won[i].into()));
                                single_game_json_arr.push(single_game_json.clone());
                            }
                            response.insert("single_game_data".to_owned(), Value::String(serde_json::to_string(&single_game_json_arr).expect("could not serialize single game json")));
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
}

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

#[get("/stats")]
async fn stats(
    req: HttpRequest
) -> impl Responder {
    println!("Rust: Getting stats from query: {:?}", req.query_string());

    let params = web::Query::<MessageType>::from_query(req.query_string());

    // TODO: do better error handling on if it does not match a message type

    let response_json = Server::handle_stats_request_message(params.unwrap().into_inner()).await;

    return match response_json {
        Value::Null => {
            let mut map = Map::new();
            map.insert("player_data".to_string(),Value::String("".to_string()));
            map.insert("games_data".to_string(), Value::String("".to_string()));
            map.insert("single_game_data".to_string(), Value::String("".to_string()));
            // println!("Sending back stats reponse: {:#?}", map);
            HttpResponse::Ok().json(map)
        }
        _ => {
            // println!("Sending back stats reponse: {:#?}", response_json);
            HttpResponse::Ok().json(response_json)
        }
    };
}

#[post("/register/{player_name}")]
#[allow(unused_assignments)]
async fn register_player(
    data: web::Data<Server>,
    player_name: web::Path<String>,
) -> impl Responder {
    let player_name = player_name.into_inner();
    println!("Rust: Registering player: {}", player_name);

    {
        let mut game_lock = data.game_state.lock().await;

        if game_lock
            .players
            .iter()
            .all(|p| p.get_name() != &player_name)
        {
            match game_lock.insert_player(&Player::new(&player_name)) {
                Ok(()) => println!("Player added successfully!"),
                Err(e) => println!("Error inserting player: {}", e),
            }
        }
    }

    // 2. Broadcast update AFTER state modification

    Server::broadcast_update(&data).await;

    let sessions_len = data.sessions.lock().unwrap().len();
    if sessions_len >= 3 {
        println!("Starting game now!!");
        let server_data_clone = data.clone();

        tokio::spawn(async move {
            Server::texas_card_game(server_data_clone).await;
            //Server::seven_card_game(server_data_clone).await;
            //Server::five_card_game(server_data_clone).await;
        });
    } else {
        println!("Waiting on more players to join...");
    }

    println!("Returning registration response...");

    // 3. Prepare response with fresh lock
    let response_state = {
        let game_lock = data.game_state.lock().await;
        let game_state = game_lock.get_game_state();
        game_state.clone()
    };

    HttpResponse::Ok().json(response_state)
}
