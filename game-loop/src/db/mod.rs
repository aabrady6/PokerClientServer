/// Connection to the database, serving as the connection point for various calls to the database
/// Calls to find, retrieve, update, and push are found here.
pub mod dbclient;

/// Authentication module used for verifying user credentials
pub mod auth;
