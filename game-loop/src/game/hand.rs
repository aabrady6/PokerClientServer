use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::game::card::Card;
use crate::game::card::Value;
use crate::game::deck::Deck;

/// A hand object assigned to the player or the communal card pot depending on game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hand {
    /// Vector of the cards that are in the hand
    pub cards: Vec<Card>,
    /// Limit of the hand, and the number of cards that can be added to it
    hand_limit: u8,
}

impl Hand {
    /// Creates a new hand to hold cards
    ///
    /// # Arguments
    /// * `n` - the hand limit in u8
    ///
    /// # Returns
    /// A new object of `Hand`
    ///
    /// # Example
    /// ```rust
    /// let hand = Hand::new(5);
    /// ```
    pub fn new(n: u8) -> Self {
        let cards = Vec::new();
        let hand_limit = n;
        Self { cards, hand_limit }
    }

    pub fn from_cards(cards: Vec<Card>) -> Self {
        let hand_limit = cards.len() as u8;
        Self { cards, hand_limit }
    }

    /// Returns the cards of the hand
    ///
    /// # Returns
    /// The cards in the hand as a `Vec<Card>`
    ///
    /// # Example
    /// ```rust
    /// let hand = Hand::new(5);
    /// let _ = hand.draw(1);
    /// assert_eq!(hand.get_hand_limit, 5);
    /// ```
    pub fn get_hand_cards(&self) -> &Vec<Card> {
        &self.cards
    }

    /// Returns the limit size of the hand
    ///
    /// # Returns
    /// Value of hand limit in u8
    ///
    /// # Example
    /// ```rust
    /// let hand = Hand::new(5);
    /// assert_eq!(hand.get_hand_limit, 5);
    /// ```
    pub fn get_hand_limit(&self) -> u8 {
        self.hand_limit
    }

    /// Returns the current number of cards in the hand
    ///
    /// # Returns
    /// Value in u8
    ///
    /// # Example
    /// ```rust
    /// let hand = Hand::new(5);
    /// assert_eq!(hand.count(), 0);
    ///
    /// let _ = hand.draw(1);
    /// assert_eq!(hand.count(), 1);
    /// ```
    pub fn count(&self) -> u8 {
        self.cards.len() as u8
    }

    /// Draws a card from the specified deck.
    ///
    /// # Arguments
    /// * `&mut deck` - mutable ref to a Deck object to draw a card from
    /// * `n` - number of cards to draw in `u8`
    ///
    /// # Returns
    /// * `Ok(_)` - Card was added to hand
    /// * `Err(_)` - Error raised on either hand size limit not allowing a hand to draw, or due to
    /// there not being enough cards to draw the specified value left in the deck
    ///
    /// # Example
    /// ```rust
    /// let deck = Deck::new();
    /// let hand = Hand::new(5);
    ///
    /// assert_eq!(deck.count(), 52);
    /// assert_eq!(hand.count(), 0);
    ///
    /// let _r = match hand.draw(&mut deck, 5) {
    ///    Ok(_) => (),
    /// Err(_) => println!("Error"),
    /// };
    ///
    /// assert_eq!(deck.count(), 47);
    /// assert_eq!(hand.count(), 5);
    /// ```
    pub fn draw(&mut self, deck: &mut Deck, n: u8) -> Result<(), &'static str> {
        if self.count() + n > self.get_hand_limit() {
            return Err("Hand limit exceeded");
        }

        if n > deck.count() {
            return Err("Deck limit exceeded");
        }

        for _ in 0..n {
            match deck.deal_card() {
                Ok(mut card) => {
                    card.face_up = false;
                    self.cards.push(card)
                }
                Err(_) => return Err("Deck limit exceeded"),
            }
        }
        Ok(())
    }

    /// Draws a card from the specified deck face up.
    ///
    /// # Arguments
    /// * `&mut deck` - mutable ref to a Deck object to draw a card from
    /// * `n` - number of cards to draw in `u8`
    ///
    /// # Returns
    /// * `Ok(_)` - Card was added to hand
    /// * `Err(_)` - Error raised on either hand size limit not allowing a hand to draw, or due to
    /// there not being enough cards to draw the specified value left in the deck
    ///
    /// # Example
    /// ```rust
    /// let deck = Deck::new();
    /// let hand = Hand::new(5);
    ///
    /// assert_eq!(deck.count(), 52);
    /// assert_eq!(hand.count(), 0);
    ///
    /// let _r = match hand.draw_face_up(&mut deck, 5) {
    ///    Ok(_) => (),
    /// Err(_) => println!("Error"),
    /// };
    ///
    /// assert_eq!(deck.count(), 47);
    /// assert_eq!(hand.count(), 5);
    /// ```
    pub fn draw_face_up(&mut self, deck: &mut Deck, n: u8) -> Result<(), &'static str> {
        if self.count() + n > self.get_hand_limit() {
            return Err("Hand limit exceeded");
        }

        if n > deck.count() {
            return Err("Deck limit exceeded");
        }

        for _ in 0..n {
            match deck.deal_card() {
                Ok(mut card) => {
                    card.face_up = true;
                    self.cards.push(card)
                }
                Err(_) => return Err("Deck limit exceeded"),
            }
        }
        Ok(())
    }

    pub fn get_face_up_cards(&self) -> Hand {
        let mut face_up_cards: Vec<Card> = Vec::new();
        for card in self.cards.clone() {
            if card.face_up {
                face_up_cards.push(card);
            }
        }
        let mut hand = Hand::new(face_up_cards.len().try_into().unwrap());
        hand.cards = face_up_cards;
        hand
    }

    /// Discards a card from the hand
    ///
    /// # Arguments
    /// * `n` - index of card to discard
    ///
    /// # Returns
    /// * `Ok(_)` - Card was discarded
    /// * `Err(_)` - Error raised on index being out of range of hand
    ///
    /// # Example
    /// ```rust
    /// let deck = Deck::new();
    /// let hand = Hand::new(5);
    ///
    /// assert_eq!(hand.count(), 0);
    ///
    /// let _r = match hand.draw(&mut deck, 5) {
    ///    Ok(_) => (),
    /// Err(_) => println!("Error"),
    /// };
    ///
    /// assert_eq!(hand.count(), 5);
    ///
    /// let _r = match hand.discard(2) {
    ///  Ok(_) => (),
    ///  Err(_) => println!("Error"),
    /// }
    ///
    /// assert_eq!(hand.count(), 4);
    /// ```
    pub fn discard(&mut self, n: u8) -> Result<(), &'static str> {
        if n < self.count() {
            self.cards.remove(n.into());
        } else {
            return Err("Index error");
        }
        Ok(())
    }

    /// Discards all the cards from the hand
    ///
    /// # Example
    /// ```rust
    /// let deck = Deck::new();
    /// let hand = Hand::new(5);
    ///
    /// assert_eq!(hand.count(), 0);
    ///
    /// let _r = match hand.draw(&mut deck, 5) {
    ///    Ok(_) => (),
    /// Err(_) => println!("Error"),
    /// };
    ///
    /// assert_eq!(hand.count(), 5);
    ///
    /// hand.discard_hand();
    ///
    /// assert_eq!(hand.count(), 0);
    /// ```
    pub fn discard_hand(&mut self) {
        for _ in 0..self.count() {
            match self.discard(0) {
                Ok(_) => (),
                Err(_) => println!("Error"),
            }
        }
    }

    /// Used to add a specific card to a hand
    ///
    /// # Arguments
    /// * `c` - a specific `Card` to add to a hand
    ///
    /// # Example
    /// ```rust
    /// let hand = Hand::new(5);
    /// let cheat_card = "AS".parse().unwrap();
    ///
    /// hand.cheat(cheat_card);
    /// assert_eq!(hand.count(), 1);
    /// ```
    pub fn cheat(&mut self, c: Card) {
        self.cards.push(c);
    }

    pub fn sort(&mut self) {
        self.cards.sort_by(|a, b| {
            let a_value = a.get_value_as_u8() * 10 + a.get_suit() as u8;
            let b_value = b.get_value_as_u8() * 10 + b.get_suit() as u8;
            b_value.cmp(&a_value)
        });
    }

    pub fn get_largest_value(&mut self) -> Value {
        self.sort();
        self.cards[0].get_value()
    }

    pub fn get_smallest_value(&mut self) -> Value {
        self.sort();
        self.cards[self.cards.len() - 1].get_value()
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            cards: Vec::new(),
            hand_limit: 0,
        }
    }
}

// This is some temp code meant to work for comparisons
impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.cards == other.cards
    }
}
// I dont know if I need this
impl Eq for Hand {}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hand_string = vec![];
        for card in &self.cards {
            hand_string.push(card.to_string());
        }
        write!(f, "{}", hand_string.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::Suit;
    use crate::game::card::Value;

    #[test]
    fn test_hand_create_new() {
        let hand = Hand::new(5);
        assert_eq!(hand.get_hand_limit(), 5);
        assert_eq!(hand.count(), 0);
    }

    #[test]
    fn test_hand_get_hand_cards() {
        let mut hand = Hand::new(5);
        let card = Card::new(Value::Ten, Suit::Club).unwrap();
        hand.cheat(card);
        assert_eq!(*hand.get_hand_cards(), vec![card]);
    }

    #[test]
    fn test_hand_get_limit() {
        let hand = Hand::new(5);
        assert_eq!(hand.get_hand_limit(), 5);
    }

    #[test]
    fn test_hand_deal_hand_limit() {
        let mut deck = Deck::new();
        let mut hand = Hand::new(5);
        assert_eq!(deck.count(), 52);
        assert_eq!(hand.count(), 0);

        let result = hand.draw(&mut deck, 5);
        assert!(result.is_ok());
        assert_eq!(deck.count(), 47);
        assert_eq!(hand.count(), 5);
    }

    #[test]
    fn test_hand_deal_under_limit() {
        let mut deck = Deck::new();
        let mut hand = Hand::new(5);
        assert_eq!(deck.count(), 52);
        assert_eq!(hand.count(), 0);

        let result = hand.draw(&mut deck, 0);
        assert!(result.is_ok());
        assert_eq!(deck.count(), 52);
        assert_eq!(hand.count(), 0);

        let result = hand.draw(&mut deck, 1);
        assert!(result.is_ok());
        assert_eq!(deck.count(), 51);
        assert_eq!(hand.count(), 1);
    }

    #[test]
    fn test_hand_deal_exceeds_hand_limit() {
        let mut deck = Deck::new();
        let mut hand = Hand::new(5);

        let result = hand.draw(&mut deck, 51);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Hand limit exceeded");
        assert_eq!(deck.count(), 52);
        assert_eq!(hand.count(), 0);
    }

    #[test]
    fn test_hand_deal_exceeds_deck_limit() {
        let mut deck = Deck::new();
        let mut hand = Hand::new(100);

        let _result = hand.draw(&mut deck, 51);
        assert_eq!(hand.count(), 51);
        assert_eq!(deck.count(), 1);

        let result = hand.draw(&mut deck, 25);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Deck limit exceeded");
        assert_eq!(deck.count(), 1);
        assert_eq!(hand.count(), 51);
    }

    #[test]
    fn test_hand_multiple_hands() {
        let mut deck = Deck::new();
        let mut hand1 = Hand::new(5);
        let mut hand2 = Hand::new(5);

        let _r1 = hand1.draw(&mut deck, 5);
        let _r2 = hand2.draw(&mut deck, 5);
        assert_eq!(deck.count(), 42);
    }

    #[test]
    fn test_hand_discard() {
        let mut deck = Deck::new();
        let mut hand = Hand::new(5);
        let _ = hand.draw(&mut deck, 5);
        assert_eq!(hand.count(), 5);

        let _ = hand.discard(0);
        assert_eq!(hand.count(), 4);
    }

    #[test]
    fn test_hand_discard_index_error() {
        let mut deck = Deck::new();
        let mut hand = Hand::new(5);
        let _ = hand.draw(&mut deck, 5);
        assert_eq!(hand.count(), 5);

        let result = hand.discard(5);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Index error");
        assert_eq!(hand.count(), 5);
    }

    #[test]
    fn test_hand_get_largest() {
        let mut hand = Hand::new(3);
        hand.cheat(Card::new(Value::Five, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Seven, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Spade).unwrap());

        let val = hand.get_largest_value();

        assert_eq!(val, Value::Seven);
    }

    #[test]
    fn test_hand_get_smallest() {
        let mut hand = Hand::new(3);
        hand.cheat(Card::new(Value::Five, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Seven, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Spade).unwrap());

        let val = hand.get_smallest_value();

        assert_eq!(val, Value::Three);
    }
}
