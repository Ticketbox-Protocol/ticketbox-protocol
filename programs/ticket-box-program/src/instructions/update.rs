use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::TicketBox;

#[derive(Accounts)]
pub struct UpdateTicketBox<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
       mut,
       has_one = creator,
        seeds = [b"ticket_box", ticket_box.uuid.as_bytes(), creator.key().as_ref()],
        bump,
    )]
    pub ticket_box: Account<'info, TicketBox>,
    // native
    // pub system_program: Program<'info, System>,
    // pub rent: Sysvar<'info, Rent>,
    // pub token_program: Program<'info, token::Token>,
    // pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

pub fn handler(
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
    if name.is_some() {
        ctx.accounts.ticket_box.name = name.unwrap();
    }

    if uri.is_some() {
        ctx.accounts.ticket_box.uri = uri.unwrap();
    }

    if start_at.is_some() {
        let start_at = start_at.unwrap();

        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        require!(
            start_at >= current_timestamp,
            ErrorCode::InvalidTicketBoxStartAt
        );
        ctx.accounts.ticket_box.start_at = start_at;
    }

    if end_at.is_some() {
        let end_at = end_at.unwrap();
        require!(
            end_at > ctx.accounts.ticket_box.start_at,
            ErrorCode::InvalidTicketBoxEndAt
        );
        ctx.accounts.ticket_box.end_at = Some(end_at);
    }

    if num_of_tickets.is_some() {
        let num_of_tickets = num_of_tickets.unwrap();
        require!(
            ticket_per_wallet.is_some(),
            ErrorCode::InvalidTicketPerWallet
        );

        let ticket_per_wallet = ticket_per_wallet.unwrap();
        require!(
            num_of_tickets > ticket_per_wallet,
            ErrorCode::InvalidTicketPerWallet
        );

        ctx.accounts.ticket_box.num_of_tickets = Some(num_of_tickets);
        ctx.accounts.ticket_box.ticket_per_wallet = Some(ticket_per_wallet);
    }

    if transferable.is_some() {
        ctx.accounts.ticket_box.transferable = transferable.unwrap();
    }

    if price.is_some() {
        ctx.accounts.ticket_box.price = price.unwrap();
    }

    if transferable.is_some() {
        ctx.accounts.ticket_box.transferable = transferable.unwrap();
    }

    // TODO update collection nft

    Ok(())
}
