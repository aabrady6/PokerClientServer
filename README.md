# Poker Project Balatro
Aaron Brady, Jason Gillanders, Francis Doll

---

# Overview
This project is a server/client based poker game, which supports 3 varieties of games, and tracks player and game statistics. This project can be interacted via a GUI or CLI interface to play games on the same network.

Upon signing in, user's can:
* Play a game of 5 Card Draw
* Play a game of 7 Card Stud
* Play a game of Texas Hold'em
* Spectate a game
* View induvidual player stats
* View past game stats

---

# Features

#### User Authentication
User's can create a new account or sign into an existing account.
Player statistics are stored in the database for persistence across game sessions.

#### Game Modes
The current dealer chooses the game variety to play. At the end of each game, the dealer button will be rotated.
Five Card Draw
Seven Card Stud
Texas Hold'em

#### Spectator Mode
User's can spectate each game and view all active players hands.

#### Player Stats
Players are able to look at all player stats that have registered.
Stats are updated at the end of each game and persist onto different sessions.

#### Game Stats
Players are able to look at all player stats that have registered.
Stats are updated at the end of each game and persist onto different sessions.

---

# Requirements

* [Rust](https://www.rust-lang.org/tools/install)
* [Node](https://nodejs.org/en/download)
* [MongoDB](https://www.mongodb.com/try/download/community-edition)

---

# Running Procedures

## Server Side Steps
### 1. Server Side Database
Start the MongoDB database instance on the server machine.
`sudo mongod --dbpath ~/data/db` or `mongod --no-auth`

##### Available Commands
`cargo run --bin db_empty`: empties the database of all games and players

### 2. Get the IP Address of the Server Machine
Run `ipconfig` or `ip a` to get the ip address of the server laptop.

### 3. Start the Rust Server
```
cd game-loop
cargo run
```

## Client Steps
### 1. Set the Server IP Address to Join
In `*/client_gui/poker_gui/`, create a .env file with the following variables, replacing xxx.xx.xx.xx with the actual server IP address:
```
VITE_RUST_SERVER_IP=xxx.xx.xx.xx
VITE_RUST_SERVER_PORT=8080
VITE_NODE_SERVER_IP=localhost
VITE_NODE_SERVER_PORT=3000
```
To play locally on a single machine, set the `VITE_RUST_SERVER_IP=localhost`

### 2a. Connecting via GUI
Run the following commands to install required packages, and start the client side node server
```
npm install
node server.js
```
In another terminal window, start the client side gui using:
```
npm run dev
```

Naviagte to `http://localhost:5173/` and the login screen will be visible.

### 2b. Connecting via CLI
TODO

