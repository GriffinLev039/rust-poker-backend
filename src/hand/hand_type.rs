use std::cmp::Ordering;

use crate::card::value::Value;

#[derive(Debug, Clone)]
pub enum HandType {
    RoyalFlush,
    StraightFlush {
        kicker: Value,
    },
    FourKind {
        kind_value: Value,
        kicker: Value,
    },
    FullHouse {
        three_value: Value,
        two_value: Value,
    },
    Flush {
        cards: Vec<Value>,
    },
    Straight {
        kicker: Value,
    },
    ThreeKind {
        three_value: Value,
        other_cards: Vec<Value>,
    },
    TwoPair {
        pair_one: Value,
        pair_two: Value,
        kicker: Value,
    },
    Pair {
        pair: Value,
        other_cards: Vec<Value>,
    },
    HighCard {
        cards: Vec<Value>,
    },
}

impl HandType {
    pub fn hand_value(&self) -> u8 {
        match *self {
            HandType::RoyalFlush => 10,
            HandType::StraightFlush { kicker: _ } => 9,
            HandType::FourKind {
                kind_value: _,
                kicker: _,
            } => 8,
            HandType::FullHouse {
                three_value: _,
                two_value: _,
            } => 7,
            HandType::Flush { cards: _ } => 6,
            HandType::Straight { kicker: _ } => 5,
            HandType::ThreeKind {
                three_value: _,
                other_cards: _,
            } => 4,
            HandType::TwoPair {
                pair_one: _,
                pair_two: _,
                kicker: _,
            } => 3,
            HandType::Pair {
                pair: _,
                other_cards: _,
            } => 2,
            HandType::HighCard { cards: _ } => 1,
        }
    }
}

impl PartialEq for HandType {
    fn eq(&self, other: &Self) -> bool {
        self.hand_value() == other.hand_value()
    }
}

impl Eq for HandType {}

impl PartialOrd for HandType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.hand_value().cmp(&other.hand_value()))
    }
}

impl Ord for HandType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.eq(&other) {
            match (self, other) {
                (
                    HandType::StraightFlush { kicker: k1 },
                    HandType::StraightFlush { kicker: k2 },
                ) => k1.cmp(k2),

                (
                    HandType::FourKind {
                        kind_value: kv1,
                        kicker: k1,
                    },
                    HandType::FourKind {
                        kind_value: kv2,
                        kicker: k2,
                    },
                ) => kv1.cmp(kv2).then_with(|| k1.cmp(k2)),
                (
                    HandType::FullHouse {
                        three_value: ev1,
                        two_value: wv1,
                    },
                    HandType::FullHouse {
                        three_value: ev2,
                        two_value: wv2,
                    },
                ) => ev1.cmp(ev2).then_with(|| wv1.cmp(wv2)),
                (HandType::Straight { kicker: k1 }, HandType::Straight { kicker: k2 }) => {
                    k1.cmp(k2)
                }
                // Both hands are flushes
                (HandType::Flush { cards: c1 }, HandType::Flush { cards: c2 }) => c1.cmp(c2),
                //Both hands are threekinds
                (
                    HandType::ThreeKind {
                        three_value: t1,
                        other_cards: c1,
                    },
                    HandType::ThreeKind {
                        three_value: t2,
                        other_cards: c2,
                    },
                ) => t1.cmp(t2).then_with(|| c1.cmp(c2)),
                // Both hands are pairs
                (
                    HandType::Pair {
                        pair: p1,
                        other_cards: c1,
                    },
                    HandType::Pair {
                        pair: p2,
                        other_cards: c2,
                    },
                ) => p1.cmp(p2).then_with(|| c1.cmp(c2)),
                // Both Hands are Highcards
                (HandType::HighCard { cards: c1 }, HandType::HighCard { cards: c2 }) => c1.cmp(c2),

                _ => Ordering::Equal, //Since it checks to see if hands are equal, the default behavior is equal...
            }
        } else {
            self.hand_value().cmp(&other.hand_value())
        }
    }
}
