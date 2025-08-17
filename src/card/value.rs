use core::fmt;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Value {
    Ace,
    King,
    Queen,
    Jack,
    Ten,
    Nine,
    Eight,
    Seven,
    Six,
    Five,
    Four,
    Three,
    Two,
}

impl Value {
    pub fn numeric_value(&self) -> u32 {
        match *self {
            Value::Ace => 14,
            Value::King => 13,
            Value::Queen => 12,
            Value::Jack => 11,
            Value::Ten => 10,
            Value::Nine => 9,
            Value::Eight => 8,
            Value::Seven => 7,
            Value::Six => 6,
            Value::Five => 5,
            Value::Four => 4,
            Value::Three => 3,
            Value::Two => 2,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Ace => write!(f, "A"),
            Value::King => write!(f, "K"),
            Value::Queen => write!(f, "Q"),
            Value::Jack => write!(f, "J"),
            Value::Ten => write!(f, "10"),
            Value::Nine => write!(f, "9"),
            Value::Eight => write!(f, "8"),
            Value::Seven => write!(f, "7"),
            Value::Six => write!(f, "6"),
            Value::Five => write!(f, "5"),
            Value::Four => write!(f, "4"),
            Value::Three => write!(f, "3"),
            Value::Two => write!(f, "2"),
        }
    }
}

// impl Eq for Value {}

// impl PartialEq for Value {
//     fn eq(&self, other: &Self) -> bool {
//         if self.numeric_value() == other.numeric_value() {
//             return true;
//         }
//         false
//     }
// }
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.numeric_value().cmp(&other.numeric_value()))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.numeric_value().cmp(&other.numeric_value())
    }
}

#[cfg(test)]
mod test {
    // use super::*;

    #[test]
    fn numeric_value_test(){
        //TODO: Write this later for the sake of coverage.
    }

    #[test]
    fn display_test(){
        //TODO: Write this later for the sake of coverage.
    }
}