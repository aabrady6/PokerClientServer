use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use rand::{seq::SliceRandom, thread_rng};

/// Represents the value of a playing card.
/// 
/// The enum includes all standard playing card values from Two through Ace,
/// plus special variants for low Ace and Empty (no value).
#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum Value {
    /// Represents an empty card value
    Empty = 0,
    /// Represents an Ace that counts as low (value 1)
    AceLow = 1,
    /// Represents the card value 2
    Two = 2,
    /// Represents the card value 3
    Three = 3,
    /// Represents the card value 4
    Four = 4,
    /// Represents the card value 5
    Five = 5,
    /// Represents the card value 6
    Six = 6,
    /// Represents the card value 7
    Seven = 7,
    /// Represents the card value 8
    Eight = 8,
    /// Represents the card value 9
    Nine = 9,
    /// Represents the card value 10
    Ten = 10,
    /// Represents the Jack (value 11)
    Jack = 11,
    /// Represents the Queen (value 12)
    Queen = 12,
    /// Represents the King (value 13)
    King = 13,
    /// Represents an Ace that counts as high (value 14)
    Ace = 14,
}

impl Value {
    /// Converts a card value to its character representation.
    ///
    /// # Returns
    /// A character representing the card value (e.g., 'A' for Ace, '2' for Two, etc.)
    pub fn conv_to_char(&self) -> char {
        match self {
            Value::Empty => ' ',
            Value::AceLow => 'A',
            Value::Two => '2',
            Value::Three => '3',
            Value::Four => '4',
            Value::Five => '5',
            Value::Six => '6',
            Value::Seven => '7',
            Value::Eight => '8',
            Value::Nine => '9',
            Value::Ten => 'T',
            Value::Jack => 'J',
            Value::Queen => 'Q',
            Value::King => 'K',
            Value::Ace => 'A',
        }
    }

    /// Creates an iterator over all standard card values (Two through Ace).
    ///
    /// # Returns
    /// An iterator yielding each standard card value
    pub fn iterator() -> impl Iterator<Item = Value> {
        static VALUES: [Value; 13] = [
            Value::Two,
            Value::Three,
            Value::Four,
            Value::Five,
            Value::Six,
            Value::Seven,
            Value::Eight,
            Value::Nine,
            Value::Ten,
            Value::Jack,
            Value::Queen,
            Value::King,
            Value::Ace,
        ];
        VALUES.iter().copied()
    }
} 

impl Card {
    /// Converts a card to its string representation.
    ///
    /// # Parameters
    /// * `all_up` - If true, shows all cards face up regardless of their `face_up` status
    ///
    /// # Returns
    /// A string representation of the card (e.g., "AS" for Ace of Spades, "##" for face down)
    #[allow(unused_assignments)]
    pub fn to_string(&self, all_up: bool) -> String {
        let mut card_string = "".to_string();
        if !self.face_up && !all_up {
            return "##".to_string();
        }
        match self.value {
            Value::Empty => return "".to_string(),
            Value::AceLow => card_string = "A".to_string(),
            Value::Two => card_string = "2".to_string(),
            Value::Three => card_string = "3".to_string(),
            Value::Four => card_string = "4".to_string(),
            Value::Five => card_string = "5".to_string(),
            Value::Six => card_string = "6".to_string(),
            Value::Seven => card_string = "7".to_string(),
            Value::Eight => card_string = "8".to_string(),
            Value::Nine => card_string = "9".to_string(),
            Value::Ten => card_string = "T".to_string(),
            Value::Jack => card_string = "J".to_string(),
            Value::Queen => card_string = "Q".to_string(),
            Value::King => card_string = "K".to_string(),
            Value::Ace => card_string = "A".to_string(),
        }

        match self.suit {
            Suit::Club => card_string = format!("{}C", card_string),
            Suit::Diamond => card_string = format!("{}D", card_string),
            Suit::Heart => card_string = format!("{}H", card_string),
            Suit::Spade => card_string = format!("{}S", card_string),
            Suit::Empty => return "".to_string(),
        }
        return card_string;
    }
}

/// Represents the suit of a playing card.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum Suit {
    /// Club suit (♣)
    Club = 0,
    /// Diamond suit (♦)
    Diamond = 1,
    /// Heart suit (♥)
    Heart = 2,
    /// Spade suit (♠)
    Spade = 3,
    /// Empty/no suit
    Empty = 4,
}

impl Suit {
    /// Creates an iterator over all standard card suits.
    ///
    /// # Returns
    /// An iterator yielding each standard card suit
    pub fn iterator() -> impl Iterator<Item = Suit> {
        static SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Diamond, Suit::Heart];
        SUITS.iter().copied()
    }
}

/// Represents a playing card with a value, suit, and face-up status.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Card {
    /// Value of the card (e.g., Value::Six)
    pub value: Value,
    /// Suit of the card (e.g., Suit::Heart)
    pub suit: Suit,
    /// Whether the card is face up (true) or face down (false)
    pub face_up: bool,
}

impl Card {
    /// Creates a new card with the specified value and suit.
    ///
    /// # Parameters
    /// * `value` - The value of the card
    /// * `suit` - The suit of the card
    ///
    /// # Returns
    /// A Result containing either the new Card or an error message
    pub fn new(value: Value, suit: Suit) -> Result<Self, &'static str> {
        Ok(Card {
            value,
            suit,
            face_up: false,
        })
    }
}

/// Represents a hand of cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hand {
    /// Vector of the cards that are in the hand
    pub cards: Vec<Card>,
    /// Maximum number of cards that can be in the hand
    hand_limit: u8,
}

impl Hand {
    /// Creates a new empty hand with the specified card limit.
    ///
    /// # Parameters
    /// * `n` - The maximum number of cards allowed in the hand
    ///
    /// # Returns
    /// A new Hand instance
    pub fn new(n: u8) -> Self {
        let cards = Vec::new();
        let hand_limit = n;
        Self { cards, hand_limit }
    }
}

/// Represents the possible actions a player can take during their turn.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlayerChoice {
    /// Player checks (passes without betting)
    Check,
    /// Player folds (gives up their hand)
    Fold,
    /// Player calls the current bet amount
    Call(u32),
    /// Player raises the bet by the specified amount
    Raise(u32),
    /// Player places an initial bet of the specified amount
    Bet(u32),
    /// Player has placed the specified amount in the pot
    PlacedInPot(u32),
    /// Player bets all their remaining chips
    AllIn,
}

/// Represents a player in the poker game, including their current state and historical stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique identifier for the player
    pub player_id: u32,
    /// Display name of the player
    pub player_name: String,
    /// Securely stored password hash
    pub hashed_password: String,
    /// Current amount of money the player has
    pub player_money: u32,
    /// Amount won in the current round
    pub round_win: u32,
    /// Description of the player's most recent action
    pub last_move: String,
    /// Total number of games played
    pub total_games: u32,
    /// Total number of games won
    pub total_wins: u32,
    /// Total number of games lost
    pub total_losses: u32,
    /// Lifetime earnings
    pub total_earnings: u32,
    /// Total amount wagered in the current game
    pub total_wagered_per_game: u32,
    /// The player's current hand of cards
    pub player_hand: Hand,
    /// Authentication token for the player
    pub token: String,
    /// Map of game ids to player choices made in those games
    pub player_choices: HashMap<String, PlayerChoice>,
}

impl Player {
    /// Creates a new empty player with default values.
    ///
    /// # Returns
    /// A new Player instance with default values
    pub fn empty() -> Self {
        Self {
            player_id: 0,
            player_name: String::new(),
            hashed_password: String::new(),
            player_money: 0,
            round_win: 0,
            total_games: 0,
            total_wins: 0,
            total_losses: 0,
            total_earnings: 0,
            last_move: "".to_string(),
            total_wagered_per_game: 0,
            token: "".to_string(),
            player_hand: Hand::new(0),
            player_choices: HashMap::new(),
        }
    }
}

/// Represents the current state of a poker game.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameState {
    /// The variant of poker being played (e.g. "Texas Hold'em")
    pub game_variant: String,
    /// The deck of cards being used for the game
    pub deck: Deck,
    /// Unique identifier for the game
    pub game_id: u32,
    /// Number of cards in each player's hand
    pub hand_size: u8,
    /// Maximum number of players allowed in the game
    pub max_players: i64,
    /// Whether demo mode is active or inactive
    pub demo_mode: String,
    /// Current round number
    pub round_number: u32,
    /// Minimum allowed bet amount
    pub minimum_bet: u32,
    /// Current highest bet amount
    pub highest_bet: u32,
    /// Player whose turn it is
    pub current_player: Player,
    /// Current action being performed
    pub player_action: String,
    /// List of active players in the game
    pub players: Vec<Player>,
    /// List of players waiting to join the game
    pub lobby: Vec<Player>,
    /// List of players watching but not participating
    pub spectators: Vec<Player>,
    /// List of spectators for dealer choice games
    pub dealer_choice_spectators: Vec<Player>,
    /// List of winners and their winning hands
    pub winner: Vec<(Player, ScoredHand)>,
    /// Community cards shared by all players
    pub community_cards: Hand,
    /// Total amount of money in the pot
    pub pot: u32,
    /// Index of the dealer position
    pub dealer: u32,
    /// Whether players have been prompted to discard cards
    pub discard_cards_prompted: bool,
    /// String representation of the current action
    pub current_action_string: String,
    /// History of actions taken in the game
    pub action_history: Vec<String>,
    /// Minimum and maximum allowed raise amounts
    pub raise_min_max: (u32, u32),
}

impl Default for GameState {
    /// Creates a new GameState with default values.
    ///
    /// # Returns
    /// A new GameState instance with default values
    fn default() -> Self {
        GameState {
            players: Vec::new(),
            lobby: Vec::new(),
            spectators: Vec::new(),
            dealer_choice_spectators: Vec::new(),
            deck: Deck::new(),
            game_variant: "".to_string(),
            game_id: 0,
            hand_size: 5,
            max_players: 5,
            pot: 0,
            demo_mode: "inactive".to_string(),
            round_number: 0,
            minimum_bet: 5,
            highest_bet: 0,
            dealer: 0,
            winner: vec![],
            current_player: Player::empty(),
            player_action: String::new(),
            community_cards: Hand::new(5),
            discard_cards_prompted: false,
            current_action_string: "".to_string(),
            action_history: Vec::new(),
            raise_min_max: (0, 0),
        }
    }
}

/// Represents a standard deck of playing cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    /// Vector containing all cards in the deck
    cards: Vec<Card>,
}

impl Deck {
    /// Creates a new shuffled deck of 52 standard playing cards.
    ///
    /// # Returns
    /// A new shuffled Deck instance
    pub fn new() -> Self {
        let mut cards = Vec::new();
        for suit in Suit::iterator() {
            for value in Value::iterator() {
                let _card = match Card::new(value, suit) {
                    Ok(card) => cards.push(card),
                    Err(_) => eprintln!("Invalid card value or suit"),
                };
            }
        }
        let mut rng = thread_rng();
        cards.shuffle(&mut rng);
        Self { cards }
    }
}

/// Represents poker hand rankings with associated card values.
/// 
/// Each variant contains a 5-card array of values that represents the
/// cards that make up the hand, sorted in order of importance for
/// determining the winner in case of ties.
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum ScoredHand {
    /// A straight flush (five sequential cards of the same suit)
    StraightFlush([Value; 5]),
    /// Four cards of the same value plus one kicker
    FourOfAKind([Value; 5]),
    /// Three cards of one value and two cards of another value
    FullHouse([Value; 5]),
    /// Five cards of the same suit
    Flush([Value; 5]),
    /// Five sequential cards of mixed suits
    Straight([Value; 5]),
    /// Three cards of the same value plus two kickers
    ThreeOfAKind([Value; 5]),
    /// Two cards of one value, two cards of another value, plus one kicker
    TwoPair([Value; 5]),
    /// Two cards of the same value plus three kickers
    OnePair([Value; 5]),
    /// Five unmatched cards, ranked by high card
    HighCard([Value; 5]),
}

impl fmt::Display for ScoredHand {
    /// Formats a ScoredHand for display.
    ///
    /// # Parameters
    /// * `f` - Formatter
    ///
    /// # Returns
    /// A Result indicating whether the formatting was successful
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (hand_name, hand_values) = match *self {
            ScoredHand::StraightFlush(values) => ("Straight Flush", values),
            ScoredHand::FourOfAKind(values) => ("Four of a Kind", values),
            ScoredHand::FullHouse(values) => ("Full House", values),
            ScoredHand::Flush(values) => ("Flush", values),
            ScoredHand::Straight(values) => ("Straight", values),
            ScoredHand::ThreeOfAKind(values) => ("Three of a Kind", values),
            ScoredHand::TwoPair(values) => ("Two Pair", values),
            ScoredHand::OnePair(values) => ("One Pair", values),
            ScoredHand::HighCard(values) => ("High Card", values),
        };

        let formatted_values: Vec<String> = hand_values
            .iter()
            .map(|&v| String::from(v.conv_to_char()))
            .collect();
        write!(f, "{}: {}", hand_name, formatted_values.join(", "))
    }
}