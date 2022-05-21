use anchor_lang::prelude::*;

pub use crate::schemas::task_vault::*;
use crate::Task;

pub fn exec(ctx: Context<InitializeTaskVault>, prize: u64, amount: u64) -> Result<()> {
    let task_vault = &mut ctx.accounts.task_vault;
    let authority = &ctx.accounts.authority;
    let task_vault_token_account = &mut ctx.accounts.task_vault_token_account;
    let authority_token_account = &mut ctx.accounts.authority_token_account;
    let mint_of_token_being_sent = &mut ctx.accounts.mint_of_token_being_sent;
    let reward_account = &ctx.accounts.reward_account;

    task_vault.prize = prize;
    task_vault.paid_to = vec![];
    task_vault.authority = authority.key();
    task_vault.token_deposit = amount;
    task_vault.mint_of_token_being_sent = mint_of_token_being_sent.key();
    task_vault.reward_account = reward_account.to_account_info().key();

    // Below is the actual instruction that we are going to send to the Token program.
    let transfer_instruction = anchor_spl::token::Transfer {
        from: authority_token_account.to_account_info(), // wallet to withdraw from
        to: task_vault_token_account.to_account_info(),
        authority: authority.to_account_info().clone(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
    );

    msg!(
        "Transferring {} tokens from {} to {}",
        amount,
        authority_token_account.key(),
        task_vault.key()
    );
    // The `?` at the end will cause the function to return early in case of an error.
    // This pattern is common in Rust.
    anchor_spl::token::transfer(cpi_ctx, task_vault.token_deposit)?;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeTaskVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // aka sender

    #[account(seeds = [b"treasurer".as_ref(), &task_vault.key().to_bytes()], bump)]
    /// CHECK: Just a pure account
    pub treasurer: AccountInfo<'info>,
    pub mint_of_token_being_sent: Box<Account<'info, anchor_spl::token::Mint>>,
    /// CHECK: The system account that can be used to reward to the user
    pub reward_account: AccountInfo<'info>,

    #[account(has_one = mint_of_token_being_sent, has_one = authority)]
    pub task: Account<'info, Task>,

    #[account(init, seeds = [b"task_vault".as_ref(), &task.key().to_bytes()], bump, payer = authority, space = TaskVault::SIZE)]
    pub task_vault: Account<'info, TaskVault>,
    #[account(init, payer = authority, associated_token::mint = mint_of_token_being_sent, associated_token::authority = treasurer)]
    pub task_vault_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = authority)]
    pub authority_token_account: Account<'info, anchor_spl::token::TokenAccount>, // aka: wallet to withdraw from

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}
