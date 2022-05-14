use anchor_lang::prelude::*;
declare_id!("3oj39XhPhs68TSn1JvLsavK1CG4NyQW7TG6R9EuE68Ze");

pub mod instructions;
pub use instructions::*;

pub mod schemas;
pub use schemas::*;

pub mod errors;
pub use errors::*;

#[program]
pub mod plats_solana {
    use super::*;

    pub fn initialize_taskvault(
        ctx: Context<InitializeTaskVault>,
        bump: u8,
        prize: u64,
        amount: u64,
    ) -> Result<()> {
        initialize_taskvault::exec(ctx, bump, prize, amount)
    }

    pub fn deposit_to_the_vault(ctx: Context<DepositToTheVault>, amount: u64) -> Result<()> {
        deposit_to_the_vault::exec(ctx, amount)
    }

    pub fn withdraw_from_the_vault(ctx: Context<WithdrawFromTheVault>, amount: u64) -> Result<()> {
        withdraw_from_the_vault::exec(ctx, amount)
    }
}
