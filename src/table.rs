use std::cmp::Ordering;

use crate::card::Card;
use crate::deck::Deck;
use crate::hand::Hand;
use crate::player::Player;
use crate::table::player_action::PlayerAction;
use table_stages::TableStage;
mod player_action;
mod table_stages;
pub struct Table {
    //Combine players, nonfolded players, bets
    //Instead give Player qualities ig
    pub players: Vec<Player>,
    river: Vec<Card>,
    small_blind: u32,
    big_blind: u32,
    pot: u32,
    dealer_pos: u32,
    table_pos: u32,
    current_stage: TableStage,
    deck: Deck,
}

impl Default for Table {
    fn default() -> Self {
        Table {
            players: Vec::new(),
            river: Vec::new(),
            small_blind: 5,
            big_blind: 10,
            pot: 0,
            dealer_pos: 0,
            table_pos: 0,
            current_stage: TableStage::PreFlop,
            deck: Deck::new(),
        }
    }
}
//The idea is that this will be split into a few specific functions
//There should be a marked target player which all functions target
//There should be functions modifying the "river"
//Variables to track constants that would matter outside of that sole players decision
//A way to track folded players and skip them

impl Table {
    pub fn new(num_of_players: usize, small_blind: u32, big_blind: u32) -> Table {
        let mut temp_player_arr: Vec<Player> = Vec::new();

        for _i in 0..num_of_players {
            temp_player_arr.push(Player::default());
        }

        Table {
            players: temp_player_arr,
            river: Vec::new(),
            small_blind,
            big_blind,
            pot: 0,
            dealer_pos: 0,
            table_pos: 0,
            current_stage: TableStage::PreFlop,
            deck: Deck::new(),
        }
    }
    pub fn deal_cards(&mut self) {
        for player in &mut self.players {
            if player.has_folded {
                continue;
            }
            //TODO: ERROR HANDLING HERE?
            player.mut_hand().draw_card(self.deck.cards.pop().unwrap());
            player.mut_hand().draw_card(self.deck.cards.pop().unwrap());
        }
    }
    fn get_dealer(&self) -> &Player {
        self.players.get(self.dealer_pos as usize).unwrap()
    }
    fn move_dealer(&mut self) {
        self.dealer_pos += 1;
    }
    fn increase_blinds(&mut self, value: u32) {
        self.big_blind += value;
    }
    fn get_small_blind(&self) -> u32 {
        self.small_blind
    }
    fn get_big_blind(&self) -> u32 {
        self.big_blind
    }
    fn mut_small_blind(&mut self) -> &mut u32 {
        &mut self.small_blind
    }
    fn mut_big_blind(&mut self) -> &mut u32 {
        &mut self.big_blind
    }
    fn next_player(&mut self) -> usize {
        debug_assert!(
            self.players.iter().any(|p| !p.has_folded && !p.is_all_in()),
            "next_player called with no active players remaining"
        );
        let n = self.players.len();
        loop {
            self.table_pos = (self.table_pos + 1) % n as u32;
            let p = &self.players[self.table_pos as usize];
            if !p.has_folded && !p.is_all_in() {
                return self.table_pos as usize;
            }
        }
    }
    fn create_bet_array(&self) -> Vec<u32> {
        self.players.iter().map(|p| p.current_bet).collect()
    }

    fn get_highest_bet(&self) -> Result<u32, String> {
        let mut bet_arr = self.create_bet_array();
        let highest_bet: u32;
        if !bet_arr.is_empty() {
            bet_arr.sort_by(|a, b| b.cmp(a));
            highest_bet = bet_arr[0];
            Ok(highest_bet)
        } else {
            Err(String::from("No players found."))
        }
    }
    //List of actions that ARE possible
    //NOT necessarily a list of valid actions, just possible ones!
    pub fn get_possible_actions(&mut self, player_num: usize) -> Vec<PlayerAction> {
        let player = self.players[player_num].clone();
        let player_chips = player.get_stack();
        let player_bet = player.current_bet;
        let highest_bet = self.get_highest_bet().unwrap(); //TODO: Error handling?
        let player_fold_status = player.has_folded;
        if player_fold_status || self.players[player_num].is_all_in() {
            return vec![];
        }
        if player_bet == highest_bet {
            vec![
                PlayerAction::Fold,
                PlayerAction::Check,
                PlayerAction::Bet { value: 0 },
                PlayerAction::AllIn,
            ]
        } else if highest_bet > player_bet {
            if player_chips > highest_bet {
                //Fold, Call, Raise, AllIn
                vec![
                    PlayerAction::Fold,
                    PlayerAction::Call,
                    PlayerAction::Raise { value: 0 },
                    PlayerAction::AllIn,
                ]
            } else {
                //Fold, AllIn
                vec![PlayerAction::Fold, PlayerAction::AllIn]
            }
        } else {
            //Unsure if any way to reach this, I assume not
            //But just to be safe I will return vec![]
            eprint!("This should be impossible to reach!");
            vec![]
        }
    }

    pub fn handle_input(
        &mut self,
        get_input: &dyn Fn(usize) -> Result<PlayerAction, String>,
        num: usize,
    ) -> Result<PlayerAction, String> {
        let p_action = get_input(num).unwrap();
        println!("ACTION CHOSEN: {:?}", p_action);

        if self.get_current_player().has_folded {
            return Err("Player has already folded - no actions can be taken.".to_string());
        }
        let result: Result<PlayerAction, String> = match p_action {
            PlayerAction::AllIn => {
                println!("  TRIGGERED: ALLIN");
                let player_stack = self.get_current_player().get_stack();
                let player_bet = self.get_current_player().current_bet;

                self.current_player_bet(player_stack - player_bet).expect(
                    "Should always be valid as long as get_possible_actions() contains this.",
                );
                Ok(PlayerAction::AllIn)
            }

            PlayerAction::Raise { value: v } | PlayerAction::Bet { value: v } => {
                println!("  TRIGGERED: RAISE");
                if self.current_player_bet(v).is_ok() {
                    Ok(PlayerAction::Raise { value: v })
                } else {
                    Err(String::from("Bet invalid"))
                }
            }
            PlayerAction::Call => {
                println!("  TRIGGERED: CALL");
                let player_chip_count = self.get_current_player().get_stack();
                let player_current_bet = self.get_current_player().current_bet;
                let mut bet_arr = self.create_bet_array();
                let mut highest_bet: u32 = u32::MAX;
                if !bet_arr.is_empty() {
                    bet_arr.sort_by(|a, b| b.cmp(a));
                    highest_bet = bet_arr[0];
                }
                if player_current_bet == highest_bet {
                    Ok(PlayerAction::Check)
                } else if player_chip_count >= highest_bet {
                    self.current_player_bet(highest_bet - player_current_bet)
                        .expect(
                            "Already handled case where player chip count is not enough to call.",
                        );
                    Ok(PlayerAction::Call)
                } else if !bet_arr.is_empty() {
                    self.current_player_bet(player_chip_count - player_current_bet)
                        .expect("Should be impossible for player bet to exceed player chip count.");
                    Ok(PlayerAction::AllIn)
                } else {
                    Err(String::from("No players found."))
                }
            }
            PlayerAction::Check => {
                println!("  TRIGGERED: CHECK");

                //If current players bet matches highest bet in pool, then its ok
                // Otherwise err
                let player_current_bet = self.get_current_player().current_bet;
                let mut bet_arr = self.create_bet_array();
                let mut highest_bet: u32 = u32::MAX;
                if !bet_arr.is_empty() {
                    bet_arr.sort_by(|a, b| b.cmp(a));
                    highest_bet = bet_arr[0];
                }
                if player_current_bet == highest_bet {
                    Ok(PlayerAction::Check)
                } else {
                    Err(String::from("No players found."))
                }
            }
            PlayerAction::Fold => {
                println!("  TRIGGERED: FOLD");
                self.get_current_player().has_folded = true;
                Ok(PlayerAction::Fold)
                //TODO: FOLDING MECHANICS ARE NOT IMPLEMENTED YET
                // So while this block is technically correct (as of now)
                // I want to revisit it once folding is finished.
            }
            _ => Err(String::from("Unsupported action.")),
        };
        result
    }

    pub fn handle_bets(&mut self, get_input: &dyn Fn(usize) -> Result<PlayerAction, String>) {
        loop {
            for i in 0..self.players.len() {
                // let player = &mut self.players[i];
                loop {
                    if let Ok(p_action) = get_input(i) {
                        //Cloning might be necessary here :(|
                        if !self.get_possible_actions(i as usize).contains(&p_action) {
                            println!("This action is invalid");
                            //I want some better way to communicate an error here.
                            //Way to return error to the player without fucking everything up? idk
                            // Err(String::from("The chosen action is not allowed under the current gamestate."))
                            continue;
                        }
                        let result: Result<PlayerAction, String> = match p_action {
                            PlayerAction::AllIn => {
                                let player_stack = self.get_current_player().get_stack();
                                let player_bet = self.get_current_player().current_bet;

                                self.current_player_bet(player_stack - player_bet).expect("Should always be valid as long as get_possible_actions() contains this.");
                                Ok(PlayerAction::AllIn)
                            }

                            PlayerAction::Raise { value: v } | PlayerAction::Bet { value: v } => {
                                if self.current_player_bet(v).is_ok() {
                                    Ok(PlayerAction::Raise { value: v })
                                } else {
                                    Err(String::from("Bet invalid"))
                                }
                            }
                            PlayerAction::Call => {
                                let player_chip_count = self.get_current_player().get_stack();
                                let player_current_bet = self.get_current_player().current_bet;
                                let mut bet_arr = self.create_bet_array();
                                let mut highest_bet: u32 = u32::MAX;
                                if !bet_arr.is_empty() {
                                    bet_arr.sort_by(|a, b| b.cmp(a));
                                    highest_bet = bet_arr[0];
                                }
                                if player_current_bet == highest_bet {
                                    Ok(PlayerAction::Check)
                                } else if player_chip_count >= highest_bet {
                                    self.current_player_bet(highest_bet - player_current_bet).expect("Already handled case where player chip count is not enough to call.");
                                    Ok(PlayerAction::Call)
                                } else if !bet_arr.is_empty() {
                                    self.current_player_bet(player_chip_count - player_current_bet).expect("Should be impossible for player bet to exceed player chip count.");
                                    Ok(PlayerAction::AllIn)
                                } else {
                                    Err(String::from("No players found."))
                                }
                            }
                            PlayerAction::Check => {
                                //If current players bet matches highest bet in pool, then its ok
                                // Otherwise err
                                let player_current_bet = self.get_current_player().current_bet;
                                let mut bet_arr = self.create_bet_array();
                                let mut highest_bet: u32 = u32::MAX;
                                if !bet_arr.is_empty() {
                                    bet_arr.sort_by(|a, b| b.cmp(a));
                                    highest_bet = bet_arr[0];
                                }
                                if player_current_bet == highest_bet {
                                    Ok(PlayerAction::Check)
                                } else {
                                    Err(String::from("No players found."))
                                }
                            }
                            PlayerAction::Fold => {
                                self.get_current_player().has_folded = true;
                                Ok(PlayerAction::Fold)
                                //TODO: FOLDING MECHANICS ARE NOT IMPLEMENTED YET
                                // So while this block is technically correct (as of now)
                                // I want to revisit it once folding is finished.
                            }
                            _ => Err(String::from("Unsupported action.")),
                        };
                        // if not, continue
                        //Then match statement
                        break;
                    } else {
                        continue;
                    }
                }
            }
            //Prior to this is all logic for handling bets
            let bets = self.create_bet_array();
            let first = &bets[0];
            if bets.iter().all(|f| f == first) {
                break;
            } else {
                continue;
            }
        }
    }

    //Function to compare all hands, returns a vec of player(s) who won the round
    //This currently only finds highest matching hand types - I need to get it to distinguish I think? I'll look back at my old code!
    // I should be able to compare using the HandType actually
    // I could be wrong but I'll see!
    fn determine_hand_order(&self) -> Vec<usize> {
        let mut winning_indexes: Vec<usize> = Vec::new();
        //Array of hand values
        let player_values: Vec<u32> = self
            .players
            .iter()
            // .map(|p| p.get_hand().clone().hand_type.unwrap().hand_value() as u32)
            .map(|p| {
                let mut array = p.get_hand().get_hand();
                array.append(&mut self.river.clone());
                let h = Hand::from(array);
                h.hand_type.unwrap().hand_value() as u32
            })
            .collect();
        // println!("[DEBUG] PLAYER VALUES: {:?}", player_values);
        let mut largest_num_index: usize = 0;
        for num in 0..player_values.len() {
            match player_values[num].cmp(&player_values[largest_num_index]) {
                Ordering::Greater => {
                    winning_indexes = vec![num];
                    largest_num_index = num;
                }
                Ordering::Equal => {
                    winning_indexes.push(num);
                }
                Ordering::Less => {}
            }
        }
        if winning_indexes.len() > 1 {
            let mut final_numbers: Vec<usize> = Vec::new();
            let mut top_hand: usize = 0;
            for i in winning_indexes.clone() {
                match self.players[i]
                    .get_hand()
                    .hand_type
                    .cmp(&self.players[top_hand].get_hand().hand_type)
                {
                    Ordering::Greater => {
                        top_hand = i;
                        final_numbers = vec![winning_indexes[i]];
                    }
                    Ordering::Equal => {
                        final_numbers.push(winning_indexes[i]);
                    }
                    Ordering::Less => {}
                }
            }
            final_numbers
        } else {
            winning_indexes
        }
    }
    //Not 100% correct as winnings are not distributed correctly
    //When someone goes all in and a draw occurs iirc
    //But its otherwise correct
    //Need to fully test!
    //Need to implement:
    //  - side pots
    fn distribute_winnings(&mut self) {
        //If a player is all in, create side pot
        //Do this for ALL all-in players where num isn't different
        //Sub-pots nest/stack

        // let mut remaining_players: Vec<Player> = self.players.clone().into_iter().filter(|p| !p.has_folded).collect();
        for p in &mut self.players {
            if !p.has_folded {
                println!("{}", self.pot * p.current_bet / self.pot);
                p.up_stack(self.pot * p.current_bet / self.pot);
            }
        }

        // todo!();

        // For the lowest all-in player, remove their bet x num of players
        // from pot. Repeat going down up until no players are all in, then just
        // give them a percentage of pot based on remaining players.

        //Get winners from list of non-folded players
        //If that list is len 1, just distribute winnings to
        // that one player.
        // Otherwise, return max bets to remaining players and then proportionately split rest of pot
        // Could do this by summing max bets, then subtracting winning hands, then splitting it
        // Based on percentage of pot made up.
    }

    // ---------------------
    // SINGLE PLAYER ACTIONS
    // ----------------------
    fn get_current_player(&mut self) -> &mut Player {
        self.players.get_mut(self.table_pos as usize).unwrap()
    }
    ///I should be able to use this for all player actions
    /// (bet, call raise, all-in?, check, fold)
    fn current_player_bet(&mut self, value: u32) -> Result<(), String> {
        self.get_current_player().down_stack(value)
    }

    fn new_hand(&mut self) {
        self.distribute_winnings();
        self.deck = Deck::new();

        for player in &mut self.players {
            if player.get_stack() == 0 {
                player.has_folded = true;
                player.current_bet = 0;
            } else {
                player.has_folded = false;
                player.current_bet = 0;
                //DEAL HAND TO PLAYERS
            }
        }
        //Reset fold status
        //Reset bet status
        //Shuffle deck
        //Deal cards
    }
}



// THESE TESTS WERE WRITTEN BY AN LLM - Sourced from Claude. Reviewed by a human, ofc!
// Comprehensive test module for hand.rs

// !TODO - NEED TO REFACTOR KICKER PROBS
// IMPORTANT: HandType's PartialEq only compares hand_value() (the rank tier),
// not the enum's fields. That means `assert_eq!(result, HandType::Pair { .. })`
// only proves "this is a pair" — it does NOT verify the kicker or pair value
// are correct, even if you write specific values in the expected side.
// Wherever field correctness matters, these tests use `if let` + explicit
// field assertions instead of assert_eq! on the whole HandType.




#[cfg(test)]
mod test {
    use crate::card::suit::Suit;
    use crate::card::value::Value;
use crate::hand::hand_type::HandType;
    use super::*;

    // ---------------------------------
    // Small helper for terser test data
    // ---------------------------------
    fn c(suit: Suit, value: Value) -> Card {
        Card::new(suit, value)
    }

    // ----------------------
    // CONSTRUCTOR EDGE CASES
    // ----------------------

    #[test]
    fn from_empty_vec_has_no_hand_type() {
        let h = Hand::from(vec![]);
        assert_eq!(h.hand_type, None);
    }

    #[test]
    fn from_nonempty_vec_computes_hand_type_immediately() {
        let h = Hand::from(vec![
            c(Suit::Spade, Value::Ace),
            c(Suit::Spade, Value::King),
            c(Suit::Spade, Value::Queen),
            c(Suit::Spade, Value::Jack),
            c(Suit::Spade, Value::Ten),
        ]);
        assert!(h.hand_type.is_some());
    }

    #[test]
    fn default_hand_is_empty_with_max_five() {
        let h = Hand::default();
        assert_eq!(h.get_hand().len(), 0);
        assert_eq!(h.hand_type, None);
    }

    // -----------------------------
    // is_flush / is_straight UNITS
    // -----------------------------

    #[test]
    fn is_flush_true_for_same_suit() {
        let h = Hand::from(vec![
            c(Suit::Heart, Value::Two),
            c(Suit::Heart, Value::Five),
            c(Suit::Heart, Value::Nine),
            c(Suit::Heart, Value::Jack),
            c(Suit::Heart, Value::Ace),
        ]);
        assert!(h.is_flush());
    }

    #[test]
    fn is_flush_false_for_mixed_suits() {
        let h = Hand::from(vec![
            c(Suit::Heart, Value::Two),
            c(Suit::Club, Value::Five),
            c(Suit::Heart, Value::Nine),
            c(Suit::Heart, Value::Jack),
            c(Suit::Heart, Value::Ace),
        ]);
        assert!(!h.is_flush());
    }

    #[test]
    fn is_flush_false_on_empty_hand() {
        let h = Hand::from(vec![]);
        // is_flush is called on a mutable-borrowed method elsewhere but is &self-only here
        let empty_hand = Hand::default();
        assert!(!empty_hand.is_flush());
        let _ = h; // keep h alive to avoid unused warning if from() short-circuits
    }

    #[test]
    fn is_straight_true_for_sequential_run() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::Nine),
            c(Suit::Spade, Value::Eight),
            c(Suit::Heart, Value::Seven),
            c(Suit::Diamond, Value::Six),
            c(Suit::Club, Value::Five),
        ]);
        assert!(h.is_straight());
    }

    #[test]
    fn is_straight_true_for_wheel_ace_low() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::Ace),
            c(Suit::Spade, Value::Two),
            c(Suit::Heart, Value::Three),
            c(Suit::Diamond, Value::Four),
            c(Suit::Club, Value::Five),
        ]);
        assert!(h.is_straight());
    }

    #[test]
    fn is_straight_false_for_broken_sequence() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::Nine),
            c(Suit::Spade, Value::Eight),
            c(Suit::Heart, Value::Six), // gap here, no Seven
            c(Suit::Diamond, Value::Five),
            c(Suit::Club, Value::Four),
        ]);
        assert!(!h.is_straight());
    }

    #[test]
    fn is_straight_false_for_pair_plus_random() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::King),
            c(Suit::Spade, Value::King),
            c(Suit::Heart, Value::Nine),
            c(Suit::Diamond, Value::Five),
            c(Suit::Club, Value::Two),
        ]);
        assert!(!h.is_straight());
    }

    // ----------------------------------------------------
    // FULL determine_hand() COVERAGE, ONE PER HAND CATEGORY
    // ----------------------------------------------------

    #[test]
    fn royal_flush() {
        let mut h = Hand::from(vec![
            c(Suit::Spade, Value::Ace),
            c(Suit::Spade, Value::King),
            c(Suit::Spade, Value::Queen),
            c(Suit::Spade, Value::Jack),
            c(Suit::Spade, Value::Ten),
        ]);
        assert_eq!(h.determine_hand(), HandType::RoyalFlush);
    }

    #[test]
    fn straight_flush_not_ace_high() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::Nine),
            c(Suit::Club, Value::Eight),
            c(Suit::Club, Value::Seven),
            c(Suit::Club, Value::Six),
            c(Suit::Club, Value::Five),
        ]);
        match h.determine_hand() {
            HandType::StraightFlush { kicker } => assert_eq!(kicker, Value::Nine),
            other => panic!("expected StraightFlush, got {:?}", other),
        }
    }

    #[test]
    fn straight_flush_wheel() {
        // NOTE: current implementation reports the sorted-first card as the
        // kicker, which for a wheel straight is the Ace (numeric 14), not
        // the Five. This test documents actual behavior, not "correct"
        // poker ranking — flag this as a known quirk if you fix it later.
        let mut h = Hand::from(vec![
            c(Suit::Diamond, Value::Ace),
            c(Suit::Diamond, Value::Two),
            c(Suit::Diamond, Value::Three),
            c(Suit::Diamond, Value::Four),
            c(Suit::Diamond, Value::Five),
        ]);
        match h.determine_hand() {
            HandType::StraightFlush { kicker } => assert_eq!(kicker, Value::Ace),
            other => panic!("expected StraightFlush, got {:?}", other),
        }
    }

    #[test]
    fn four_of_a_kind_with_kicker() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::King),
            c(Suit::Diamond, Value::King),
            c(Suit::Heart, Value::King),
            c(Suit::Spade, Value::King),
            c(Suit::Club, Value::Two),
        ]);
        match h.determine_hand() {
            HandType::FourKind { kind_value, kicker } => {
                assert_eq!(kind_value, Value::King);
                assert_eq!(kicker, Value::Two);
            }
            other => panic!("expected FourKind, got {:?}", other),
        }
    }

    #[test]
    fn full_house_three_over_two() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::Queen),
            c(Suit::Diamond, Value::Queen),
            c(Suit::Heart, Value::Queen),
            c(Suit::Spade, Value::Four),
            c(Suit::Club, Value::Four),
        ]);
        match h.determine_hand() {
            HandType::FullHouse { three_value, two_value } => {
                assert_eq!(three_value, Value::Queen);
                assert_eq!(two_value, Value::Four);
            }
            other => panic!("expected FullHouse, got {:?}", other),
        }
    }

    #[test]
    fn flush_non_sequential() {
        let mut h = Hand::from(vec![
            c(Suit::Diamond, Value::Ace),
            c(Suit::Diamond, Value::Jack),
            c(Suit::Diamond, Value::Eight),
            c(Suit::Diamond, Value::Five),
            c(Suit::Diamond, Value::Three),
        ]);
        match h.determine_hand() {
            HandType::Flush { cards } => {
                assert_eq!(
                    cards,
                    vec![Value::Ace, Value::Jack, Value::Eight, Value::Five, Value::Three]
                );
            }
            other => panic!("expected Flush, got {:?}", other),
        }
    }

    #[test]
    fn straight_mixed_suits() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::Ten),
            c(Suit::Diamond, Value::Nine),
            c(Suit::Heart, Value::Eight),
            c(Suit::Spade, Value::Seven),
            c(Suit::Club, Value::Six),
        ]);
        match h.determine_hand() {
            HandType::Straight { kicker } => assert_eq!(kicker, Value::Ten),
            other => panic!("expected Straight, got {:?}", other),
        }
    }

    #[test]
    fn three_of_a_kind_no_pair_among_rest() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::Nine),
            c(Suit::Diamond, Value::Nine),
            c(Suit::Heart, Value::Nine),
            c(Suit::Spade, Value::King),
            c(Suit::Club, Value::Two),
        ]);
        match h.determine_hand() {
            HandType::ThreeKind { three_value, mut other_cards } => {
                assert_eq!(three_value, Value::Nine);
                other_cards.sort();
                assert_eq!(other_cards, vec![Value::Two, Value::King]);
            }
            other => panic!("expected ThreeKind, got {:?}", other),
        }
    }

    #[test]
    fn two_pair_with_kicker() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::King),
            c(Suit::Diamond, Value::King),
            c(Suit::Heart, Value::Four),
            c(Suit::Spade, Value::Four),
            c(Suit::Club, Value::Two),
        ]);
        match h.determine_hand() {
            HandType::TwoPair { pair_one, pair_two, kicker } => {
                let mut pairs = vec![pair_one, pair_two];
                pairs.sort();
                assert_eq!(pairs, vec![Value::Four, Value::King]);
                assert_eq!(kicker, Value::Two);
            }
            other => panic!("expected TwoPair, got {:?}", other),
        }
    }

    #[test]
    fn one_pair_with_three_kickers() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::Seven),
            c(Suit::Diamond, Value::Seven),
            c(Suit::Heart, Value::King),
            c(Suit::Spade, Value::Queen),
            c(Suit::Club, Value::Three),
        ]);
        match h.determine_hand() {
            HandType::Pair { pair, mut other_cards } => {
                assert_eq!(pair, Value::Seven);
                other_cards.sort();
                assert_eq!(other_cards, vec![Value::Three, Value::Queen, Value::King]);
            }
            other => panic!("expected Pair, got {:?}", other),
        }
    }

    #[test]
    fn high_card_no_matches() {
        let mut h = Hand::from(vec![
            c(Suit::Club, Value::Ace),
            c(Suit::Diamond, Value::Jack),
            c(Suit::Heart, Value::Eight),
            c(Suit::Spade, Value::Five),
            c(Suit::Club, Value::Two),
        ]);
        match h.determine_hand() {
            HandType::HighCard { cards } => {
                assert_eq!(
                    cards,
                    vec![Value::Ace, Value::Jack, Value::Eight, Value::Five, Value::Two]
                );
            }
            other => panic!("expected HighCard, got {:?}", other),
        }
    }

    // -----------------------------------------
    // HandType RANK ORDERING (hand_value + Ord)
    // -----------------------------------------

    #[test]
    fn hand_value_ranks_ascend_correctly() {
        assert!(HandType::HighCard { cards: vec![] }.hand_value() < HandType::Pair { pair: Value::Two, other_cards: vec![] }.hand_value());
        assert!(HandType::Pair { pair: Value::Two, other_cards: vec![] }.hand_value() < HandType::TwoPair { pair_one: Value::Two, pair_two: Value::Three, kicker: Value::Four }.hand_value());
        assert!(HandType::TwoPair { pair_one: Value::Two, pair_two: Value::Three, kicker: Value::Four }.hand_value() < HandType::ThreeKind { three_value: Value::Two, other_cards: vec![] }.hand_value());
        assert!(HandType::ThreeKind { three_value: Value::Two, other_cards: vec![] }.hand_value() < HandType::Straight { kicker: Value::Two }.hand_value());
        assert!(HandType::Straight { kicker: Value::Two }.hand_value() < HandType::Flush { cards: vec![] }.hand_value());
        assert!(HandType::Flush { cards: vec![] }.hand_value() < HandType::FullHouse { three_value: Value::Two, two_value: Value::Three }.hand_value());
        assert!(HandType::FullHouse { three_value: Value::Two, two_value: Value::Three }.hand_value() < HandType::FourKind { kind_value: Value::Two, kicker: Value::Three }.hand_value());
        assert!(HandType::FourKind { kind_value: Value::Two, kicker: Value::Three }.hand_value() < HandType::StraightFlush { kicker: Value::Two }.hand_value());
        assert!(HandType::StraightFlush { kicker: Value::Two }.hand_value() < HandType::RoyalFlush.hand_value());
    }

    #[test]
    fn different_tiers_compare_by_hand_value_even_with_weak_kicker() {
        // A pair with a low kicker still beats high card with an ace kicker.
        let pair = HandType::Pair { pair: Value::Two, other_cards: vec![Value::Three] };
        let high_card = HandType::HighCard { cards: vec![Value::Ace] };
        assert!(pair > high_card);
    }

    #[test]
    fn same_tier_breaks_tie_on_primary_value() {
        let low_pair = HandType::Pair { pair: Value::Three, other_cards: vec![Value::King] };
        let high_pair = HandType::Pair { pair: Value::Jack, other_cards: vec![Value::Two] };
        assert!(high_pair > low_pair);
    }

    #[test]
    fn same_tier_and_primary_value_breaks_tie_on_kickers() {
        let weak_kicker = HandType::Pair { pair: Value::Seven, other_cards: vec![Value::Two, Value::Three, Value::Four] };
        let strong_kicker = HandType::Pair { pair: Value::Seven, other_cards: vec![Value::Two, Value::Three, Value::King] };
        assert!(strong_kicker > weak_kicker);
    }

    #[test]
    fn full_house_ties_break_on_three_value_first() {
        let lower = HandType::FullHouse { three_value: Value::Four, two_value: Value::King };
        let higher = HandType::FullHouse { three_value: Value::Five, two_value: Value::Two };
        // Higher three-of-a-kind wins even though its pair is weaker.
        assert!(higher > lower);
    }

    // ---------------------------------
    // Hand-level Ord (used by Table for
    // determine_hand_order/showdown)
    // ---------------------------------

    #[test]
    fn hand_ordering_flush_beats_straight() {
        let flush = Hand::from(vec![
            c(Suit::Heart, Value::Two),
            c(Suit::Heart, Value::Five),
            c(Suit::Heart, Value::Nine),
            c(Suit::Heart, Value::Jack),
            c(Suit::Heart, Value::King),
        ]);
        let straight = Hand::from(vec![
            c(Suit::Club, Value::Ten),
            c(Suit::Diamond, Value::Nine),
            c(Suit::Heart, Value::Eight),
            c(Suit::Spade, Value::Seven),
            c(Suit::Club, Value::Six),
        ]);
        assert!(flush > straight);
    }

    #[test]
    fn hand_equality_is_by_tier_only_not_exact_hand() {
        // Documenting the existing PartialEq behavior: two Hands with the
        // same HandType tier are "equal" even if their actual cards differ,
        // because Hand::eq delegates to HandType::eq, which only compares
        // hand_value(). This is worth knowing before relying on `==`
        // anywhere in showdown/winner logic.
        let pair_low = Hand::from(vec![
            c(Suit::Club, Value::Three),
            c(Suit::Diamond, Value::Three),
            c(Suit::Heart, Value::King),
            c(Suit::Spade, Value::Queen),
            c(Suit::Club, Value::Two),
        ]);
        let pair_high = Hand::from(vec![
            c(Suit::Club, Value::Jack),
            c(Suit::Diamond, Value::Jack),
            c(Suit::Heart, Value::Nine),
            c(Suit::Spade, Value::Eight),
            c(Suit::Club, Value::Seven),
        ]);
        assert_eq!(pair_low, pair_high); // true today; see note above
    }

    // -----------------
    // draw_card BEHAVIOR
    // -----------------

    #[test]
    fn draw_card_respects_hand_max() { 
        let mut h = Hand {
            hand: vec![],
            hand_type: None,
            hand_max: 2,
        };
        h.draw_card(c(Suit::Club, Value::Two));
        h.draw_card(c(Suit::Diamond, Value::Three));
        h.draw_card(c(Suit::Heart, Value::Four)); // should be dropped, hand is full
        assert_eq!(h.get_hand().len(), 2);
    }

    #[test]
    fn draw_card_does_not_score_single_card_hand() {
        let mut h = Hand::default();
        h.draw_card(c(Suit::Club, Value::Ace));
        // hand_type only updates once hand.len() > 1
        assert_eq!(h.hand_type, None);
    }

    #[test]
    fn draw_card_scores_once_second_card_is_added() {
        let mut h = Hand::default();
        h.draw_card(c(Suit::Club, Value::Ace));
        h.draw_card(c(Suit::Diamond, Value::King));
        assert!(h.hand_type.is_some());
    }
}