use crate::game::card::{Card, Suit, Value};
use serde::{Deserialize, Serialize};

use rand::{seq::SliceRandom, thread_rng};

/// Deck object composed of cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    /// A vector of the cards in the deck
    cards: Vec<Card>,
}

impl Deck {
    /// Creates a new shuffled standard deck of 52 card_str
    ///
    /// # Returns
    /// A new `deck` object with 52 shuffled cards
    ///
    /// # Example
    /// ```rust
    /// let deck = Deck::new()
    /// assert_eq!(deck.count(), 52);
    /// ```
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

    /// Gives the current count of the number of cards in the deck
    ///
    /// # Returns
    /// Number of cards in the deck as u8
    ///
    /// # Example
    /// ```rust
    /// let deck = Deck::new();
    /// assert_eq!(deck.count(), 52);
    ///
    /// _c: Card = deck.deal_card();
    /// assert_eq!(deck.count(), 51);
    /// ```
    pub fn count(&self) -> u8 {
        self.cards.len() as u8
    }

    /// Checks if a card is currently in the deck_length
    ///
    /// # Returns
    /// * `true` - card exists in the deck
    /// * `false` - card does not exist in the deck
    ///
    /// # Example
    /// ```rust
    /// let deck = Deck::new();
    /// let card: Card = "AS".parse().unwrap();
    ///
    /// assert_eq!(deck.contains(&card), true);
    /// ```
    pub fn contains(&self, c: Card) -> bool {
        self.cards.contains(&c)
    }

    /// Deals a new card, and removes it from the deck. Will return an error if the size of the
    /// deck is 0.
    ///
    /// # Returns
    /// * `Ok(Card)` - card that is removed from the deck
    /// * `Err` - unable to deal a card
    ///
    /// # Example
    /// ```rust
    /// let deck = Deck::new();
    /// let card: Card = deck.deal_card().unwrap();
    /// ```
    pub fn deal_card(&mut self) -> Result<Card, &'static str> {
        if let Some(card) = self.cards.pop() {
            Ok(card)
        } else {
            Err("Deck is empty, unable to deal another card")
        }
    }

    /// Checks if the deck is empty. Returns true if empty.
    ///
    /// # Returns
    /// * `true` - deck is empty
    /// * `false` - deck is not is_empty
    ///
    /// # Example
    /// ```rust
    /// let deck = Deck::new();
    /// assert_eq!(deck.is_empty(), false);
    ///
    /// while !deck.is_empty() {
    ///     let _c: Card = deck.deal_card().unwrap();
    /// }
    /// assert_eq!(deck.is_empty(), true);
    /// ```
    pub fn is_empty(&self) -> bool {
        let deck_length = self.cards.len();
        if deck_length <= 0 {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_new_deck() {
        let deck = Deck::new();
        assert_eq!(deck.count(), 52);

        for suit in Suit::iterator() {
            for value in Value::iterator() {
                let c = Card::new(value, suit).expect("did not create a card");
                assert_eq!(deck.contains(c), true);
            }
        }

        // try a random value just to make sure all of the suits are created... redundant check
        let test_val = Value::Ace;

        let h = Suit::Heart;
        let c = Card::new(test_val, h).expect("did not create a card");
        assert_eq!(deck.contains(c), true);

        let s = Suit::Spade;
        let c = Card::new(test_val, s).expect("did not create a card");
        assert_eq!(deck.contains(c), true);

        let d = Suit::Diamond;
        let c = Card::new(test_val, d).expect("did not create a card");
        assert_eq!(deck.contains(c), true);

        let cl = Suit::Club;
        let c = Card::new(test_val, cl).expect("did not create a card");
        assert_eq!(deck.contains(c), true);
    }

    #[test]
    fn test_deck_deal_card() {
        let mut deck = Deck::new();
        assert_eq!(deck.count(), 52);

        let c: Card = deck.deal_card().unwrap();
        assert_eq!(deck.count(), 51);
        assert_eq!(deck.contains(c), false);
        assert_eq!(c.face_up, false);
    }

    #[test]
    fn test_deck_is_empty() {
        let mut deck = Deck::new();
        assert_eq!(deck.count(), 52);
        assert_eq!(deck.is_empty(), false);

        for _card in 0..52 {
            let _c: Card = deck.deal_card().unwrap();
        }
        assert_eq!(deck.count(), 0);
        assert_eq!(deck.is_empty(), true);
        let c_error = match deck.deal_card() {
            Ok(_) => panic!("Deck should be empty. Expected error but got Card!"),
            Err(e) => e,
        };
        assert_eq!(c_error, "Deck is empty, unable to deal another card");
    }
}
