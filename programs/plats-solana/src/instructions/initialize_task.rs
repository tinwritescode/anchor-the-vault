use anchor_lang::prelude::*;

pub use crate::schemas::task::*;
pub use crate::schemas::task_vault::*;

pub fn exec(ctx: Context<InitializeTask>, prize: u64, amount: u64) -> Result<()> {
    Ok(())
}

pub struct InitializeTask<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Task::SIZE,
      )]
    pub task: Account<'info, Task>,

    #[account(seeds = [b"task".as_ref(), &task.key().to_bytes()], bump)]
    /// CHECK: Just a pure account
    pub treasurer: AccountInfo<'info>,
    pub mint: Box<Account<'info, token::Mint>>,
}
