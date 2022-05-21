use anchor_lang::prelude::*;

#[account]
pub struct Task {
    pub authority: Pubkey,
    pub mint_of_token_being_sent: Pubkey,
    pub task_id: u64,
}

impl Task {
    pub const SIZE: usize = 8 + 32 + 32 + 8;
}
