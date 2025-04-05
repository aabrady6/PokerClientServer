# Poker Project Balatro
Aaron Brady, Jason Gillanders, Francis Doll

---

# Overview
This project is a server/client based poker game, which supports 3 varieties of games, and tracks player and game statistics. This project can be interacted via a GUI or CLI interface to play games on the same network.

Upon signing in, users can:
* Create or join a table for the following poker variants:
    * 5 Card Draw
    * 7 Card Stud
    * Texas Hold'em
* Spectate a game
* View individual player stats
* View past game stats

---

# Features

#### User Authentication
Users can create a new account or sign into an existing account.
These accounts are password protected, with the passwords hashed for safety and security.
Player statistics are stored in the database for persistence across game sessions.

#### Game Modes
The current dealer chooses the game variety to play. At the end of each game, the dealer button will be rotated. The games currently supported are:
* Five Card Draw
* Seven Card Stud
* Texas Hold'em

#### Spectator Mode
Users can spectate each game and view all active players hands, like watching a professional poker game on TV.

#### Player Stats
Players are able to look at all player stats that have registered.
Stats are updated at the end of each game and persist onto different sessions. These stats can be viewed during a game, as well as when spectating.

#### Game Stats
Players are able to look at all player stats that have registered.
Stats are updated at the end of each game and persist onto different sessions. These stats can be viewed during a game, as well as when spectating.

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
Run `ipconfig` or `ip a` to get the ip address of the server laptop. Ensure that the server is running on a network that you can control. It is recommended to use a hotspot for easier control of the network.

### 3. Start the Rust Server
```
cd game-loop
cargo run
```

### 4. Available Commands while the Server is Running
The admin is able to reset all game and player stats using stdin on the server terminal. Available commands:

* `reset` - resets all game and player stats. Resets the GameID to 0.
* `stop` - shuts down the server.

However, the `reset` command should only be used while the table is empty, and no games are currently being played.

## Client Steps
### 1. Set the Server IP Address to Join
In `*/client_gui/poker_gui/`, create a .env file with the following variables, replacing xxx.xx.xx.xx with the actual server IP address:
```
VITE_RUST_SERVER_IP=xxx.xx.xx.xx
VITE_RUST_SERVER_PORT=8080
VITE_NODE_SERVER_IP=localhost
VITE_NODE_SERVER_PORT=3000
```
To play locally on a single machine, set the `VITE_RUST_SERVER_IP=localhost`. For the clients connectivity, ensure they are on the same local network as the server. It is recommended to use a hotspot to ensure they are on the same network.

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
Navigate to the client code in the project and run the application:
```
cd game-loop/client_cli
cargo run
```

This will open the client application in the current terminal window. Follow the prompts on the screen to play poker!

