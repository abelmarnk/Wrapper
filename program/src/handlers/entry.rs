use pinocchio::{
    account_info::{
        AccountInfo, 
        RefMut
    }, cpi::slice_invoke_signed, instruction::{
        AccountMeta, 
        Instruction, 
        Seed, 
        Signer
    }, log::sol_log_slice, msg, program_error::ProgramError, pubkey::{self, Pubkey}, ProgramResult
};

use crate::{
    config::{Config, CONFIG_MAX_ACCOUNTS}, 
    constants::{
        HASH_LENGTH
    }, 
    utils::{
        hashv, is_program_account, is_signer
    }
};

extern crate alloc;

use alloc::string::ToString;

use alloc::vec::Vec;

pub struct Entry<'a, 'b>{
    program_accounts:&'a[AccountInfo],
    commit_account:&'a AccountInfo,
    program_data:&'b[u8], 
    config_data:RefMut<'a, Config>
}

impl<'a, 'b> TryFrom<(&'a[AccountInfo], &'b[u8])> for Entry<'a, 'b> {
    #[inline(always)] // This function is only ever called once.
    fn try_from(value: (&'a[AccountInfo], &'b[u8])) -> Result<Self, Self::Error> {

        msg!("-0");

        // Extract accounts
        let [program_accounts@.., starter_account, 
            commit_account] = value.0 else{
                return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Extract data(no metadata is added to the instruction data)
        let program_data = value.1;

        msg!("-1");

        
        // Check if the account is owned by the program
        is_program_account(commit_account, Config::LEN, &crate::ID)?;

        msg!("-2");
        
        let mut data_ref = commit_account.try_borrow_mut_data()?;

        // Extract config data
        let config_data = 
        bytemuck::try_from_bytes_mut::<Config>(&mut data_ref). // Alignment = 1;
        map_err(|_| ProgramError::InvalidAccountData)?;

        msg!("-3");

        // Check if the starter signed and is as expected
        is_signer(starter_account)?;

        if starter_account.key().ne(&config_data.starter_key){
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("-4");
        
        // Check if instruction data matches the form committed to
        if !config_data.base.length_matches_commit_type(program_data.len()){
            return Err(ProgramError::InvalidInstructionData);
        }

        msg!("-5");
        
        // Check if the keys are sufficient
        if config_data.base.account_count[0] as usize > program_accounts.len() {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        msg!("-6");        

        // Check if the commit condition is valid, and update it if so
        config_data.base.update_condition()?;

        msg!("-7");

        // Should not panic since above conversion was successful
        let config_data_ref = RefMut::map(
            data_ref, |old_data_ref| bytemuck::from_bytes_mut::<Config>(old_data_ref));

        Ok(
            Entry{ 
                program_accounts, 
                commit_account, 
                config_data:config_data_ref, 
                program_data 
            }
        )
    }

    type Error = ProgramError;
}

impl<'a, 'b> Entry<'a, 'b>{

#[inline(always)] // This function would only ever be called once, it is separated for readability
fn get_commit_accounts(program_accounts:&'a[AccountInfo], config_data:&Config)-> Result<[&'a [u8];CONFIG_MAX_ACCOUNTS], ProgramError>{
    let mut commit_accounts:[&[u8];CONFIG_MAX_ACCOUNTS] = [&[];CONFIG_MAX_ACCOUNTS];

    
    for (index, commit_account) in config_data.base.account_indices.iter().
    take(usize::from(u8::from_le_bytes(config_data.base.account_count))).zip(commit_accounts.iter_mut()){
        let index = usize::from(*index);
        if index.ge(&program_accounts.len()) {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        *commit_account = program_accounts[index].key().as_ref();

        msg!(index.to_string().as_str());

        msg!("------");

        sol_log_slice(commit_account);
    }

    Ok(commit_accounts)
}

#[inline(always)] // This function is only ever called once and is separated for readabilty
fn get_account_metas(program_accounts:&[AccountInfo])->Vec<AccountMeta>{
        program_accounts.iter().
        take(program_accounts.len() - 1). // Exclude the program account
        map(|account|
        AccountMeta{
            is_signer:account.is_signer(),
            is_writable: account.is_writable(),
            pubkey: account.key()
        }
    ).collect()
}

#[inline(always)]
fn get_program_account(program_accounts:&[AccountInfo])->&AccountInfo{
    &program_accounts[program_accounts.len() - 1] // Program account is always added to the back
}

#[inline(always)] // This function is only ever called once.
pub fn process(&self)->ProgramResult{

    let commit_accounts = 
        Self::get_commit_accounts(self.program_accounts, &self.config_data)?; // Get the accounts that were committed to.
    
    let mut commit_accounts_hash:[u8;HASH_LENGTH] = [0;HASH_LENGTH];
    
    hashv(&commit_accounts[..usize::from(self.config_data.base.account_count[0])], 
    &mut commit_accounts_hash);

    msg!("-9");

    let commit_seeds =   // Use the necessary information
     self.config_data.base.get_commit_seeds(&commit_accounts_hash, Some(self.program_data))?; // to construct the commit key

     // Check commit PDA
    let expected_commit_account = pubkey::create_program_address(commit_seeds.as_ref(), 
        &crate::ID)
            .map_err(|_| ProgramError::InvalidSeeds)?;

    if !self.commit_account.key().eq(&expected_commit_account) { // The existence of the commit account
        return Err(ProgramError::InvalidSeeds); // as well as it's derivation guarantees it was created by
                                                // the expected user(s)
    }

    msg!("-10");

    // Build the instruction
    let instruction = Instruction {
         program_id: Self::get_program_account(self.program_accounts).key(), 
         data: self.program_data, 
         accounts:&Self::get_account_metas(self.program_accounts)
        };

    msg!("-11");

    // Build the accountinfos
    let accounts_ref:Vec<&AccountInfo> = self.program_accounts.iter().
        map(|account| account).collect();

    msg!("-12");

    // Build the signers
    let seeds:[[Seed;2];4] = core::array::from_fn
        (|index| [Seed::from(self.config_data.signer_keys[index].as_ref()), 
        Seed::from(&self.config_data.base.signer_bumps[index..(index + 1)])]);

    let signers:[Signer;4] = core::array::from_fn(|index|Signer::from(&seeds[index]));

    msg!("-13");

    // Invoke the main program with the provided instruction.
    slice_invoke_signed(&instruction,  accounts_ref.as_slice(), signers.as_ref())?;

    msg!("-14");
    
    Ok(())
}
}
