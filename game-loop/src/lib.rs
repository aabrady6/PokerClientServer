//! # Poker Playing System Documentation
//!
//! A Client-Server application for playing poker!
//! Supports Five Card Draw, Five Card Stud, and Texas Hold 'Em.
//!
//! Provides an easy-to-use interface for hosting and joining
//! poker tables using TCP/IP protocol. Fun and exciting!
//!
//! ## Modules
//! - `game`: Holds any general information about any poker variant.
//! - `game_type`: Holds information and implementations of specific poker variants.
//! - `db`: Holds the MongoDB database that stores game and player statistics.
//! - `ui`: Creates the main menu of the application.
//!
//! ## Traits
//! - `SendRecv`: A trait for serial communication devices to handle sending/receiving data.
//!
//! ## Dependencies
//!
//! ### Essential Dependencies:
//! - [`rand`]: A library for generating random numbers, used for shuffling cards and generating randomness in the game.
//! - [`itertools`]: A set of utilities to enhance Rust’s iterator capabilities, providing functions like combinations that make it easier to implement various game-related logic.
//! - [`serde`]: Used for serializing and deserializing Rust data structures into formats like JSON or BSON, especially important for database interaction and communication between the client and server.
//! - [`futures`]: Provides abstractions for working with asynchronous operations, simplifying the handling of async tasks like networking and database operations.
//! - [`tokio`]: An asynchronous runtime for Rust that powers many async operations in the game, including interactions with MongoDB and handling concurrent network connections.
//!
//! ### Web and Server Dependencies:
//! - [`actix-web`]: A framework for building asynchronous web servers. It’s used to handle incoming HTTP requests and WebSocket connections, providing a real-time interface between the clients and the server.
//! - [`actix-web-actors`]: Adds WebSocket support to Actix-web, enabling the server to handle WebSocket connections to manage real-time communication with players in the poker game.
//! - [`actix-cors`]: A middleware for Actix that allows you to configure CORS (Cross-Origin Resource Sharing), enabling your web application to communicate with the server across different origins.
//! - [`actix`]: A powerful actor framework for building concurrent systems in Rust. It is used to manage the state and behavior of individual game components, like player connections and game states.
//!
//! ### Security and Database Dependencies:
//! - [`argon2`]: A password hashing library used to securely hash player passwords and verify them during the login process.
//! - [`once_cell`]: A utility for defining lazily initialized global variables, useful for managing the broadcast sender that distributes game state updates to all connected players.
//! - [`serde_json`]: Provides support for working with JSON data, allowing easy serialization and deserialization of data between the server and clients.
//!
//! ## Overview of the Game Flow
//! The server manages the game logic and state, including player management, game variants (Five Card Draw, Five Card Stud, Texas Hold 'Em), and real-time communication via WebSockets.
//! The game allows multiple players to join, and the state of the game is broadcast to all connected players during each round. Players can make decisions (fold, bet, etc.), and the server will determine the winner of each round and update the database with game statistics.
//!
//! ## Communication Protocols
//! The system uses WebSockets to handle real-time communication with clients. Players are able to join the game, make moves, and receive game updates using a persistent WebSocket connection.
//!
//! ## MongoDB Integration
//! The system stores game and player data (such as player statistics and game outcomes) in a MongoDB database. This includes player information (e.g., player name, money, games played, etc.) and game statistics (e.g., number of wins, losses, etc.).

/// DB contains all code and scripts for the database to interact with the program
pub mod db;
/// Game contains the logic for a given poker game
pub mod game;
/// UI contains code related to UI elements for Project Part 1.
pub mod ui;
