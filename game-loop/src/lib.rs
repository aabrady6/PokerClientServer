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
//! - [`rand`]: allows for random number generation.
//! - [`itertools`]: creates easier implementations of iterator logic.
//! - [`serde`]: used for serialization of objects.
//! - [`futures`]: allows for async functions to be made more easily.
//! - [`tokio`]: helps with MongoDB integration.
//! - [`prettytable`]: used for formatting tables in a nice way.
//! - [`mongodb`]: creating a database.
//!
//!

/// DB contains all code and scripts for the database to interact with the program
pub mod db;
/// Game contains the logic for a given poker game
pub mod game;
/// UI contains code related to UI elements for Project Part 1.
pub mod ui;
