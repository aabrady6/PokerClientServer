# Poker Project Balatro
Aaron Brady, Jason Gillanders, Francis Doll

## Concept

The Poker Playing System is a client-server system by definition in that there is a dealer that manages the poker game and at least two or more players. The dealer can be thought of as a server that manages and coordinates resources, while the players are resource consumers that provide server inputs and feedback.

## Part 1

In part 1 of the Poker Project, we have implemented a localized version of the poker server including 5 Card Draw, and player and game statistic tracking. Upon starting the server, the user will have the option to either create a new user or sign in as an existing user in the data base. This allows for the player stats to be persisted across different instances. 

Upon signing in, a user will have the option to play a game of 5 Card Draw, view Player Stats, View Game stats or reset stats. 

`5 Card Draw` - In this portion of the project, we pre-loaded user's from the database to mock client connections. This will initiate a new 5 Card Draw poker game. Upon completion of a game, the statistics will be sent to the data base.

`5 Card Stud` - *coming soon...*

`Texas Hold'em` - *coming soon...*

`Player Stats` - View Player stats from past games. This allows a user to view the first 100 alphabetic users or search for a specific user's statistics. 

`Game Stats` - View Game stats from previous 100 games. This allows a user to view the previous 100 games, or search for a specific game statistics. Viewing the specific games will allow the stats of the user's who were in the specific game.

`Reset Stats` - This resets all of the statistics for the games, deleting all of the game data. This will reset all of the player stats, but keep the user's in the data base.  

### Running Part 1
In part 1, we are relying on pre-loading players into the games. To do this, we must pre-fill the database to mock existing players. To run the project:

* start the server with:`sudo mongod --dbpath ~/data/db`
* change directories into the `game-loop` directory
* run `cargo run --bin db_fill` to preload players into the data base
* start the program with `cargo run`

### Cleaning the Database
* to remove the pre-loaded players run `cargo run --bin db_clean`
