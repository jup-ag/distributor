#![allow(unused_imports)]

use super::*;
use anchor_lang::solana_program;

#[inline(always)]
unsafe fn p_entrypoint(input: *mut u8) -> Option<u64> {
    let (program_id_ref, accounts, ix_data) = solana_program::entrypoint::deserialize(input);

    let program_id_pk = *program_id_ref;

    if ix_data.len() >= 8 {
        let new_dist_disc = crate::instruction::NewDistributor::DISCRIMINATOR;
        if ix_data.starts_with(crate::instruction::NewClaim::DISCRIMINATOR) {
            let res = crate::instructions::p_handle_new_claim(&program_id_pk, &accounts, ix_data);
            return Some(match res {
                Ok(()) => solana_program::entrypoint::SUCCESS,
                Err(e) => {
                    let pe: anchor_lang::prelude::ProgramError = e.into();
                    pe.into()
                }
            });
        }

        if ix_data.starts_with(&new_dist_disc) {
            let res =
                crate::instructions::p_handle_new_distributor(&program_id_pk, &accounts, ix_data);
            return Some(match res {
                Ok(()) => solana_program::entrypoint::SUCCESS,
                Err(e) => {
                    let pe: anchor_lang::prelude::ProgramError = e.into();
                    pe.into()
                }
            });
        }
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
