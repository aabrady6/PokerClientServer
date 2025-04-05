## Installation Instructions
Ensure Rust, Cargo, MongoDB, and Node are installed on your machine. For further instructions, visit the following links:
* [Rust](https://www.rust-lang.org/tools/install)
* [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
* [MongoDB](https://www.mongodb.com/try/download/community-edition)
* [Node](https://nodejs.org/en/download)

Clone this repository:
- using SSH: `git clone git@github.com:ece421-2025/poker-client-project-balatro2.git`
- using HTTPS: `git clone https://github.com/ece421-2025/poker-client-project-balatro2.git`

Navigate to the main project directory: `cd poker-client-project-balatro2`

## Server Setup and Deployment

### Setting up the Database for the Server

The poker server requires a MongoDB database to track game and player statistics. To correctly set up and deploy the database, follow these steps:

Create a directory to hold the database. For example, use this command to create the database directory in the home directory (using Unix-based operating system): `cd ~`, `sudo mkdir -p data/db`.

Run the database using `sudo mongod --dbpath ~/data/db`, and ensure that the database is running correctly.

To empty the database of all games and players, use the command `cargo run --bin db_empty` in the main project directory.

### Running the Server

To run the server, ensure you are in the correct directory: `cd game-loop`.
Then, run the server using `cargo run`. This may take a few moments to fully build and run the project.

Once the server is up and running, clients are free to start connecting to it. There are also two commands available to server admins:

* `reset` - resets all game and player stats. Resets the GameID to 0.
* `stop` - shuts down the server.

However, the `reset` command should only be used while the table is empty, and no games are currently being played.

## Running a Client

There are two different ways to connect to the poker server: a CLI and a GUI. Below, connecting using either client is described. However, one step is required for either client first.

The IP address of the server must be found and communicated to the clients. A server administrator must find their local IP address by running the appropriate command. Run `ipconfig` or `ip a` to get the IP address of the server machine. Ensure that the server is running on a network that you can control. It is recommended to use a hotspot for easier control of the network.

This IP address must be shared to clients. Then, the client must make a file called `.env` in their subsequent client directory. The following information must be put in the file (with the server IP address replacing the x's here):
```
VITE_RUST_SERVER_IP=xxx.xxx.xxx.xxx
VITE_RUST_SERVER_PORT=8080
VITE_NODE_SERVER_IP=localhost
VITE_NODE_SERVER_PORT=3000
```

For troubleshooting, ensure that the client and server are connected to the same local network. You can also check that the server is running correctly by pinging the server. For example:
`ping xxx.xxx.xxx.xxx:8080`.

### Using the Client GUI

To use the client GUI, you will need to have Node installed on your machine. Refer to the start of this user manual to learn how to download it.

Running the GUI requires two terminal windows. In the first window, navigate to the correct folder and run the command to start the connectivity script:
```
cd game-loop/client_gui/poker_gui/
node server.js
```

Then, in the second terminal window, navigate to the same folder and start up the GUI front end:
```
cd game-loop/client_gui/poker_gui/
npm run dev
```
This will open the front end at the address `http://localhost:5173/`. Navigate to this address using your preferred web browser to connect.

### Using the Client CLI

To use the client CLI, you will need Rust and Cargo installed on your machine. Refer to the start of this user manual to learn how to download them.

Running the CLI requires one terminal window. Navigate to the correct folder and run the command to start the client:
```
cd game-loop/client_cli/
cargo run
```

## Playing Poker using a Client

This guide will help walk you through playing any variant of poker on either of the client options. For variant specific rules and help, access the Help menus of either client.

### Login and Registering
On first loading of the client, you will be prompted to log in or register a new account. A username and password are required. Passwords are hashed and stored securely in the database.

### Waiting Room
Once logged in, a player will be greeted by the welcome screen. If they are the first player there and/or there are no games currently being played, they are considered the "dealer" for the table, and have more responsibility. They have three choices: join the table, spectate the game, or leave the table. If they join the table, they will be asked which game variant to play. Once selected, they will wait until enough players are at the table to start the game. The game variants to choose from are: Five Card Draw, Seven Card Stud, and Texas Hold ‘Em.

If a player is not the first player (and thus not the dealer), they see the same options, except for the choice of game variant. They will be forced to play whatever game the dealer chose.

In addition, once logged in, a player can choose to access the help menu and stats menu at any time. The help menu displays the house rules and how to play each variant of poker. The stats menu shows the player both player stats and game stats. These stats can be accessed both outside of gameplay, and within a game.

### Choosing player roles
Once a player has chosen a given poker variant to play, the dealer will start the game when there are enough players to play the game. In all cases for this application, there must be at least two (2) players at a table to start the game. Once the game has started, the player will be faced with various options, depending on the variant. However, there are some actions that are present regardless of the poker variant. Firstly, a “dealer” will be chosen, and that player will be designated with a dealer chip. After the dealer is randomly selected, the next two players to the “left” of the dealer are 
considered the small and big blinds, respectively.

### The Betting Round
After the blinds are chosen, the player to the left of the big blind will be given the standard betting options. They can: fold, and sit out the rest of that round; call, and match the largest current bet amount of the betting round; or raise, and bet an additional amount of chips to the current call amount. (Note: the raise amount is the *total* amount the player wants to place in the pot. So by raising $10, they choose to set their amount to $10). If a player has already matched the current maximum bet amount, they can also check, which does not add any additional chips to the pot, but keeps the player in the game. Depending on how the client is using the software, these inputs will either be typed into the terminal via a CLI, or physically chosen using the GUI.

### Playing the rest of the game
Once all players have matched the amount they will add to the pot, the game will move onto the next round. However, there is one exception: if all but one player have folded, then the remaining player will win all the chips in the pot, and the game ends. What this round entails depends on the variant being played. This may involve drawing new cards to your hand, seeing community cards, or swapping out cards in your hand for new ones. There may also be additional betting rounds interspersed within these other actions. These betting rounds are the same as what is described above, except that the betting round now begins with the first player to the big blind's left. If by the end of the final betting round there are still at least two players remaining, the game moves on to the showdown.

### The Showdown
If there are multiple players remaining in the game after the final betting round, the showdown begins. Each player will show their hand, and the player that can create the best five-card hand wins the chips in the pot. If there is a tie in the power level of their hands, then each player that tied will split the pot evenly between them. You can find the rankings of the poker hands on cardplayer.com.

### After a game
When the winning player(s) has (have) collected the pot, a player can choose to leave the table and the subsequent games. This allows new players to join the table, especially if that table was full previously. A player may stay as long as they wish to at a given table. House rules allow for if a user loses all money, they get a $1000 cash deposit.
