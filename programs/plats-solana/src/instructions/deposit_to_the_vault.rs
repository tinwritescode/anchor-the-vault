use anchor_lang::prelude::*;

pub use crate::schemas::task_vault::*;

pub fn exec(ctx: Context<DepositToTheVault>, amount: u64) -> Result<()> {
    let task_vault = &mut ctx.accounts.task_vault;
    let authority_token_account = &mut ctx.accounts.authority_token_account;
    let task_vault_token_account = &mut ctx.accounts.task_vault_token_account;
    let authority = &ctx.accounts.authority;

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

    task_vault.token_deposit += amount;

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawFromTheVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // aka sender

    #[account(mut)]
    pub task_vault: Account<'info, TaskVault>,

    #[account(seeds = [b"treasurer".as_ref(), &task_vault.key().to_bytes()], bump)]
    /// CHECK: Just a pure account
    pub treasurer: AccountInfo<'info>,

    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = treasurer)]
    pub task_vault_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = authority)]
    pub authority_token_account: Account<'info, anchor_spl::token::TokenAccount>, // aka: wallet to withdraw from

    #[account(mut)]
    pub mint_of_token_being_sent: Box<Account<'info, anchor_spl::token::Mint>>,

    // Programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

#[derive(Accounts)]
pub struct DepositToTheVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // aka sender

    #[account(mut, has_one = mint_of_token_being_sent)]
    pub task_vault: Account<'info, TaskVault>,

    #[account(seeds = [b"treasurer".as_ref(), &task_vault.key().to_bytes()], bump)]
    /// CHECK: Just a pure account
    pub treasurer: AccountInfo<'info>,
    pub mint_of_token_being_sent: Box<Account<'info, anchor_spl::token::Mint>>,

    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = treasurer)]
    pub task_vault_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = authority)]
    pub authority_token_account: Account<'info, anchor_spl::token::TokenAccount>, // aka: wallet to withdraw from

    // Programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}
