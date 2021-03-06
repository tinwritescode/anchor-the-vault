use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Not enough tokens from the vault.")]
    NotEnoughTokensFromTheVault,
    #[msg("Already paid")]
    AlreadyPaid,
}
