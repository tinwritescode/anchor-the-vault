use anchor_lang::prelude::*;

declare_id!("3oj39XhPhs68TSn1JvLsavK1CG4NyQW7TG6R9EuE68Ze");

#[program]
pub mod plats_solana {
    use std::vec;

    use super::*;

    pub fn initialize_taskvault(
        ctx: Context<InitializeTaskVault>,
        taskvault_bump: u8,
        prize: u64,
    ) -> Result<()> {
        ctx.accounts.task_vault.bump = taskvault_bump;
        ctx.accounts.task_vault.prize = prize;
        ctx.accounts.task_vault.paid = vec![];
        ctx.accounts.task_vault.authority = ctx.accounts.authority.key();

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeTaskVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(init, seeds = [b"task_vault".as_ref()], bump, payer = authority, space = TaskVault::SIZE)]
    pub task_vault: Account<'info, TaskVault>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct TaskVault {
    pub authority: Pubkey,
    pub token_deposit: u64,
    pub prize: u64,
    pub bump: u8,
    pub paid: Vec<Pubkey>,
}

impl TaskVault {
    const SIZE: usize = 8 + 32 + 8 + 8 + 1 + 9000; // 9000 is big enough
}
