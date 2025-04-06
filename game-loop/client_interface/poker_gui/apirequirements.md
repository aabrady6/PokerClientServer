# Server Sends/Client Receives:
- Game state
    - Information about each player:
        - name
        - total cash
        - current amount bet
        - face up cards (if applicable)
        - face down cards
        - last action taken
        - token (SB, BB, D) (if applicable)
    - Turn-based actions
        - Which player's turn it is
        - What action the player is being prompted to do (choose cards, make betting choice, etc)
            - Give an array of all possible actions for the given state
                - For each action, state if the action requires a value, and if so, the range of the acceptable vlaues
    - Community cards (if applicable)
    - Current money in the pot
    - Possibly the history of all actions taken in this game

# Client Sends/Server Receives:
- Player name for logging in
- Cards to replace (5-card draw)
- Betting options (may include a value)
    - Fold
    - Call
    - Check
    - Raise (with a value)
    - All in
- Leave table

# API Endpoints
- `/ws`
    - ### Description:
        - Sets up a WebSocket connection between the client and server.
    - ### Inputs:
        - `req`: `actix_web::HttpRequest`
        - `stream`: `web::Payload`
        - `data`: `web::Data<AppState>`
    - ### Outputs:
        - `Result<HttpResponse, Error>`

- `/register/{player_name}`
    - ### Description:
        - Adds a player to the lobby/game.
    - ### Inputs:
        - `player_name`: `web::Path<String>`
        - `data`: `web::Data<AppState>`
    - ### Outputs:
        - `Result<HttpResponse, Error>`

- `/game`
    - ### Description:
        - Gets the current game state to send to the clients on request.
    - ### Inputs:
        - `data`: `web::Data<AppState>`
    - ### Outputs:
        - `Result<HttpResponse, Error>`

- `/handle_action/{player_name}`
    - ### Description:
        - Handles the betting action of a given player. Involves checking if the action is valid, and if it requires a value, ensures the value is valid as well.
    - ### Inputs:
        - `data`: `web::Data<AppState>`
        - `player_name`: `web::Path<String>`
        - `action`: `web::Json<String>`
        - `value`: `web::Json<String>`
    - ### Outputs:
        - `Result<HttpResponse, Error>`

- `/redraw_cards/{player_name}`
    - ### Description:
        - Receives the cards to remove and replace from a given player's hand.
    - ### Inputs:
        - `data`: `web::Data<AppState>`
        - `player_name`: `web::Path<String>`
        - `cards`: `web::Json<Vec<String>>`
    - ### Outputs:
        - `Result<HttpResponse, Error>`

- `/leave_table/{player_name}`
    - ### Description:
        - Removes a player from the lobby/game.
    - ### Inputs:
        - `player_name`: `web::Path<String>`
        - `data`: `web::Data<AppState>`
    - ### Outputs:
        - `Result<HttpResponse, Error>`