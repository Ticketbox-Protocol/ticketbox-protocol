use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Token Account Owner")]
    InvalidTokenAccountOwner,
    #[msg("Invalid Token Account Mint")]
    InvalidTokenAccountMint,
    #[msg("Invalid Ticket Box `start_at`")]
    InvalidTicketBoxStartAt,
    #[msg("Invalid Ticket Box `end_at`")]
    InvalidTicketBoxEndAt,
    #[msg("`ticket_per_wallet` has to smaller than `num_of_tickets`")]
    InvalidTicketPerWallet,
    #[msg("Invalid Ticket Valid")]
    InvalidTicketPrice,
    #[msg("Sold out")]
    SoldOut,
    #[msg("Event ended")]
    EventEnded,
    #[msg("Account is not initialized!")]
    Uninitialized,
    #[msg("Public key mismatch")]
    PublicKeyMismatch,
    #[msg("Not enough tokens to pay for this minting")]
    NotEnoughTokens,
    #[msg("Not enough SOL to pay for this minting")]
    NotEnoughSOL,
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    #[msg("Mint Mismatch!")]
    MintMismatch,
}
