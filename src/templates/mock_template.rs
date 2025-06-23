/*
    This will be generated as a static mock code used to generate the common contexts
*/

use anchor_lang::prelude::*;
use {crate_name}::{{AccountInfo, ID as PROGRAM_ID}};

pub fn mock_pubkey(label: &str) -> Pubkey {
    let mut bytes = [0u8; 32];
    let label_bytes = label.as_bytes();
    bytes[..label_bytes.len().min(32)].copy_from_slice(&label_bytes[..label_bytes.len().min(32)]);
    Pubkey::new_from_array(bytes)
}

/// Mocks a generic signer AccountInfo
pub fn mock_signer_account(label: &str) -> AccountInfo<'static> {
    let key = Box::leak(Box::new(mock_pubkey(label)));
    let lamports = Box::leak(Box::new(1_000_000u64));
    let data = Box::leak(Box::new([0u8; 10])); 

    // Box::leak() will take ownership of value, put it on the heap, and return a ref with `'static` lifetime of that value
    // that ref will live for entire duration of the program and will never be dropped(unless the program ends).

    AccountInfo::new(
        key,
        true,
        true,
        lamports,
        &mut data[..],
        key,
        false,
        0,
    )
}

/// Mocks a PDA AccountInfo with some dummy data
pub fn mock_pda_account(seeds: &[&[u8]], program_id: &Pubkey, size: usize) -> AccountInfo<'static> {
    let (pda, _bump) = Pubkey::find_program_address(seeds, program_id);
    let pda = Box::leak(Box::new(pda));

    let lamports = Box::leak(Box::new(1_000_000u64));

    // Anchor expects the first 8 bytes of the account data to be a unique discriminator for the account type
    let mut data = [0u8; 64];
    let discriminator = <ShipmentIdCounter as anchor_lang::Discriminator>::DISCRIMINATOR;
    data[..8].copy_from_slice(&discriminator);
    let data = Box::leak(Box::new(data)); 

    let owner = Box::leak(Box::new(*program_id));

    AccountInfo::new(
        pda,
        false,
        true,
        lamports,
        &mut data[..],
        owner,
        false,
        0,
    )
}

/// Returns a dummy system program AccountInfo
pub fn mock_system_program() -> AccountInfo<'static> {
    let key = Box::leak(Box::new(anchor_lang::system_program::ID));
    let lamports = Box::leak(Box::new(1_000_000u64));
    let data = Box::leak(Box::new([0u8; 10]));
    let owner = Box::leak(Box::new(PROGRAM_ID));

    AccountInfo::new(
        key,
        false,
        false,
        lamports,
        &mut data[..],
        owner,
        true, // should be executable because it is a program
        0,
    )
}