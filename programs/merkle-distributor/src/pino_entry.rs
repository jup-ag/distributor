#![allow(unused_imports)]

use super::*;
use anchor_lang::solana_program;

const NEW_CLAIM_MAX_ACCOUNTS: usize = 8;

#[inline(always)]
unsafe fn p_entrypoint(input: *mut u8) -> Option<u64> {
    let (program_id_ref, accounts, ix_data) = solana_program::entrypoint::deserialize(input);

    let program_id_pk = *program_id_ref;

    // Fast-path only `new_claim`
    if ix_data.len() >= 8 && ix_data.starts_with(crate::instruction::NewClaim::DISCRIMINATOR) {
        let res = crate::instructions::p_handle_new_claim(&program_id_pk, &accounts, ix_data);
        return Some(match res {
            Ok(()) => solana_program::entrypoint::SUCCESS,
            Err(e) => {
                // Map Anchor Error to u64 code
                let pe: anchor_lang::prelude::ProgramError = e.into();
                pe.into()
            }
        });
    }

    None
}

#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    if let Some(ret) = p_entrypoint(input) {
        ret
    } else {
        let (program_id_ref, accounts, ix_data) = solana_program::entrypoint::deserialize(input);
        let program_id_pk = *program_id_ref;
        match super::entry(&program_id_pk, &accounts, ix_data) {
            Ok(()) => solana_program::entrypoint::SUCCESS,
            Err(err) => err.into(),
        }
    }
}

solana_program::custom_heap_default!();
solana_program::custom_panic_default!();
