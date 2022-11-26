use anchor_lang::{
    prelude::*,
    solana_program::{program_pack::{IsInitialized, Pack}, program_memory::sol_memcmp, pubkey::PUBKEY_BYTES, program::invoke_signed},
};

use anchor_spl::{token, associated_token::get_associated_token_address};
use mpl_token_metadata::utils::assert_owned_by;

use crate::errors::ErrorCode;

pub fn assert_initialized<T: Pack + IsInitialized>(account_info: &AccountInfo) -> Result<T> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(ErrorCode::Uninitialized.into())
    } else {
        Ok(account)
    }
}

pub fn assert_is_ata(
    ata: &AccountInfo,
    wallet: &Pubkey,
    mint: &Pubkey,
) -> core::result::Result<token::spl_token::state::Account, ProgramError> {
    assert_owned_by(ata, &token::spl_token::id())?;
    assert_owned_by(ata, &token::ID)?;
    let ata_account: token::spl_token::state::Account = assert_initialized(ata)?;
    assert_keys_equal(&ata_account.owner, wallet)?;
    assert_keys_equal(&ata_account.mint, mint)?;
    assert_keys_equal(&get_associated_token_address(wallet, mint), ata.key)?;
    Ok(ata_account)
}

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

pub fn assert_keys_equal(key1: &Pubkey, key2: &Pubkey) -> Result<()> {
    if !cmp_pubkeys(key1, key2) {
        err!(ErrorCode::PublicKeyMismatch)
    } else {
        Ok(())
    }
}

pub struct TokenTransferParams<'a: 'b, 'b> {
    /// CHECK: account checked in CPI
    pub source: AccountInfo<'a>,
    /// CHECK: account checked in CPI
    pub destination: AccountInfo<'a>,
    pub amount: u64,
    /// CHECK: account checked in CPI
    pub authority: AccountInfo<'a>,
    pub authority_signer_seeds: &'b [&'b [u8]],
    /// CHECK: account checked in CPI
    pub token_program: AccountInfo<'a>
}

pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> Result<()> {
    let TokenTransferParams{
        source,
        destination,
        authority,
        authority_signer_seeds,
        amount,
        token_program
    }   = params;

    let mut signer_seeds = vec![];
    if !authority_signer_seeds.is_empty() {
        signer_seeds.push(authority_signer_seeds);
    }

    let result = invoke_signed(
        &token::spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount
        )?,
        &[
            source, destination, authority, token_program
        ],
        &signer_seeds
    );

    result.map_err(|_| ErrorCode::TokenTransferFailed.into())
}