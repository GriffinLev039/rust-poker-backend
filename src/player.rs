use crate::{card::Card, hand::Hand};

#[derive(Debug, Clone)]
pub struct Player {
    hand: Hand,
    pub has_folded: bool,
    chip_stack: u32,
    pub current_bet: u32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            hand: Hand::default(),
            chip_stack: 2000,
            has_folded: false,
            current_bet: 0
        }
    }
}
impl Player {
    fn new(h: Vec<Card>, c: u32) -> Self {
        Player {
            hand: Hand::from(h),
            chip_stack: c,
            has_folded: false,
            current_bet: 0,
        }
    }

    pub fn get_hand(&self) -> &Hand {
        &self.hand
    }

    pub fn mut_hand(&mut self) -> &mut Hand{
        &mut self.hand
    }

    fn discard_cards(&mut self, targets: &mut Vec<u32>) {
        targets.sort();
        targets.reverse();
        for num in targets {
            self.hand.mut_hand().remove(*num as usize);
        }
    }

    fn draw_card(&mut self, c: Card) {
        self.hand.draw_card(c);
    }

    fn mut_stack(&mut self) -> &mut u32 {
        &mut self.chip_stack
    }

    pub fn up_stack(&mut self, value: u32) {
        self.chip_stack += value;
    }

    pub fn down_stack(&mut self, value: u32) -> Result<(), String> {
        if self.chip_stack < value {
            return Err(String::from("Not enough :("));
        }
        self.chip_stack -= value;
        Ok(())
    }

    pub fn get_stack(&self) -> u32 {
        self.chip_stack
    }

    pub fn is_all_in(&self) -> bool {
        self.chip_stack == 0
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.hand == other.hand
    }
}

impl Eq for Player {}

impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.hand.cmp(&other.hand))
    }
}

impl Ord for Player {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hand.cmp(&other.hand)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn constructors_test() {
        let _p1 = Player::default();
        let _p2: Player = Player::new(vec![], 5);
        let _p3: Player = Player::new(
            vec![
                Card::default(),
                Card::default(),
                Card::default(),
                Card::default(),
                Card::default(),
            ],
            2000,
        );
    }

    #[test]
    fn stack_test() {
        let mut p1 = Player::default();
        assert_eq!(p1.get_stack(), 2000);
        p1.up_stack(100);
        assert_eq!(p1.get_stack(), 2100);
        p1.down_stack(50);
        assert_eq!(p1.get_stack(), 2050);
    }

    #[test]
    fn discard_test() {
        let mut p: Player = Player::new(
            vec![
                Card::default(),
                Card::default(),
                Card::default(),
                Card::default(),
                Card::default(),
            ],
            2000,
        );
        let mut my_vec: Vec<u32> = vec![0, 1, 2];
        p.discard_cards(&mut my_vec);
    }

    #[test]
    fn draw_test() {
        let mut p: Player = Player::new(
            vec![
                Card::default(),
                Card::default(),
                Card::default(),
                Card::default(),
            ],
            2000,
        );
        p.draw_card(Card::default());
    }
    #[test]
    fn get_set_test(){
        let mut p:Player = Player::default();
        p.get_hand();
        p.mut_hand();
    }
}
