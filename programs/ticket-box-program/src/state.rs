use anchor_lang::prelude::*;

#[account]
pub struct TicketBox {
    pub uuid: String,
    pub creator: Pubkey,
    pub name: String,
    pub uri: String,
    pub start_at: i64,
    pub end_at: Option<i64>,         // 0 for unlimit
    pub num_of_tickets: Option<i64>, // 0 for unlimit
    pub num_of_sold: i64,
    pub ticket_per_wallet: Option<i64>,
    pub currency: Option<Pubkey>, // None for sol
    pub price: u64, // 0 for free
    pub transferable: bool,
    pub escrow: Pubkey,
}

impl TicketBox {
    pub const SIZE: usize = 8 // discriminator
    + (4 + 256) // uuid, max = 256
    + 32 // creator
    + (4 + 256) // name, max = 256
    + (4 + 1000) // uri, max = 1000
    + 8 // start_at
    + 8 // end_at
    + 8 // num_of_tickets
    + 8 // num_of_sold
    + 4 // ticket_per_wallet
    + 32 // currency
    + 8 // price
    + 1 // transferable
    + 32; // escrow
}

#[account]
pub struct CollectionPda {
    pub authority: Pubkey,
    pub mint: Pubkey,
    // pub bump: u8,
}