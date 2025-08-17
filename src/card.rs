pub mod suit;
pub mod value;

use core::fmt;

use suit::Suit;
use value::Value;

#[derive(Debug, Clone, Copy)]
pub struct Card {
    pub suit: Suit,
    pub value: Value,
}

impl Card {
    pub fn new(suit: Suit, value: Value) -> Self {
        Card { suit, value }
    }

    pub fn numeric_value(&self) -> u32 {
        self.value.numeric_value()
    }
}

impl Default for Card {
    fn default() -> Self {
        Card {
            suit: Suit::Diamond,
            value: Value::Ace,
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.suit, self.value)
    }
}

impl Eq for Card {}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.value.numeric_value() == other.value.numeric_value()
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn card_basics() {
        let _card = Card::default();
        let _card = Card::new(Suit::Diamond, Value::Ace);
    }
}
