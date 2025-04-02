use serde::{Deserialize, Serialize};

use core::fmt;
use std::str::FromStr;

/// The value of a playing card
#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum Value {
    Empty = 0,
    AceLow = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

impl Value {
    /// Used to iterate over the Value enum
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

impl TryFrom<char> for Value {
    type Error = &'static str;

    /// Converts a character representation of the value into a type Value
    ///
    /// # Argument
    /// * `value` - a character representation of the card value. Valid inputs 1..=9, T, J, Q, K, A
    ///
    /// # Returns
    /// * `Ok(Value)` - a type Value of the value
    /// * `Err` - invalid string input
    ///
    /// # Example
    /// ```rust
    ///  let value = Value::try_from('Q').unwrap();
    ///  assert_eq!(value, Value::Queen);
    ///    
    /// ```
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '0' => Ok(Value::Empty),
            '1' => Ok(Value::AceLow),
            '2' => Ok(Value::Two),
            '3' => Ok(Value::Three),
            '4' => Ok(Value::Four),
            '5' => Ok(Value::Five),
            '6' => Ok(Value::Six),
            '7' => Ok(Value::Seven),
            '8' => Ok(Value::Eight),
            '9' => Ok(Value::Nine),
            'T' => Ok(Value::Ten),
            'J' => Ok(Value::Jack),
            'Q' => Ok(Value::Queen),
            'K' => Ok(Value::King),
            'A' => Ok(Value::Ace),
            _ => Err("not a valid card char, must be between 2-9,T,J,Q,K,A"),
        }
    }
}

impl TryFrom<u8> for Value {
    type Error = &'static str;

    /// Converts a character representation of the value into a type Value
    ///
    /// # Argument
    /// * `value` - a character representation of the card value. Valid inputs 1..=9, T, J, Q, K, A
    ///
    /// # Returns
    /// * `Ok(Value)` - a type Value of the value
    /// * `Err` - invalid string input
    ///
    /// # Example
    /// ```rust
    ///  let value = Value::try_from('Q').unwrap();
    ///  assert_eq!(value, Value::Queen);
    ///    
    /// ```
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Value::Empty),
            1 => Ok(Value::AceLow),
            2 => Ok(Value::Two),
            3 => Ok(Value::Three),
            4 => Ok(Value::Four),
            5 => Ok(Value::Five),
            6 => Ok(Value::Six),
            7 => Ok(Value::Seven),
            8 => Ok(Value::Eight),
            9 => Ok(Value::Nine),
            10 => Ok(Value::Ten),
            11 => Ok(Value::Jack),
            12 => Ok(Value::Queen),
            13 => Ok(Value::King),
            14 => Ok(Value::Ace),
            _ => Err("not a valid card val, must be between 2-14"),
        }
    }
}

impl TryFrom<Value> for char {
    type Error = &'static str;

    /// Converts a type Value into the character representation
    ///
    /// # Argument
    /// * `value` - type Value of card.
    ///
    /// # Returns
    /// * `Ok(char)` - the character representation of the value
    ///
    /// # Example
    /// ```rust
    /// let val = Value::Ace;
    /// let val_char = char::try_from(val).unwrap();
    /// assert_eq!(val_char, 'A');
    /// ```
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Empty => ' ',
            Value::AceLow => '1',
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
        })
    }
}

/// Represents the suit of a card
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum Suit {
    Club = 0,
    Diamond = 1,
    Heart = 2,
    Spade = 3,
    Empty = 4,
}

impl Suit {
    /// Used to iterate over the suit enum
    pub fn iterator() -> impl Iterator<Item = Suit> {
        static SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Diamond, Suit::Heart];
        SUITS.iter().copied()
    }
}

impl TryFrom<char> for Suit {
    type Error = &'static str;

    /// Converts a character representation of the suit into a type Suit
    ///
    /// # Argument
    /// * `suit` - a character representation of the card suit. Valid inputs S, C, D, H
    ///
    /// # Returns
    /// * `Ok(Suit)` - a type suit of the card's suit
    /// * `Err` - invalid string input
    ///
    /// # Example
    /// ```rust
    /// let suit = Suit::try_from('S').unwrap();
    /// assert_eq!(suit, Suit::Spade);
    /// ```
    fn try_from(suit: char) -> Result<Self, Self::Error> {
        match suit {
            'S' => Ok(Suit::Spade),
            'C' => Ok(Suit::Club),
            'D' => Ok(Suit::Diamond),
            'H' => Ok(Suit::Heart),
            _ => Err("not a valid suit char, must be between S,C,D,H"),
        }
    }
}

impl TryFrom<Suit> for char {
    type Error = &'static str;

    /// Converts a type Suit into the character representation
    ///
    /// # Argument
    /// * `suit` - type Suit of card.
    ///
    /// # Returns
    /// * `Ok(char)` - the character representation of the suit
    ///
    /// # Example
    /// ```rust
    ///  let suit = Suit::Heart
    ///  let suit_char = char::try_from(suit).unwrap();
    ///  assert_eq!(suit_char, 'H');
    /// ```
    fn try_from(suit: Suit) -> Result<Self, Self::Error> {
        Ok(match suit {
            Suit::Spade => 'S',
            Suit::Club => 'C',
            Suit::Diamond => 'D',
            Suit::Heart => 'H',
            Suit::Empty => 'E',
        })
    }
}

/// Playing card object with a value and suit
#[derive(Debug, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Card {
    /// Value of the card in type `Value` ie Value::Six
    pub value: Value,
    /// Suit of the card in type `Suit` ie Suit::Heart
    pub suit: Suit,
    /// True - face up, False - face down
    pub face_up: bool,
}

impl Card {
    /// Constructor for a new Card
    ///
    /// # Arguments
    /// * `value` - type Value of the card
    /// * `suit` - type Suit of the card
    ///
    /// # Example
    /// ```rust
    /// let val: Value = Value::Five;
    /// let suit: Suit = Suit::Spade;
    ///
    /// let c = match Card::new(val, suit) {
    ///     Ok(card) => card,
    ///     Err(_) => panic!("Failed to create a card"),
    /// };
    ///
    /// assert_eq!(c.get_value_as_u8, 5);
    /// assert_eq!(c.get_suit, Suit::Spade);
    /// ```
    pub fn new(value: Value, suit: Suit) -> Result<Self, &'static str> {
        Ok(Card {
            value,
            suit,
            face_up: false,
        })
    }

    /// Retrieves the suit of the card.
    ///
    /// # Returns
    /// The `Suit` of the card.
    ///
    /// # Examples
    /// ```rust
    /// let card_string2 = "AS";
    /// let c2: Card = card_string2.parse().unwrap();
    /// assert_eq!(c2.get_suit(), Suit::Spade);
    /// ```
    pub fn get_suit(self: Card) -> Suit {
        self.suit
    }

    /// Retrieves the value of the card
    ///
    /// # Returns
    /// The value of the card.
    ///
    /// # Examples
    /// ```rust
    ///let card_string2 = "AS";
    ///let c2: Card = card_string2.parse().unwrap();
    /// assert_eq!(c2.get_value, Value::Ace);
    /// ```
    pub fn get_value(self: Card) -> Value {
        self.value
    }

    /// Retrieves the value of the card in u8
    ///
    /// # Returns
    /// The numeric value of the card.
    ///
    /// # Examples
    /// ```rust
    ///let card_string2 = "AS";
    ///let c2: Card = card_string2.parse().unwrap();
    /// assert_eq!(c2.get_value_as_u8(), 14);
    /// ```
    pub fn get_value_as_u8(self: Card) -> u8 {
        self.value as u8
    }

    /// Tests if the values of two cards are equal, ignores the suit
    ///
    /// # Returns
    /// `true` - values are the same
    /// `false` - values are different
    ///
    /// # Examples
    /// ```rust
    /// let c1: Card = "AH".parse().unwrap();
    /// let c2: Card = "AS".parse().unwrap();
    ///
    /// assert_eq!(c1.is_value_equal(&c2), true);
    ///
    /// let c3: Card = "4D".parse().unwrap();
    /// assert_eq!(c1.is_value_equal(&c3), false);
    /// ```
    pub fn is_value_equal(&self, other: &Card) -> bool {
        self.value == other.value
    }
}

/// Implements `PartialOrd` to allow ordering of `Card` instances.
impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Implements `Ord` to define a total ordering for `Card` instances based on value.
impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_value_as_u8().cmp(&other.get_value_as_u8())
    }
}

/// Implements `PartialEq` to compare two `Card` instances for equality.
impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.suit == other.suit
    }
}

/// Implements `FromStr` to enable parsing a `Card` from a string.
impl FromStr for Card {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err("Invalid Card input format");
        }

        let val_char = s.chars().nth(0).unwrap();
        let suit_char = s.chars().nth(1).unwrap();

        let value = Value::try_from(val_char)?;
        let suit = Suit::try_from(suit_char)?;

        Ok(Card {
            value,
            suit,
            face_up: false,
        })
    }
}

/// Implements `Display` to format a `Card` as a string.
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_char = char::try_from(self.value).unwrap();
        let suit_char = char::try_from(self.suit).unwrap();
        write!(f, "{}{}", value_char, suit_char)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_constructor() {
        let val: Value = Value::Five;
        let suit: Suit = Suit::Heart;

        let c = match Card::new(val, suit) {
            Ok(card) => card,
            Err(_) => panic!("Failed to create a valid card"),
        };
        assert_eq!(c.get_value_as_u8(), 5);
        assert_eq!(c.get_suit(), Suit::Heart);
    }

    #[test]
    fn test_card_constructor_from_string() {
        let card_string = "9H";
        let c: Card = card_string.parse().unwrap();
        assert_eq!(c.get_value_as_u8(), 9);
        assert_eq!(c.get_suit(), Suit::Heart);

        let card_string2 = "AS";
        let c2: Card = card_string2.parse().unwrap();
        assert_eq!(c2.get_value_as_u8(), 14);
        assert_eq!(c2.get_suit(), Suit::Spade);

        let card_string3 = "QC";
        let c3: Card = card_string3.parse().unwrap();
        assert_eq!(c3.get_value_as_u8(), 12);
        assert_eq!(c3.get_suit(), Suit::Club);

        let card_string4 = "2D";
        let c4: Card = card_string4.parse().unwrap();
        assert_eq!(c4.get_value_as_u8(), 2);
        assert_eq!(c4.get_suit(), Suit::Diamond);
    }

    #[test]
    fn test_card_invalid_length() {
        let card_string = "ABC";
        let c = card_string.parse::<Card>();
        assert!(c.is_err());
        assert_eq!(c.unwrap_err(), "Invalid Card input format");
    }

    #[test]
    fn test_card_constructor_from_str_not_equal() {
        let card_string = "QH";
        let c: Card = card_string.parse().unwrap();
        assert_ne!(c.value, Value::Five);
        assert_ne!(c.suit, Suit::Diamond);
    }

    #[test]
    fn test_card_print() {
        let card_string = "5S";
        let c: Card = card_string.parse().unwrap();
        assert_eq!(c.to_string(), "5S");
    }

    #[test]
    fn test_card_constructor_typed() {
        let c = Card {
            value: Value::Ace,
            suit: Suit::Spade,
            face_up: false,
        };
        assert_eq!(c.value, Value::Ace);
        assert_eq!(c.suit, Suit::Spade);
    }

    #[test]
    fn test_card_char_from_value() {
        assert_eq!(char::try_from(Value::Two).unwrap(), '2');
        assert_eq!(char::try_from(Value::King).unwrap(), 'K');
    }

    #[test]
    fn test_card_value_from_char() {
        let val = Value::try_from('J').unwrap();
        assert_eq!(val, Value::Jack);
    }

    #[test]
    fn test_card_value_from_char_invalid() {
        let val = Value::try_from('P');
        assert!(val.is_err());
        assert_eq!(
            val.unwrap_err(),
            "not a valid card char, must be between 2-9,T,J,Q,K,A"
        );
    }

    #[test]
    fn test_card_suit_from_char() {
        let s = Suit::try_from('C').unwrap();
        assert_eq!(s, Suit::Club);
    }

    #[test]
    fn test_card_suit_from_char_invalid() {
        let s = Suit::try_from('Z');
        assert!(s.is_err());
        assert_eq!(
            s.unwrap_err(),
            "not a valid suit char, must be between S,C,D,H"
        );
    }

    #[test]
    fn test_card_value() {
        let card_string = "TC";
        let c: Card = card_string.parse().unwrap();
        assert_eq!(c.get_value(), Value::Ten);
    }

    #[test]
    fn test_card_value_to_u8() {
        let card_string = "TC";
        let c: Card = card_string.parse().unwrap();
        assert_eq!(c.get_value_as_u8(), 10);
    }

    #[test]
    fn test_card_comparison() {
        let card_string = "AS";
        let c1: Card = card_string.parse().unwrap();

        let card_string2 = "KH";
        let c2: Card = card_string2.parse().unwrap();

        let card_string3 = "9C";
        let c3: Card = card_string3.parse().unwrap();

        let card_string4 = "AS";
        let c4: Card = card_string4.parse().unwrap();

        assert!(c1 > c2);
        assert!(c3 < c2);
        assert!(c1 == c4);
        assert!(c2 != c3);
    }

    #[test]
    fn test_card_compare_values() {
        let c1: Card = "AS".parse().unwrap();
        let c2: Card = "AH".parse().unwrap();
        assert_eq!(c1.is_value_equal(&c2), true);

        let c3: Card = "4S".parse().unwrap();
        assert_eq!(c1.is_value_equal(&c3), false);
    }

    #[test]
    fn test_card_clone() {
        let card_string = "7D";
        let c1: Card = card_string.parse().unwrap();
        let card_clone = c1.clone();
        assert_eq!(c1, card_clone);
    }

    #[test]
    fn test_card_copy() {
        let card_string = "7D";
        let c1: Card = card_string.parse().unwrap();
        let card_copy = c1;
        assert_eq!(c1, card_copy);
    }
}
