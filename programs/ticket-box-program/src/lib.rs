use anchor_lang::prelude::*;

pub mod state;
pub use state::*;

pub mod utils;
pub use utils::*;

pub mod errors;
pub use errors::*;

pub mod instructions;
pub use instructions::*;

declare_id!("9oaNngp1cLnRchZRqbA3ubz1mUx5kWv4TNkpNq41Vwqc");

#[program]
pub mod ticket_box_program {

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        uuid: String,
        name: String,
        uri: String,
        start_at: i64,
        end_at: Option<i64>,
        num_of_tickets: Option<i64>,
        ticket_per_wallet: Option<i64>,
        price: u64,
        transferable: bool,
    ) -> Result<()> {
        initialize::handler(
            ctx,
            uuid,
            name,
            uri,
            start_at,
            end_at,
            num_of_tickets,
            ticket_per_wallet,
            price,
            transferable,
        )
    }

    pub fn update(
        ctx: Context<UpdateTicketBox>,
        name: Option<String>,
        uri: Option<String>,
        start_at: Option<i64>,
        end_at: Option<i64>,
        num_of_tickets: Option<i64>,
        ticket_per_wallet: Option<i64>,
        price: Option<u64>,
        transferable: Option<bool>,
    ) -> Result<()> {
        update::handler(
            ctx,
            name,
            uri,
            start_at,
            end_at,
            num_of_tickets,
            ticket_per_wallet,
            price,
            transferable,
        )
    }

    pub fn mint<'info>(
        ctx: Context<'_, '_, '_, 'info, MintTicket<'info>>,
        uri: String,
    ) -> Result<()> {
        mint::handler(ctx, uri)
    }
}
