//! Gives the ablilty to order and check equality of scored poker hands.
//!
//! Creates an abstraction for a ScoredHand, and implements ordering
//! and equality checking for these scored hands.
//! Also gives the ability to find the best scoring poker hand for a given
//! 5 card poker hand.
//!

use crate::game::card::{Card, Value};
use crate::game::hand::Hand;
use core::fmt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

/// A hand-like enum for holding the 'power level' of a poker hand.
/// Can differentiate between different hand types, as well as
/// within the same ScoredHand.
#[derive(Debug, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum ScoredHand {
    StraightFlush([Value; 5]),
    FourOfAKind([Value; 5]),
    FullHouse([Value; 5]),
    Flush([Value; 5]),
    Straight([Value; 5]),
    ThreeOfAKind([Value; 5]),
    TwoPair([Value; 5]),
    OnePair([Value; 5]),
    HighCard([Value; 5]),
}

impl ScoredHand {
    /// Returns the 'rank' of a scored hand, with a higher number meaning
    /// a stronger hand.
    ///
    /// # Returns
    /// The 'rank' of the scored hand.
    ///
    /// # Example
    ///  ```rust
    /// let mut hand = Hand::new(5);
    /// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
    /// hand.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
    /// hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
    /// hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
    /// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
    /// let scored_hand = get_best_5_card_hand(&hand);
    ///
    /// assert_eq!(scored_hand.rank(), 10);
    /// ```
    pub fn rank(&self) -> u8 {
        match self {
            ScoredHand::StraightFlush(_) => 10,
            ScoredHand::FourOfAKind(_) => 9,
            ScoredHand::FullHouse(_) => 8,
            ScoredHand::Flush(_) => 7,
            ScoredHand::Straight(_) => 6,
            ScoredHand::ThreeOfAKind(_) => 5,
            ScoredHand::TwoPair(_) => 4,
            ScoredHand::OnePair(_) => 3,
            ScoredHand::HighCard(_) => 2,
        }
    }
}

impl fmt::Display for ScoredHand {
    /// Displays the string version of a [`ScoredHand`].
    ///
    /// # Arguments
    /// * `f` - the [`fmt::Formatter`] to be written with.
    ///
    /// # Returns
    /// The [`Result`] of the new string.
    ///
    /// # Example
    /// ```rust
    /// let mut hand = Hand::new(5);
    /// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
    /// hand.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
    /// hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
    /// hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
    /// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
    /// let scored_hand = get_best_5_card_hand(&hand);
    ///
    /// let scored_hand_format = format!("{scored_hand}");
    ///
    /// assert_eq!(scored_hand_format, "Straight Flush: 5, 4, 3, 2, 1");
    /// ```
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
            .map(|&v| String::from(char::try_from(v).expect("Could not convert value")))
            .collect();
        write!(f, "{}: {}", hand_name, formatted_values.join(", "))
    }
}

impl PartialOrd for ScoredHand {
    /// Decides the partial ordering between [`ScoredHand`]s.
    /// Uses the full ordering function.
    ///
    /// # Arguments
    /// * `other` - the [`ScoredHand`] to be compared with.
    ///
    /// # Returns
    /// The partial ordering of the given [`ScoredHand`]s.
    ///
    /// # Example
    /// ```rust
    /// let mut hand1 = Hand::new(5);
    /// hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
    /// hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
    /// hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
    /// hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
    /// hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
    /// let mut hand2 = Hand::new(5);
    /// hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
    /// hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
    /// hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
    /// hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
    /// hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
    ///
    /// assert_eq!(get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2), true);
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredHand {
    /// Decides the ordering between [`ScoredHand`]s.
    /// Differentiates between both different hand types (eg. [`ScoredHand::FullHouse`] vs [`ScoredHand::HighCard`]),
    /// as well as between hands of the same type.
    ///
    /// # Arguments
    /// * `other` - the [`ScoredHand`] to be compared with.
    ///
    /// # Returns
    /// The ordering of the given [`ScoredHand`]s.
    ///
    /// # Example
    /// ```rust
    /// let mut hand1 = Hand::new(5);
    /// hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
    /// hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
    /// hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
    /// hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
    /// hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
    /// let mut hand2 = Hand::new(5);
    /// hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
    /// hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
    /// hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
    /// hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
    /// hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
    ///
    /// assert_eq!(get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2), true);
    /// ```
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.rank() == other.rank() {
            match (self, other) {
                (ScoredHand::FullHouse(a), ScoredHand::FullHouse(b)) => {
                    if a[0] > b[0] {
                        return Ordering::Greater;
                    } else if a[0] < b[0] {
                        return Ordering::Less;
                    }
                    if a[3] > b[3] {
                        return Ordering::Greater;
                    } else if a[3] < b[3] {
                        return Ordering::Less;
                    }
                    if a[4] > b[4] {
                        return Ordering::Greater;
                    } else if a[4] < b[4] {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                (ScoredHand::TwoPair(a), ScoredHand::TwoPair(b)) => {
                    if a[0] > b[0] {
                        return Ordering::Greater;
                    } else if a[0] < b[0] {
                        return Ordering::Less;
                    }
                    if a[2] > b[2] {
                        return Ordering::Greater;
                    } else if a[2] < b[2] {
                        return Ordering::Less;
                    }
                    if a[4] > b[4] {
                        return Ordering::Greater;
                    } else if a[4] < b[4] {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                (ScoredHand::StraightFlush(a), ScoredHand::StraightFlush(b)) => {
                    if a[0] > b[0] {
                        return Ordering::Greater;
                    } else if a[0] < b[0] {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                (ScoredHand::FourOfAKind(a), ScoredHand::FourOfAKind(b)) => {
                    if a[0] > b[0] {
                        return Ordering::Greater;
                    } else if a[0] < b[0] {
                        return Ordering::Less;
                    }
                    if a[4] > b[4] {
                        return Ordering::Greater;
                    } else if a[4] < b[4] {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                (ScoredHand::Flush(a), ScoredHand::Flush(b)) => {
                    if a[0] > b[0] {
                        return Ordering::Greater;
                    } else if a[0] < b[0] {
                        return Ordering::Less;
                    }
                    if a[1] > b[1] {
                        return Ordering::Greater;
                    } else if a[1] < b[1] {
                        return Ordering::Less;
                    }
                    if a[2] > b[2] {
                        return Ordering::Greater;
                    } else if a[2] < b[2] {
                        return Ordering::Less;
                    }
                    if a[3] > b[3] {
                        return Ordering::Greater;
                    } else if a[3] < b[3] {
                        return Ordering::Less;
                    }
                    if a[4] > b[4] {
                        return Ordering::Greater;
                    } else if a[4] < b[4] {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                (ScoredHand::Straight(a), ScoredHand::Straight(b)) => {
                    if a[0] > b[0] {
                        return Ordering::Greater;
                    } else if a[0] < b[0] {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                (ScoredHand::ThreeOfAKind(a), ScoredHand::ThreeOfAKind(b)) => {
                    if a[0] > b[0] {
                        return Ordering::Greater;
                    } else if a[0] < b[0] {
                        return Ordering::Less;
                    }
                    if a[3] > b[3] {
                        return Ordering::Greater;
                    } else if a[3] < b[3] {
                        return Ordering::Less;
                    }
                    if a[4] > b[4] {
                        return Ordering::Greater;
                    } else if a[4] < b[4] {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                (ScoredHand::OnePair(a), ScoredHand::OnePair(b)) => {
                    if a[0] > b[0] {
                        return Ordering::Greater;
                    } else if a[0] < b[0] {
                        return Ordering::Less;
                    }
                    if a[2] > b[2] {
                        return Ordering::Greater;
                    } else if a[2] < b[2] {
                        return Ordering::Less;
                    }
                    if a[3] > b[3] {
                        return Ordering::Greater;
                    } else if a[3] < b[3] {
                        return Ordering::Less;
                    }
                    if a[4] > b[4] {
                        return Ordering::Greater;
                    } else if a[4] < b[4] {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                (ScoredHand::HighCard(a), ScoredHand::HighCard(b)) => {
                    if a[0] > b[0] {
                        return Ordering::Greater;
                    } else if a[0] < b[0] {
                        return Ordering::Less;
                    }
                    if a[1] > b[1] {
                        return Ordering::Greater;
                    } else if a[1] < b[1] {
                        return Ordering::Less;
                    }
                    if a[2] > b[2] {
                        return Ordering::Greater;
                    } else if a[2] < b[2] {
                        return Ordering::Less;
                    }
                    if a[3] > b[3] {
                        return Ordering::Greater;
                    } else if a[3] < b[3] {
                        return Ordering::Less;
                    }
                    if a[4] > b[4] {
                        return Ordering::Greater;
                    } else if a[4] < b[4] {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                _ => return Ordering::Equal,
            }
        }
        self.rank().cmp(&other.rank())
    }
}

impl PartialEq for ScoredHand {
    /// Decides the equality between [`ScoredHand`]s.
    /// Uses the ordering trait, and returns if it is [`Ordering::Equal`].
    ///
    /// # Arguments
    /// * `other` - the [`ScoredHand`] to be compared with.
    ///
    /// # Returns
    /// Whether the [`ScoredHand`]s are equal.
    ///
    /// # Example
    /// ```rust
    /// let mut hand1 = Hand::new(5);
    /// hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
    /// hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
    /// hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
    /// hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
    /// hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
    /// let mut hand2 = Hand::new(5);
    /// hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
    /// hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
    /// hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
    /// hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
    /// hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
    ///
    /// assert_eq!(get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2), true);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// Determines if a [`Hand`] is a Straight Flush.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a Straight Flush.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_straight_flush(&hand), true);
/// ```
///
fn check_if_hand_straight_flush(hand: &Hand) -> bool {
    if !check_if_hand_flush(hand) {
        return false;
    } else if !check_if_hand_straight(hand) {
        return false;
    }
    true
}

/// Determines if a [`Hand`] is a Straight Flush with an Ace Low.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a Straight Flush with an Ace Low.
///
/// # Example
/// ```rust
///
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_straight_flush_ace_low(&hand), true);
/// ```
///
fn check_if_hand_straight_flush_ace_low(hand: &Hand) -> bool {
    if !check_if_hand_flush(hand) {
        return false;
    } else if !check_if_hand_straight_ace_low(hand) {
        return false;
    }
    true
}

/// Determines if a [`Hand`] is a Four of a Kind.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a Four of a Kind.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Nine, Suit::Spade).unwrap());
/// hand.cheat(Card::new(Value::Nine, Suit::Diamond).unwrap());
/// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_four_of_a_kind(&hand), true);
/// ```
///
fn check_if_hand_four_of_a_kind(hand: &Hand) -> bool {
    // converts hand into HashMap to count the frequency of each card value in the hand
    let card_counts =
        hand.get_hand_cards()
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<u8, u8>, card| {
                *acc.entry(card.get_value_as_u8()).or_insert(0) += 1;
                acc
            });
    if card_counts.values().sorted().eq([1, 4].iter()) {
        return true;
    }
    false
}

/// Determines if a [`Hand`] is a Full House.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a Full House.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Jack, Suit::Spade).unwrap());
/// hand.cheat(Card::new(Value::Jack, Suit::Diamond).unwrap());
///
/// assert_eq!(check_if_hand_full_house(&hand), true);
/// ```
///
fn check_if_hand_full_house(hand: &Hand) -> bool {
    // converts hand into HashMap to count the frequency of each card value in the hand
    let card_counts =
        hand.get_hand_cards()
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<u8, u8>, card| {
                *acc.entry(card.get_value_as_u8()).or_insert(0) += 1;
                acc
            });
    if card_counts.values().sorted().eq([2, 3].iter()) {
        return true;
    }
    false
}

/// Determines if a [`Hand`] is a Flush.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a Flush.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_flush(&hand), true);
/// ```
///
fn check_if_hand_flush(hand: &Hand) -> bool {
    // converts hand into HashMap to count the frequency of each suit in the hand
    let suit_counts =
        hand.get_hand_cards()
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<char, i32>, card| {
                *acc.entry(
                    char::try_from(card.get_suit()).expect("Could get get char from suit"),
                )
                .or_insert(0) += 1;
                acc
            });
    if suit_counts.values().len() > 1 {
        return false;
    }
    true
}

/// Determines if a [`Hand`] is a Straight.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a Straight.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Ten, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_straight(&hand), true);
/// ```
///
fn check_if_hand_straight(hand: &Hand) -> bool {
    let mut sorted_hand = hand.get_hand_cards().clone();
    sorted_hand.sort();
    sorted_hand
        .iter()
        .map(|card| card.get_value_as_u8() - sorted_hand[0].get_value_as_u8())
        .eq(0..5)
}

/// Determines if a [`Hand`] is a Straight with an Ace Low.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a Straight with an Ace Low.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Two, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_straight_ace_low(&hand), true);
/// ```
///
fn check_if_hand_straight_ace_low(hand: &Hand) -> bool {
    let mut sorted_hand: Vec<Card> = hand.get_hand_cards().clone();
    sorted_hand.sort();
    // if highest value card is an Ace, try checking again with Ace being low valued
    if sorted_hand[4].get_value_as_u8() == Value::Ace as u8 {
        sorted_hand[4] = match Card::new(Value::AceLow, sorted_hand[4].get_suit()) {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };
        sorted_hand.sort();
        return sorted_hand
            .iter()
            .map(|card| card.get_value_as_u8() - sorted_hand[0].get_value_as_u8())
            .eq(0..5);
    }
    false
}

/// Determines if a [`Hand`] is a Three of a Kind.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a Three of a Kind.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
/// hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_three_of_a_kind(&hand), true);
/// ```
///
fn check_if_hand_three_of_a_kind(hand: &Hand) -> bool {
    // converts hand into HashMap to count the frequency of each card value in the hand
    let card_counts =
        hand.get_hand_cards()
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<u8, u8>, card| {
                *acc.entry(card.get_value_as_u8()).or_insert(0) += 1;
                acc
            });
    if card_counts.values().sorted().eq([1, 1, 3].iter()) {
        return true;
    }
    false
}

/// Determines if a [`Hand`] is a Two Pair.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a Two Pair.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Three, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_two_pair(&hand), true);
/// ```
///
fn check_if_hand_two_pair(hand: &Hand) -> bool {
    // converts hand into HashMap to count the frequency of each card value in the hand
    let card_counts =
        hand.get_hand_cards()
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<u8, u8>, card| {
                *acc.entry(card.get_value_as_u8()).or_insert(0) += 1;
                acc
            });
    if card_counts.values().sorted().eq([1, 2, 2].iter()) {
        return true;
    }
    false
}

/// Determines if a [`Hand`] is a One Pair.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a One Pair.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_one_pair(&hand), true);
/// ```
///
fn check_if_hand_one_pair(hand: &Hand) -> bool {
    // converts hand into HashMap to count the frequency of each card value in the hand
    let card_counts =
        hand.get_hand_cards()
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<u8, u8>, card| {
                *acc.entry(card.get_value_as_u8()).or_insert(0) += 1;
                acc
            });
    if card_counts.values().sorted().eq([1, 1, 1, 2].iter()) {
        return true;
    }
    false
}

/// Determines if a [`Hand`] is a High Card.
///
/// # Arguments
/// * `hand` - the [`Hand`] to be checked.
///
/// # Returns
/// Whether the [`Hand`] is a High Card.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Three, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
///
/// assert_eq!(check_if_hand_high_card(&hand), true);
/// ```
///
#[allow(dead_code)]
fn check_if_hand_high_card(hand: &Hand) -> bool {
    // converts hand into HashMap to count the frequency of each card value in the hand
    let card_counts =
        hand.get_hand_cards()
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<u8, u8>, card| {
                *acc.entry(card.get_value_as_u8()).or_insert(0) += 1;
                acc
            });
    if card_counts.values().sorted().eq([1, 1, 1, 1, 1].iter()) {
        return true;
    }
    false
}

/// Converts a [`Hand`] into a specially sorted [`Value`] array of size 5.
/// By sorted, this means into the order in which the most important scoring
/// elements would be.
///
/// For example, if a hand was a Three of a Kind of 3's, with an Ace and Ten kicker, it would be sorted to
/// 3, 3, 3, A, T.
///
/// # Arguments
/// * `hand` - the [`Hand`] to convert.
///
/// # Returns
/// A specially sorted [`Value`] array of size 5.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
/// hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
///
/// let sorted_hand = convert_hand_to_power_sorting(&hand);
///
/// assert_eq!(sorted_hand, [Value::Ace, Value::Ace, Value::Five, Value::Four, Value::Three]);
/// ```
fn convert_hand_to_power_sorting(hand: &Hand) -> [Value; 5] {
    let card_count_map =
        hand.get_hand_cards()
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<u8, u8>, card| {
                *acc.entry(card.get_value_as_u8()).or_insert(0) += 1;
                acc
            });
    let mut card_counts: Vec<(u8, u8)> = card_count_map.into_iter().collect();
    card_counts.sort_by(|(val_a, count_a), (val_b, count_b)| {
        count_b.cmp(&count_a).then_with(|| val_b.cmp(&val_a))
    });

    let mut card_counts_u8 = Vec::new();

    for card_info in card_counts.iter() {
        for _ in 0..(card_info.1) {
            card_counts_u8.push(card_info.0);
        }
    }

    let card_counts: Vec<Value> = card_counts_u8
        .iter()
        .map(|x| Value::try_from(*x).expect("Could not get Value from val"))
        .collect();

    // let card_counts: Vec<Value> = card_counts.iter().map(|x| Value::try_from(x.0).expect("Could not get Value from val")).collect();
    // set a default array
    let mut card_arr: [Value; 5] = [Value::AceLow; 5];
    for (i, value) in card_counts.iter().enumerate() {
        card_arr[i] = *value;
    }

    card_arr
}

/// Returns the highest ranking [`ScoredHand`] that a 5-card [`Hand`] can be.
///
/// # Arguments
/// * `hand`: the [`Hand`] that is wanted to find the [`ScoredHand`] of.
///
/// # Returns
/// The [`ScoredHand`] of the best hand that could be created.
///
/// # Example
/// ```rust
/// let mut hand = Hand::new(5);
/// hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
/// hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());
///
/// assert_eq!(get_best_5_card_hand(&hand), ScoredHand::StraightFlush(convert_hand_to_power_sorting(&hand)));
/// ```
pub fn get_best_5_card_hand(hand: &Hand) -> ScoredHand {
    assert_eq!(hand.count(), 5);
    let hand_type = if check_if_hand_straight_flush(hand) {
        ScoredHand::StraightFlush(convert_hand_to_power_sorting(hand))
    } else if check_if_hand_straight_flush_ace_low(hand) {
        // CONVERT ACE TO ACELOW
        let mut ace_low_hand = Hand::new(5);
        for card in hand.get_hand_cards() {
            if card.get_value() == Value::Ace {
                ace_low_hand
                    .cheat(Card::new(Value::AceLow, card.get_suit()).expect("Error creating card"));
            } else {
                ace_low_hand.cheat(*card);
            }
        }
        ScoredHand::StraightFlush(convert_hand_to_power_sorting(&ace_low_hand))
    } else if check_if_hand_four_of_a_kind(hand) {
        ScoredHand::FourOfAKind(convert_hand_to_power_sorting(hand))
    } else if check_if_hand_full_house(hand) {
        ScoredHand::FullHouse(convert_hand_to_power_sorting(hand))
    } else if check_if_hand_flush(hand) {
        ScoredHand::Flush(convert_hand_to_power_sorting(hand))
    } else if check_if_hand_straight(hand) {
        ScoredHand::Straight(convert_hand_to_power_sorting(hand))
    } else if check_if_hand_straight_ace_low(hand) {
        // CONVERT ACE TO ACELOW
        let mut ace_low_hand = Hand::new(5);
        for card in hand.get_hand_cards() {
            if card.get_value() == Value::Ace {
                ace_low_hand
                    .cheat(Card::new(Value::AceLow, card.get_suit()).expect("Error creating card"));
            } else {
                ace_low_hand.cheat(*card);
            }
        }
        ScoredHand::Straight(convert_hand_to_power_sorting(&ace_low_hand))
    } else if check_if_hand_three_of_a_kind(hand) {
        ScoredHand::ThreeOfAKind(convert_hand_to_power_sorting(hand))
    } else if check_if_hand_two_pair(hand) {
        ScoredHand::TwoPair(convert_hand_to_power_sorting(hand))
    } else if check_if_hand_one_pair(hand) {
        ScoredHand::OnePair(convert_hand_to_power_sorting(hand))
    } else {
        // high card
        ScoredHand::HighCard(convert_hand_to_power_sorting(hand))
    };
    hand_type
}

pub fn get_best_from_7_card_hand(hand: &Hand) -> ScoredHand {
    hand.get_hand_cards()
        .iter()
        .cloned()
        .combinations(5)
        .map(|comb| get_best_5_card_hand(&Hand::from_cards(comb)))
        .max()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::{Card, Suit, Value};
    use crate::game::hand::Hand;

    #[test]
    fn test_scored_hand_rank() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let scored_hand = get_best_5_card_hand(&hand);

        assert_eq!(scored_hand.rank(), 10);
    }

    #[test]
    fn test_scored_hand_format() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let scored_hand = get_best_5_card_hand(&hand);

        let scored_hand_format = format!("{scored_hand}");

        assert_eq!(scored_hand_format, "Straight Flush: 5, 4, 3, 2, 1");
    }

    #[test]
    fn test_scored_7_card_hand_straight_flush() {
        let mut hand = Hand::new(7);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        //let scored_hand = get_best_5_card_hand(&hand);
        let scored_hand = get_best_from_7_card_hand(&hand);
        let scored_hand_format = format!("{scored_hand}");

        assert_eq!(scored_hand_format, "Straight Flush: 5, 4, 3, 2, 1");
    }

    #[test]
    fn test_scored_7_card_hand_high() {
        let mut hand = Hand::new(7);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Two, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Diamond).unwrap());
        hand.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::King, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        //let scored_hand = get_best_5_card_hand(&hand);
        let scored_hand = get_best_from_7_card_hand(&hand);
        let scored_hand_format = format!("{scored_hand}");

        assert_eq!(scored_hand_format, "High Card: A, K, Q, 9, 5");
    }

    #[test]
    fn test_scored_7_card_hand_full_house() {
        let mut hand = Hand::new(7);
        hand.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Two, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Diamond).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::King, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        //let scored_hand = get_best_5_card_hand(&hand);
        let scored_hand = get_best_from_7_card_hand(&hand);
        let scored_hand_format = format!("{scored_hand}");

        assert_eq!(scored_hand_format, "Full House: 3, 3, 3, 2, 2");
    }

    #[test]
    fn test_straight_flush() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_straight_flush(&hand), true);
    }

    #[test]
    fn test_straight_flush_ace_low() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_straight_flush_ace_low(&hand), true);
    }

    #[test]
    fn test_four_of_a_kind() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Nine, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Nine, Suit::Diamond).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_four_of_a_kind(&hand), true);
    }

    #[test]
    fn test_full_house() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Jack, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Jack, Suit::Diamond).unwrap());

        assert_eq!(check_if_hand_full_house(&hand), true);
    }

    #[test]
    fn test_flush() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_flush(&hand), true);
    }

    #[test]
    fn test_straight() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Ten, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_straight(&hand), true);
    }

    #[test]
    fn test_straight_ace_low() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Two, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_straight_ace_low(&hand), true);
    }

    #[test]
    fn test_wrap_around_not_straight() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Two, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_straight(&hand), false);
    }

    #[test]
    fn test_three_of_a_kind() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_three_of_a_kind(&hand), true);
    }

    #[test]
    fn test_two_pair() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_two_pair(&hand), true);
    }

    #[test]
    fn test_one_pair() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_one_pair(&hand), true);
    }

    #[test]
    fn test_high_card() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());

        assert_eq!(check_if_hand_high_card(&hand), true);
    }

    #[test]
    fn test_get_best_5_card_hand() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::King, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand),
            ScoredHand::StraightFlush(convert_hand_to_power_sorting(&hand))
        );
    }

    #[test]
    fn test_straight_flush_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_four_of_a_kind_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Diamond).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Diamond).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_full_house_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Diamond).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Diamond).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_flush_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_straight_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ten, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_three_of_a_kind_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_two_pair_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_one_pair_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_high_card_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) == get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_straight_flush_not_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_four_of_a_kind_not_eq_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Diamond).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Eight, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Eight, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Eight, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Eight, Suit::Diamond).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_four_of_a_kind_not_eq_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Diamond).unwrap());
        hand1.cheat(Card::new(Value::Six, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Diamond).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_full_house_not_eq_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Diamond).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ten, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Ten, Suit::Diamond).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_full_house_not_eq_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Diamond).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Diamond).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_flush_not_eq_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_flush_not_eq_2_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_flush_not_eq_mid_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_flush_not_eq_2_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Eight, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_flush_not_eq_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Six, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_straight_not_eq() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Two, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_three_of_a_kind_not_eq_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::King, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::King, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_three_of_a_kind_not_eq_mid_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Six, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_three_of_a_kind_not_eq_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_two_pair_not_eq_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::King, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_two_pair_not_eq_mid_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Two, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_two_pair_not_eq_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_one_pair_not_eq_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::King, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_one_pair_not_eq_2_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Six, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_one_pair_not_eq_2_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Six, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Six, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_one_pair_not_eq_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Two, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_high_card_not_eq_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_high_card_not_eq_2_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_high_card_not_eq_mid_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Eight, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_high_card_not_eq_2_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Six, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_high_card_not_eq_low_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_straight_flush_gt_four_of_a_king() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Diamond).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_four_of_a_kind_gt_full_house() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Diamond).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Diamond).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_full_house_gt_flush() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Nine, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Diamond).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_flush_gt_straight() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ten, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::King, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_straight_gt_three_of_a_kind() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ten, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Jack, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Queen, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::King, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_three_of_a_kind_gt_two_pair() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Spade).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_two_pair_gt_one_pair() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_one_pair_gt_high_card() {
        let mut hand1 = Hand::new(5);
        hand1.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand1.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand1.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        let mut hand2 = Hand::new(5);
        hand2.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Three, Suit::Club).unwrap());
        hand2.cheat(Card::new(Value::Five, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Seven, Suit::Heart).unwrap());
        hand2.cheat(Card::new(Value::Nine, Suit::Heart).unwrap());

        assert_eq!(
            get_best_5_card_hand(&hand1) > get_best_5_card_hand(&hand2),
            true
        );
    }

    #[test]
    fn test_convert_hand_to_power_sorting() {
        let mut hand = Hand::new(5);
        hand.cheat(Card::new(Value::Ace, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Ace, Suit::Club).unwrap());
        hand.cheat(Card::new(Value::Three, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Four, Suit::Heart).unwrap());
        hand.cheat(Card::new(Value::Five, Suit::Heart).unwrap());

        let sorted_hand = convert_hand_to_power_sorting(&hand);

        assert_eq!(
            sorted_hand,
            [
                Value::Ace,
                Value::Ace,
                Value::Five,
                Value::Four,
                Value::Three
            ]
        );
    }
}
