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
    fn next_player(&mut self) -> &Player {
        self.table_pos += 1;
        self.players.get(self.table_pos as usize).unwrap()
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
        } else if player_fold_status {
            vec![]
        } else {
            //Unsure if any way to reach this, I assume not
            //But just to be safe I will return vec![]
            eprint!("This should be impossible to reach!");
            vec![PlayerAction::Fold]
        }
    }

    pub fn handle_input(
        &mut self,
        get_input: &dyn Fn(usize) -> Result<PlayerAction, String>,
        num: usize,
    ) -> Result<PlayerAction, String> {
        let p_action = get_input(num).unwrap();
        println!("ACTION CHOSEN: {:?}", p_action);
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

#[cfg(test)]
mod test {
    use crate::{
        card::{suit::Suit, value::Value},
        hand::Hand,
    };

    use super::*;

    #[test]
    fn constructor_test() {
        let mut _table = Table::default();
        let mut _table2: Table = Table::new(5, 10, 20);
    }

    #[test]
    fn test_big_blind() {
        let mut table: Table = Table::new(5, 10, 20);
        assert_eq!(table.big_blind, 20);
        table.increase_blinds(10);
        assert_eq!(table.big_blind, 30);
    }

    #[test]
    fn test_small_blind() {
        let mut table: Table = Table::new(5, 10, 20);
        assert_eq!(table.small_blind, 10);
        table.increase_blinds(10);
        // assert_eq!(table.small_blind, 20);
    }

    #[test]
    fn dealer_test() {
        let mut table = Table::new(5, 10, 20);
        table.deal_cards();
        for player in table.players {
            assert_eq!(player.get_hand().get_hand().len(), 2);
        }
    }

    #[test]
    fn valid_actions_test() {
        //Player with bet equal to highest bet
        let mut table_1: Table = Table::new(2, 10, 20);
        table_1.players[0].current_bet = 500;
        assert_eq!(
            table_1.get_possible_actions(0),
            vec![
                PlayerAction::Fold,
                PlayerAction::Check,
                PlayerAction::Bet { value: 0 },
                PlayerAction::AllIn
            ]
        );
        //Player with bet lower than highest bet, but enough chips
        let mut table_2: Table = Table::new(5, 10, 20);
        table_2.players[1].current_bet = 500;
        table_2.players[0].up_stack(5000);
        assert_eq!(
            table_2.get_possible_actions(0),
            vec![
                PlayerAction::Fold,
                PlayerAction::Call,
                PlayerAction::Raise { value: 0 },
                PlayerAction::AllIn
            ]
        );
        //Player with bet lower than highest bet and not enough chips
        let mut table_3: Table = Table::new(5, 10, 20);
        table_3.players[0]
            .down_stack(1500)
            .expect("Nothing bad will happen!");
        table_3.players[1].current_bet = 2000;
        assert_eq!(
            table_3.get_possible_actions(0),
            vec![PlayerAction::Fold, PlayerAction::AllIn]
        );
    }

    #[test]
    fn player_action_test() {
        let mut table = Table::new(1, 10, 20);
        let mut table_2: Table = Table::new(2, 10, 20);
        //Player folding successfully
        fn return_fold(_i: usize) -> Result<PlayerAction, String> {
            Ok(PlayerAction::Fold)
        }
        assert_eq!(table.handle_input(&return_fold, 0), Ok(PlayerAction::Fold));
        //Player checking successfully
        fn return_check(_i: usize) -> Result<PlayerAction, String> {
            Ok(PlayerAction::Check)
        }
        assert_eq!(
            table.handle_input(&return_check, 0),
            Ok(PlayerAction::Check)
        );
        //Player checking unsuccessfully
        // todo!();

        //Player calling successfully
        fn return_call(_i: usize) -> Result<PlayerAction, String> {
            Ok(PlayerAction::Call)
        }
        table_2.players[1].current_bet = 500;
        assert_eq!(
            table_2.handle_input(&return_call, 0),
            Ok(PlayerAction::Call)
        );
        //Player calling unsuccessfully
        // todo!();

        //Player raising/betting successfully
        fn return_raise(_: usize) -> Result<PlayerAction, String> {
            Ok(PlayerAction::Raise { value: 250 })
        }
        assert_eq!(
            table.handle_input(&return_raise, 0),
            Ok(PlayerAction::Raise { value: 250 })
        );

        //Player raising/betting unsuccessfully
        // todo!();

        //Player going all in successfully
        fn return_all_in(_: usize) -> Result<PlayerAction, String> {
            Ok(PlayerAction::AllIn)
        }
        assert_eq!(
            table.handle_input(&return_all_in, 0),
            Ok(PlayerAction::AllIn)
        );
    }

    #[test]
    fn hand_ordering_test() {
        //TODO: Clean up so I am not constructing unnecessary hands
        let winning_hand: Hand = Hand::from(vec![
            Card::new(Suit::Diamond, Value::Ten),
            Card::new(Suit::Spade, Value::Nine),
        ]);
        let true_winning_hand: Hand = Hand::from(vec![
            Card::new(Suit::Club, Value::Ace),
            Card::new(Suit::Diamond, Value::Ten),
        ]);
        let river = vec![
            Card::new(Suit::Club, Value::Four),
            Card::new(Suit::Diamond, Value::King),
            Card::new(Suit::Spade, Value::Queen),
            Card::new(Suit::Diamond, Value::Jack),
            Card::new(Suit::Spade, Value::Seven),
        ];
        let mut winning_player: Player = Player::default();
        winning_player.mut_hand().set_hand(winning_hand.get_hand());

        let mut table: Table = Table::new(5, 10, 20);
        for i in 1..table.players.len() {
            table.players[i]
                .mut_hand()
                .draw_card(Card::new(Suit::Club, Value::Eight));
            table.players[i]
                .mut_hand()
                .draw_card(Card::new(Suit::Diamond, Value::Two));
        }
        table.players[0] = winning_player.clone();
        table.players[0].mut_hand().hand_type =
            Some(table.players[0].clone().mut_hand().determine_hand());
        table.river = river;
        assert_eq!(table.determine_hand_order(), vec![0]);

        //Example hand with clear winner based on kicker
        table.players[1].mut_hand().set_hand(vec![]);
        table.players[1]
            .mut_hand()
            .draw_card(Card::new(Suit::Club, Value::Ace));
        table.players[1]
            .mut_hand()
            .draw_card(Card::new(Suit::Club, Value::Ten));
        table.players[1].mut_hand().hand_type =
            Some(table.players[1].clone().mut_hand().determine_hand());
        println!(
            "DEBUG:TABLE player #1 {:?}",
            table.players[1].get_hand().hand_type
        );
        assert_eq!(table.determine_hand_order(), vec![1]);

        //Example hand with multiple winners
        table.players[1] = winning_player;
        table.players[1].mut_hand().hand_type =
            Some(table.players[1].clone().mut_hand().determine_hand());
        println!(
            "DEBUG:TABLE player #1 {:?}",
            table.players[1].get_hand().hand_type
        );
        assert_eq!(table.determine_hand_order(), vec![0, 1]);
    }

    #[test]
    fn winnings_test() {
        let mut table = Table::new(5, 10, 20);
        for mut i in 0..table.players.len() {
            table.players[i].current_bet = 1000;
        }
        table.pot = 5000;
        table.distribute_winnings();
        assert_eq!(table.players[0].get_stack(), 3000);
        //Distribute winnings and side pot
        // todo!();
    }
}
