use anchor_lang::prelude::*;

pub use crate::errors::ErrorCode;
pub use crate::schemas::task_vault::*;

pub fn exec(ctx: Context<WithdrawFromTheVault>, amount: u64) -> Result<()> {
    let task_vault = &mut ctx.accounts.task_vault;
    let authority_token_account = &mut ctx.accounts.authority_token_account;
    let task_vault_token_account = &mut ctx.accounts.task_vault_token_account;
    let treasurer = &ctx.accounts.treasurer;

    if task_vault.token_deposit < amount {
        return err!(ErrorCode::NotEnoughTokensFromTheVault);
    }

    msg!("Token deposit {}", task_vault.token_deposit);

    task_vault.token_deposit -= amount;

    // Below is the actual instruction that we are going to send to the Token program.
    // let seeds: &[&[&[u8]]] = &[&[
    //     "treasurer".as_ref(),
    //     &task_vault.key().to_bytes(),
    //     &[task_vault.bump],
    // ]];
    let seeds: &[&[&[u8]]] = &[&[
        "treasurer".as_ref(),
        &task_vault.key().to_bytes(),
        &[*ctx.bumps.get("treasurer").unwrap()],
    ]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: task_vault_token_account.to_account_info().clone(),
            to: authority_token_account.to_account_info().clone(),
            authority: treasurer.to_account_info().clone(),
        },
        seeds,
    );

    msg!(
        "Withdrawing {} tokens from {} to {}",
        amount,
        task_vault_token_account.key(),
        authority_token_account.key()
    );

    anchor_spl::token::transfer(cpi_ctx, amount)?;

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawFromTheVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // aka sender

    #[account(mut, has_one = authority)]
    pub task_vault: Account<'info, TaskVault>,

    #[account(seeds = [b"treasurer", &task_vault.key().to_bytes()], bump)]
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
