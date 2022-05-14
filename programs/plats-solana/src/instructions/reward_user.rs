use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::TaskVault;

pub fn exec(ctx: Context<RewardUser>) -> Result<()> {
    let task_vault = &mut ctx.accounts.task_vault;
    let user_to_reward = &mut ctx.accounts.user_to_reward;
    let task_vault_token_account = &mut ctx.accounts.task_vault_token_account;
    let user_to_reward_token_account = &mut ctx.accounts.user_to_reward_token_account;
    let treasurer = &ctx.accounts.treasurer;

    if task_vault.paid_to.contains(&user_to_reward.key()) {
        return err!(ErrorCode::AlreadyPaid);
    }

    task_vault.paid_to.push(user_to_reward.key());

    let amount = {
        if task_vault.prize > task_vault.token_deposit {
            task_vault.token_deposit
        } else {
            task_vault.prize
        }
    };
    task_vault.token_deposit -= amount;

    // Below is the actual instruction that we are going to send to the Token program.
    let seeds: &[&[&[u8]]] = &[&[
        "treasurer".as_ref(),
        &task_vault.key().to_bytes(),
        &[*ctx.bumps.get("treasurer").unwrap()],
    ]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: task_vault_token_account.to_account_info().clone(),
            to: user_to_reward_token_account.to_account_info().clone(),
            authority: treasurer.to_account_info().clone(),
        },
        seeds,
    );

    msg!(
        "Rewarding {} tokens from {} to {}",
        amount,
        task_vault_token_account.key(),
        user_to_reward.key()
    );

    anchor_spl::token::transfer(cpi_ctx, amount)?;

    Ok(())
}

#[derive(Accounts)]
pub struct RewardUser<'info> {
    #[account(mut)]
    pub reward_account: Signer<'info>, // aka sender

    #[account(mut)]
    /// CHECK: The user account that is being rewarded
    pub user_to_reward: AccountInfo<'info>,

    #[account(mut, has_one = reward_account)]
    pub task_vault: Account<'info, TaskVault>,

    #[account(seeds = [b"treasurer", &task_vault.key().to_bytes()], bump)]
    /// CHECK: Just a pure account
    pub treasurer: AccountInfo<'info>,

    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = treasurer)]
    pub task_vault_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = user_to_reward)]
    pub user_to_reward_token_account: Account<'info, anchor_spl::token::TokenAccount>, // aka: wallet to withdraw from

    #[account(mut)]
    pub mint_of_token_being_sent: Box<Account<'info, anchor_spl::token::Mint>>,

    // Programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}
