extern crate alloc;

use alloc::string::ToString;

use pinocchio::{
    account_info::AccountInfo, memory::sol_memcmp, msg, program_error::ProgramError, pubkey::{self, Pubkey}, syscalls, ProgramResult
};

use crate::CustomError;

// To be called after the accounts bound has been checked.
pub fn extract_program_signers<'a,'b>(accounts:&'a[AccountInfo], signers_count: u8, accounts_count: u8) -> &'a[AccountInfo]{
    &accounts[((accounts_count - signers_count) as usize)..(accounts_count as usize)]
}

// To be called after the accounts bound has been checked.
pub fn extract_signers<'a,'b>(accounts:&'a[AccountInfo], signers_count: u8, accounts_count: u8) -> &'a[AccountInfo]{
    &accounts[(accounts_count as usize)..((accounts_count + signers_count) as usize)]
}

/// Verify signer PDAs and their signatures, all slices are expected to be of the same length
pub fn verify_signers(
    signers: &[AccountInfo],
    program_signers: &[AccountInfo],
    signer_bumps: &[u8],
) -> ProgramResult {

    msg!("-3.5");

    if program_signers.len() > signers.len(){
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("-3.75");

    for ((signer, program_signer), bump) in 
        signers.iter().zip(program_signers.iter()).zip(signer_bumps.iter()) {

        msg!("--");
            
        if !signer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let expected_program_signer =
            pubkey::create_program_address(&[signer.key().as_ref(), &[*bump]], &crate::ID)
                .map_err(|_| CustomError::InvalidSignerSeeds)?;

        if !are_keys_equal(&expected_program_signer, program_signer.key()) {
            return Err(ProgramError::from(CustomError::InvalidSignerSeeds));
        }
    }
    Ok(())
}

// Accounts should have been checked to be up to size before call
pub fn extract_commit_account<'a>(accounts:&'a[AccountInfo])-> &'a AccountInfo{
    &accounts[accounts.len() - 1]
}

pub fn extract_payer_account<'a>(accounts:&'a[AccountInfo])-> &'a AccountInfo{
        &accounts[accounts.len() - 3]
}

pub fn extract_program_account<'a>(accounts:&'a[AccountInfo])-> &'a AccountInfo{
        &accounts[accounts.len() - 2]
}

pub fn extract_amount(instruction_data: &[u8])->Result<u64, ProgramError>{
    instruction_data
        .get(..8)
        .and_then(|bytes| Some(u64::from_le_bytes(bytes.try_into().ok()?)))
        .ok_or(ProgramError::InvalidInstructionData)
}

pub fn extract_bump(instruction_data: &[u8])->Result<u8, ProgramError>{
    instruction_data
        .get(8)
        .and_then(|byte| Some(*byte))
        .ok_or(ProgramError::InvalidInstructionData)
}

pub fn extract_decimals(instruction_data: &[u8])->Result<u8, ProgramError>{
    instruction_data
        .get(9)
        .and_then(|byte| Some(*byte))
        .ok_or(ProgramError::InvalidInstructionData)
}

pub fn are_keys_equal(first:&Pubkey, second:&Pubkey)->bool{
    unsafe{
        sol_memcmp(first.as_ref(), second.as_ref(), size_of::<Pubkey>()) == 0
    }
}

#[inline(always)]
pub fn is_program_account(account:&AccountInfo, data_len:usize, owner:&Pubkey)->Result<(), ProgramError>{

    if account.lamports().eq(&0){
        return Err(ProgramError::UninitializedAccount);
    }

    msg!("Data length:- ");
    msg!(account.data_len().to_string().as_str());

    if account.data_len().ne(&data_len){
        return Err(ProgramError::InvalidAccountData);
    }

    if account.owner().ne(owner){
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
    
}

#[inline(always)]
pub fn is_signer(account:&AccountInfo)-> Result<(), ProgramError>{
    if !account.is_signer(){
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(())
}

#[inline(always)]
pub fn is_unitialized(account:&AccountInfo)->Result<(), ProgramError>{
    if account.data_is_empty() && account.lamports().eq(&0) && 
        account.is_owned_by(&pinocchio_system::ID){
        return Err(ProgramError::UninitializedAccount);
    }

    Ok(())
}

#[inline(always)]
pub fn hashv(values:&[&[u8]], hash_result:&mut [u8; 32]){
        unsafe {
            syscalls::sol_sha256(
                values as *const _ as *const u8,
                values.len() as u64,
                hash_result as *mut _ as *mut u8,
            );
        }
}