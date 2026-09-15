use crate::table::Table;

mod card;
mod deck;
mod hand;
mod player;
mod table;

use std::io;

fn main() {
    let mut buffer = String::new();
    println!("Configure table:");
    println!("Enter the number of players: ");
    io::stdin().read_line(&mut buffer).unwrap();
    let player_num: usize = buffer.trim().parse().unwrap();
    buffer.clear();

    println!("Enter the small blind: ");
    io::stdin().read_line(&mut buffer).unwrap();
    let small_blind: usize = buffer.trim().parse().unwrap();
    buffer.clear();

    println!("Enter the big blind: ");
    io::stdin().read_line(&mut buffer).unwrap();
    let big_blind: usize = buffer.trim().parse().unwrap();
    buffer.clear();

    let mut table = Table::new(player_num, small_blind as u32, big_blind as u32);
    println!("{}", table.players.len());
    table.deal_cards();
    for player in table.players {
        println!("{:?}", player)
    }
    //Betting Round(s)
    //Check for move on?
    //
    


    // ---------------------------------------
    // Loop start:
    // Push info for all characters
    // For player in players
    // Check action
    // Respond to action
    // Print response
    // End loop
    // Check if all bets are equal or smth?
    // If all bets aren't equal, repeat betting loop
    // If current stage is at final stage, and all bets are equal
    // Distribute winnings n do final steps
    // -----------------------------------------

    /*
       LOOP
           FOR PLAYER IN PLAYERS
               CHECK PLAYER ACTION
                   IF VALID
               DO PLAYER ACTION
                   IF NOT
                       RETRY??
               PRINT RESPONSE
           END
           IF ALL BETS EQUAL
               REPEAT LOOP TILL BETS EQUAL
           ELSE CONT
           IF CURRENT STAGE AT FINAL STAGE, AND ALL BETS EQUAL
               EXIT LOOP SUCCESSFULLY
           ELSE
               DISTRIBUTE WINNINGS, FINAL STEPS TO PREP FOR NEXT HAND!
    */
}

//Break loop based on a certain input OR when chip stack = 0
//Change something so when chip stack is = 0 players are permafolded!
//man i love working on this but I really don't want to rn!
//Probably add a function to Table to represent this? Or create a file Game which handles and loops
//all game logic. That'll probs be better. Lets do it in main for now!
