#[derive(Debug, PartialEq, Eq)]
pub enum PlayerAction {
    Fold,
    Check,
    Call,
    Raise { value: u32 },
    Bet { value: u32 },
    CheckFold,
    AllIn,
}
