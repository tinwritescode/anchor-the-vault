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
        amount: u64,
    ) -> Result<()> {
        let task_vault = &mut ctx.accounts.task_vault;
        let authority = &ctx.accounts.authority;
        let task_vault_token_account = &mut ctx.accounts.task_vault_token_account;
        let authority_token_account = &mut ctx.accounts.authority_token_account;

        task_vault.bump = taskvault_bump;
        task_vault.prize = prize;
        task_vault.paid = vec![];
        task_vault.authority = authority.key();
        task_vault.token_deposit = amount;

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

    pub fn deposit_to_the_vault(ctx: Context<DepositToTheVault>, amount: u64) -> Result<()> {
        let task_vault = &mut ctx.accounts.task_vault;
        let authority_token_account = &mut ctx.accounts.authority_token_account;
        let task_vault_token_account = &mut ctx.accounts.task_vault_token_account;
        let authority = &ctx.accounts.authority;

        task_vault.token_deposit += amount;

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

    pub fn withdraw_from_the_vault(
        ctx: Context<WithdrawFromTheVault>,
        amount: u64,
        nonce: u8,
    ) -> Result<()> {
        let task_vault = &mut ctx.accounts.task_vault;
        let authority_token_account = &mut ctx.accounts.authority_token_account;
        let task_vault_token_account = &mut ctx.accounts.task_vault_token_account;

        assert!(
            task_vault.token_deposit >= amount,
            "Not enough tokens in the vault"
        );

        task_vault.token_deposit -= amount;

        // Below is the actual instruction that we are going to send to the Token program.
        let transfer_instruction = anchor_spl::token::Transfer {
            from: task_vault_token_account.to_account_info(), // wallet to withdraw from
            to: authority_token_account.to_account_info(),
            authority: ctx.accounts.treasurer.to_account_info(),
        };

        let seeds: &[&[&[u8]]] = &[&[
            "task_vault_treasurer".as_ref(),
            &task_vault.key().to_bytes(),
            &[*ctx.bumps.get("task_vault_treasurer").unwrap()],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            seeds,
        );

        msg!(
            "Transferring {} tokens from {} to {}",
            amount,
            task_vault_token_account.key(),
            authority_token_account.key()
        );

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeTaskVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // aka sender

    #[account(seeds = [b"task_vault_treasurer".as_ref(), &task_vault.key().to_bytes()], bump)]
    /// CHECK: Just a pure account
    pub treasurer: AccountInfo<'info>,
    pub mint_of_token_being_sent: Box<Account<'info, anchor_spl::token::Mint>>,

    #[account(init, seeds = [b"task_vault".as_ref()], bump, payer = authority, space = TaskVault::SIZE)]
    pub task_vault: Account<'info, TaskVault>,
    #[account(init, payer = authority, associated_token::mint = mint_of_token_being_sent, associated_token::authority = treasurer)]
    pub task_vault_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = authority)]
    pub authority_token_account: Account<'info, anchor_spl::token::TokenAccount>, // aka: wallet to withdraw from

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
}

#[account]
pub struct TaskVault {
    pub authority: Pubkey,
    pub token_deposit: u64,
    pub prize: u64,
    pub bump: u8, //temporary, useless, remove it later
    pub paid: Vec<Pubkey>,
}

impl TaskVault {
    const SIZE: usize = 8 + 32 + 8 + 8 + 1 + 9000; // 9000 is big enough
}

#[derive(Accounts)]
pub struct DepositToTheVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // aka sender

    #[account(mut)]
    pub task_vault: Account<'info, TaskVault>,

    #[account(seeds = [b"task_vault_treasurer".as_ref(), &task_vault.key().to_bytes()], bump)]
    /// CHECK: Just a pure account
    pub treasurer: AccountInfo<'info>,

    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = treasurer)]
    pub task_vault_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = authority)]
    pub authority_token_account: Account<'info, anchor_spl::token::TokenAccount>, // aka: wallet to withdraw from

    pub mint_of_token_being_sent: Box<Account<'info, anchor_spl::token::Mint>>,

    // Programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

#[derive(Accounts)]
pub struct WithdrawFromTheVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // aka sender

    #[account(mut)]
    pub task_vault: Account<'info, TaskVault>,

    #[account(seeds = [b"task_vault_treasurer".as_ref(), &task_vault.key().to_bytes()], bump)]
    /// CHECK: Just a pure account
    pub treasurer: AccountInfo<'info>,

    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = treasurer)]
    pub task_vault_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    #[account(mut, associated_token::mint = mint_of_token_being_sent, associated_token::authority = authority)]
    pub authority_token_account: Account<'info, anchor_spl::token::TokenAccount>, // aka: wallet to withdraw from

    pub mint_of_token_being_sent: Box<Account<'info, anchor_spl::token::Mint>>,

    // Programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}
