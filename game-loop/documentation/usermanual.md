## Installation Instructions:
Ensure Rust and Cargo are installed on your machine. For further instructions, visit the Rust install page [6].

Clone this repository:
- using SSH: `git clone git@github.com:ece421-2025/poker-project-balatro.git`
- using HTTPS: `git clone https://github.com/ece421-2025/poker-project-balatro.git`

Navigate to the main project directory: `cd poker-project-balatro`

## Build the entire project: `cargo build`
Setting up the Database for the Server:

The poker server requires a MongoDB database to track game and player statistics. To correctly set up and deploy the database, follow these steps:
Create a directory to hold the database.

For example, use this command to create the database directory in the home directory (using Unix-based operating system): `cd ~`, `sudo mkdir -p data/db`.

Run the database using `sudo mongod --dbpath ~/data/db`, and ensure that the database is running correctly.
In addition to simply running the database, you can pre-populate the database with mock data for easily testing the stats displayed in the application.

To do so, follow these steps:
To fill the database with mock data, run `cargo run --bin db_fill`. This will put 10 mock users inside of the database.

To clean the database of the mock data and users, run `cargo run --bin db_clean`.

## Running the Server for Local Game (Testing):
First, for testing purposes, the local games depend on pre-populating the database with mock users.
Ensure the database is started using  `sudo mongod --dbpath ~/data/db`.

Then, if not already done so, add mock users and data to the database using `cargo run --bin db_fill`.

Finally, start the server using the command: `cargo run --bin server-test`
This will open the server in the local terminal. The administrator can now use this terminal to execute any game of poker, controlling all of the players as a single person. This is used for local testing of the application, ensuring that the server/dealer functionality works as expected.

## Running the Server for Production:
Start the server using the command: `cargo run --bin server`.

No other actions are needed for the server; it now waits for clients to connect to it.

## Running a Client:
Start a client instance using the command: `cargo run --bin client`.

This command will attempt to make a connection to the server. On success, you will be brought to the welcome screen.

## Waiting Room:
Once connected to the server, a player will be greeted by the welcome screen. They will be given multiple choices: they can join a Five Card Draw table; a Five Card Stud table, a Texas Hold ‘Em table; check the hand-by-hand statistics from their previous game; see the statistics of other players; or search for particular players and games to see the statistics. All of these operations can be done via the CLI or GUI interfaces.

## Playing a Poker Game:

### Choosing player roles:
Once a player has chosen a given poker variant to play, the dealer will start the game when there are enough players to play the game. In all cases for this application, there must be at least two (2) players at a table to start the game. Once the game has started, the player will be faced with various options, depending on the variant. However, there are some actions that are present regardless of the poker variant. Firstly, a “dealer” will be chosen, and that player will be designated with a dealer chip. After the dealer is randomly selected, the next two players to the “left” of the dealer are 
considered the small and big blinds, respectively.

### The Betting Round:
After the blinds are chosen, the player to the left of the big blind will be given the standard betting options. They can: fold, and sit out the rest of that game; call, and match the largest current bet amount of the betting round; raise, and bet an additional amount of chips to the current call amount; or all in, and bet all of their remaining chips. If a player has already matched the current maximum bet amount, they can also check, which does not add any additional chips to the pot, but keeps the player in the game. Depending on how the client is using the software, these inputs will either be typed into the terminal via a CLI, or physically chosen using the GUI.

### Playing the rest of the game:
Once all players have matched the amount they will add to the pot, the game will move onto the next round. However, there is one exception: if all but one player have folded, then the remaining player will win all the chips in the pot, and the game ends. What this round entails depends on the variant being played. This may involve drawing new cards to your hand, seeing community cards, or swapping out cards in your hand for new ones. There may also be additional betting rounds interspersed within these other actions. These betting rounds are the same as what is described above, except that the betting round now begins with the first player to the big blind's left. If by the end of the final betting round there are still at least two players remaining, the game moves on to the showdown.

### The Showdown:
If there are multiple players remaining in the game after the final betting round, the showdown begins. Each player will show their hand, and the player that can create the best five-card hand wins the chips in the pot. If there is a tie in the power level of their hands, then each player that tied will split the pot evenly between them. You can find the rankings of the poker hands on cardplayer.com [7].

### After a game:
When the winning player(s) has (have) collected the pot, a player can choose to leave the table and the subsequent games. This allows new players to join the table, especially if that table was full previously. A player may stay as long as they wish to at a given table. House rules allow for if a user loses all money, they get a $1000 cash deposit. This is just for part 1 of the game and will be adjusted in the future release.
