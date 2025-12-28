use anchor_lang::{
    prelude::*,
    solana_program::{program_error::ProgramError, system_program},
};

#[inline(always)]
pub fn read_token_mint_owner(ai: &AccountInfo) -> Result<(Pubkey, Pubkey)> {
    let data = ai.try_borrow_data()?;
    if data.len() < 64 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let mut mint_bytes = [0u8; 32];
    mint_bytes.copy_from_slice(&data[0..32]);

    let mut owner_bytes = [0u8; 32];
    owner_bytes.copy_from_slice(&data[32..64]);
    Ok((
        Pubkey::new_from_array(mint_bytes),
        Pubkey::new_from_array(owner_bytes),
    ))
}

#[inline(always)]
pub fn read_token_amount(ai: &AccountInfo) -> Result<u64> {
    let data = ai.try_borrow_data()?;
    if data.len() < 72 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[64..72]);
    Ok(u64::from_le_bytes(buf))
}

#[inline(always)]
pub fn close_account_to_destination(
    account: &AccountInfo,
    destination: &AccountInfo,
) -> Result<()> {
    let amount = account.lamports();
    {
        let mut dest_lamports_ref = destination.try_borrow_mut_lamports()?;
        let new_value = (**dest_lamports_ref)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        **dest_lamports_ref = new_value;
    }
    {
        let mut source_lamports_ref = account.try_borrow_mut_lamports()?;
        **source_lamports_ref = 0;
    }
    account.assign(&system_program::ID);
    let mut data = account.try_borrow_mut_data()?;
    for b in data.iter_mut() {
        *b = 0;
    }
    Ok(())
}
