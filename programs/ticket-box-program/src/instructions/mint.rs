use {
    anchor_lang::{
        prelude::*,
        solana_program::{
            program::{invoke, invoke_signed},
            system_instruction,
        },
    },
    anchor_spl::{associated_token, token},
    mpl_token_metadata::{
        instruction as mpl_instruction, state::Collection,
        ID as MPL_TOKEN_METADATA_ID,
    },
    std::vec,
};

use crate::{assert_is_ata, errors::ErrorCode, TokenTransferParams};
use crate::{spl_token_transfer, TicketBox};

// use std::vec;
// use anchor_lang::solana_program::program::invoke;
// use mpl_token_metadata::{state::Collection, utils::assert_derivation};

#[derive(Accounts)]
pub struct MintTicket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"ticket_box", ticket_box.uuid.as_bytes(), ticket_box.creator.as_ref()],
        bump
    )]
    pub ticket_box: Account<'info, TicketBox>,

    /// CHECK: wallet can be any account and is not written to or read
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,
    //ticket
    #[account(mut)]
    pub ticket_mint: Signer<'info>,
    /// CHECK: We're about to create this with Anchor
    #[account(mut)]
    pub ticket_token_account: UncheckedAccount<'info>,
    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub ticket_metadata: UncheckedAccount<'info>,
    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub ticket_master_edition: UncheckedAccount<'info>,
    /// CHECK: We're about to create this with Metaplex
    pub collection_mint: UncheckedAccount<'info>,
    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,
    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    // native
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,
    // remaining accounts
    // token_account_info
    // transfer_authority_info
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, MintTicket<'info>>,
    _uri: String,
) -> Result<()> {
    let payer = &ctx.accounts.payer;
    let ticket_box = &ctx.accounts.ticket_box;
    let wallet = &ctx.accounts.wallet;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if ticket_box.end_at.is_some() {
        require!(
            ticket_box.end_at.unwrap() > current_timestamp,
            ErrorCode::EventEnded
        );
    }

    if ticket_box.num_of_tickets.is_some() {
        require!(
            ticket_box.num_of_sold < ticket_box.num_of_tickets.unwrap(),
            ErrorCode::SoldOut
        );
    }

    // transfer fee
    let price = ticket_box.price;
    if price > 0 {
        if let Some(mint) = ticket_box.currency {
            // TODO validate token account
            let token_account_info = &ctx.remaining_accounts[0];
            let transfer_authority_info = &ctx.remaining_accounts[1];

            let token_account = assert_is_ata(token_account_info, &payer.key(), &mint)?;
            if token_account.amount < price {
                return err!(ErrorCode::NotEnoughTokens);
            }
            spl_token_transfer(TokenTransferParams {
                source: token_account_info.clone(),
                destination: wallet.to_account_info(),
                amount: price,
                authority: transfer_authority_info.to_account_info(),
                authority_signer_seeds: &[],
                token_program: ctx.accounts.token_program.to_account_info(),
            })?;
        } else {
            if ctx.accounts.payer.lamports() < price {
                return err!(ErrorCode::NotEnoughSOL);
            }
            // transfer sol
            invoke(
                &system_instruction::transfer(&ctx.accounts.payer.key(), &wallet.key(), price),
                &[
                    ctx.accounts.payer.to_account_info(),
                    wallet.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }
    }

    mint_ticket(ctx)?;

    Ok(())
}

fn mint_ticket(ctx: Context<MintTicket>) -> Result<()> {
    // mint ticket
    // let creators = vec![mpl_token_metadata::state::Creator {
    //     address: ctx.accounts.payer.key(),
    //     verified: true,
    //     share: 100,
    // }];

    let bump = *ctx.bumps.get("ticket_box").unwrap();
    let ticket_box = &ctx.accounts.ticket_box;
    // let ticket_box_key = ticket_box.key();
    let signer_seeds = [
        b"ticket_box".as_ref(),
        ticket_box.uuid.as_ref(),
        ticket_box.creator.as_ref(),
        &[bump],
    ];

    msg!("Creating metadata account...");
    let mut nft_name = String::from(&ticket_box.name);
    nft_name.push_str(&(ticket_box.num_of_sold + 1).to_string());
    msg!("check nft name {}", nft_name);
    invoke_signed(
        &mpl_instruction::create_metadata_accounts_v3(
            MPL_TOKEN_METADATA_ID,
            ctx.accounts.ticket_metadata.key(),
            ctx.accounts.ticket_mint.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.payer.key(),
            ticket_box.key(),
            nft_name,
            String::from("TICKET"),
            ctx.accounts.ticket_box.uri.clone(),
            None,
            200,
            false,
            true,
            Some(Collection {
                verified: false,
                key: ctx.accounts.collection_mint.key(),
            }),
            None,
            None,
        ),
        &[
            ctx.accounts.ticket_metadata.to_account_info(),
            ctx.accounts.ticket_mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.ticket_box.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&signer_seeds],
    )?;

    msg!("Creating master edition metadata account...");
    invoke_signed(
        &mpl_instruction::create_master_edition_v3(
            MPL_TOKEN_METADATA_ID,
            ctx.accounts.ticket_master_edition.key(),
            ctx.accounts.ticket_mint.key(),
            ticket_box.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.ticket_metadata.key(),
            ctx.accounts.payer.key(),
            Some(0), // max_supply: Option<u64>
        ),
        &[
            ctx.accounts.ticket_master_edition.to_account_info(),
            ctx.accounts.ticket_mint.to_account_info(),
            ctx.accounts.ticket_box.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.ticket_metadata.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&signer_seeds],
    )?;

    msg!("Set and verify collection...");
    invoke_signed(
        &mpl_instruction::verify_sized_collection_item(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.ticket_metadata.key(),
            ctx.accounts.ticket_box.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.collection_mint.key(),
            ctx.accounts.collection_metadata.key(),
            ctx.accounts.collection_master_edition.key(),
            None,
        ),
        &[
            ctx.accounts.ticket_metadata.to_account_info(),
            ctx.accounts.ticket_box.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            ctx.accounts.collection_metadata.to_account_info(),
            ctx.accounts.collection_master_edition.to_account_info(),
        ],
        &[&signer_seeds],
    )?;

    msg!("Token mint process completed successfully.");

    Ok(())
}
