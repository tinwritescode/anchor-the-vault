use anchor_lang::prelude::*;

#[account]
pub struct TaskVault {
    pub authority: Pubkey,
    pub token_deposit: u64,
    pub prize: u64,
    pub paid_to: Vec<Pubkey>,
    // task id is text of 30 chracters
    pub task_id: String,
    pub mint_of_token_being_sent: Pubkey,
    pub reward_account: Pubkey,
}

impl TaskVault {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 1 + 32 + 9000; // 9000 is big enough
}
