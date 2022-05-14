use anchor_lang::prelude::*;

#[account]
pub struct TaskVault {
    pub authority: Pubkey,
    pub token_deposit: u64,
    pub prize: u64,
    pub paid: Vec<Pubkey>,
    pub bump: u8,
    pub mint_of_token_being_sent: Pubkey,
}

impl TaskVault {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 1 + 32 + 9000; // 9000 is big enough
}
