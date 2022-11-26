use {
    anchor_lang::{prelude::*, solana_program::program::invoke_signed},
    anchor_spl::token::{self},
    mpl_token_metadata::{
        instruction as mpl_instruction, state::CollectionDetails, utils::assert_owned_by,
        ID as MPL_TOKEN_METADATA_ID,
    },
};

use crate::TicketBox;
use crate::{assert_initialized, cmp_pubkeys, errors::ErrorCode};

#[derive(Accounts)]
#[instruction( uuid: String )]
pub struct Initialize<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        seeds = [b"ticket_box", uuid.as_bytes(), creator.key().as_ref()],
        bump,
        space = TicketBox::SIZE
    )]
    pub ticket_box: Account<'info, TicketBox>,

    /// CHECK: wallet can be any account and is not written to or read
    pub wallet: UncheckedAccount<'info>,

    //collection
    #[account(mut)]
    pub collection_mint: Signer<'info>,
    /// CHECK: We're about to create this with Anchor
    #[account(mut)]
    pub collection_token_account: UncheckedAccount<'info>,
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
    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,
}

pub fn handler(
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
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    require!(
        start_at >= current_timestamp,
        ErrorCode::InvalidTicketBoxStartAt
    );

    if end_at.is_some() {
        require!(end_at.unwrap() > start_at, ErrorCode::InvalidTicketBoxEndAt);
    }

    if num_of_tickets.is_some() {
        require!(
            ticket_per_wallet.is_some(),
            ErrorCode::InvalidTicketPerWallet
        );

        require!(
            num_of_tickets.unwrap() > ticket_per_wallet.unwrap(),
            ErrorCode::InvalidTicketPerWallet
        );
    }

    ctx.accounts.ticket_box.creator = ctx.accounts.creator.key();
    ctx.accounts.ticket_box.uuid = uuid.clone();
    ctx.accounts.ticket_box.name = name.clone();
    ctx.accounts.ticket_box.uri = uri.clone();
    ctx.accounts.ticket_box.start_at = start_at;
    ctx.accounts.ticket_box.end_at = end_at;
    ctx.accounts.ticket_box.num_of_tickets = num_of_tickets;
    ctx.accounts.ticket_box.num_of_sold = 0;
    ctx.accounts.ticket_box.ticket_per_wallet = ticket_per_wallet;
    ctx.accounts.ticket_box.price = price;
    ctx.accounts.ticket_box.transferable = transferable;
    ctx.accounts.ticket_box.escrow = ctx.accounts.wallet.key();

    if !ctx.remaining_accounts.is_empty() {
        let token_mint_info = &ctx.remaining_accounts[0];
        let _token_mint: token::spl_token::state::Mint = assert_initialized(token_mint_info)?;
        let token_account: token::spl_token::state::Account =
            assert_initialized(&ctx.accounts.wallet)?;
        assert_owned_by(token_mint_info, &token::spl_token::id())?;
        assert_owned_by(&ctx.accounts.wallet, &token::spl_token::id())?;
        if !cmp_pubkeys(&token_account.mint, token_mint_info.key) {
            return err!(ErrorCode::MintMismatch);
        }
        ctx.accounts.ticket_box.currency = Some(*token_mint_info.key);
    }

    // create nft collection
    // mint_collection(ctx, &uuid, &name, &uri)?;

    Ok(())
}

pub fn mint_collection(
    ctx: Context<Initialize>,
    uuid: &str,
    box_name: &str,
    box_uri: &String,
) -> Result<()> {
    let bump = *ctx.bumps.get("ticket_box").unwrap();
    let creator_key = ctx.accounts.creator.key();
    let signer_seeds = [
        b"ticket_box".as_ref(),
        uuid.as_bytes().as_ref(),
        creator_key.as_ref(),
        &[bump],
    ];

    msg!("Creating metadata account...");
    invoke_signed(
        &mpl_instruction::create_metadata_accounts_v3(
            MPL_TOKEN_METADATA_ID,
            ctx.accounts.collection_metadata.key(),
            ctx.accounts.collection_mint.key(),
            creator_key,
            creator_key,
            ctx.accounts.ticket_box.key(),
            box_name.to_string(),
            String::from("BOX"),
            box_uri.to_string(),
            None,
            200,
            false,
            true,
            None,
            None,
            Some(CollectionDetails::V1 { size: 0 }),
        ),
        &[
            ctx.accounts.collection_metadata.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            ctx.accounts.creator.to_account_info(),
            ctx.accounts.creator.to_account_info(),
            ctx.accounts.ticket_box.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        &[&signer_seeds],
    )?;

    msg!("Creating master edition metadata account...");
    invoke_signed(
        &mpl_instruction::create_master_edition_v3(
            MPL_TOKEN_METADATA_ID,
            ctx.accounts.collection_master_edition.key(),
            ctx.accounts.collection_mint.key(),
            ctx.accounts.ticket_box.key(),
            creator_key,
            ctx.accounts.collection_metadata.key(),
            creator_key,
            Some(0), // max_supply: Option<u64>
        ),
        &[
            ctx.accounts.collection_master_edition.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            ctx.accounts.ticket_box.to_account_info(),
            ctx.accounts.creator.to_account_info(),
            ctx.accounts.creator.to_account_info(),
            ctx.accounts.collection_metadata.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        &[&signer_seeds],
    )?;

    msg!("Collection mint process completed successfully.");

    Ok(())
}
