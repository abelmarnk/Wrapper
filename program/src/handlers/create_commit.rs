use pinocchio::{
    account_info::AccountInfo, instruction::{
        Seed, 
        Signer
    }, log::{sol_log, sol_log_64, sol_log_slice}, msg, program_error::ProgramError, pubkey, sysvars::{
        rent::Rent, 
        Sysvar
    }, ProgramResult
};

use bytemuck;

use crate::{
    config::{
        Config, ConfigBase, CONFIG_MAX_ACCOUNTS, CONFIG_MAX_SIGNERS, CONFIG_MIN_ACCOUNTS
    }, 
    constants::HASH_LENGTH, 
    state::{
        condition::CommitCondition,
        error::CustomError
    }, utils::{
        hashv, 
        is_signer, 
        verify_signers
    }
};


pub struct CreateCommit<'a, 'b>{
    pub commit_accounts:&'a[AccountInfo],
    pub signers:&'a[AccountInfo],
    pub starter_account:&'a AccountInfo,
    pub commit_account:&'a AccountInfo,
    pub config_base_data:&'b ConfigBase,
}

impl<'a, 'b> TryFrom<(&'a [AccountInfo], &'b[u8])> for CreateCommit<'a, 'b> {
    fn try_from(value: (&'a [AccountInfo], &'b[u8])) -> Result<Self, Self::Error> {

        msg!("--1");

        // Extract accounts/data
        let [other_accounts@.., 
            starter_account, _, commit_account] = value.0 else{
                return Err(ProgramError::NotEnoughAccountKeys);
            };
        let data = value.1;

        sol_log_slice(data.len().to_le_bytes().as_ref());

        let config_base_data = 
            bytemuck::try_from_bytes::<ConfigBase>(&data).
                map_err(|_|ProgramError::InvalidInstructionData)?;

        // Check bounds constraints
        let commit_account_count = u8::from_le_bytes(config_base_data.account_count);

        let signer_account_count = u8::from_le_bytes(config_base_data.signer_count);

        msg!("-0");

        if usize::from(commit_account_count).lt(&CONFIG_MIN_ACCOUNTS) || 
            usize::from(commit_account_count).gt(&CONFIG_MAX_ACCOUNTS) {
            return Err(ProgramError::from(CustomError::InvalidAccountCount));
        }

        msg!("-1");

        if signer_account_count.eq(&0) || usize::from(signer_account_count).gt(&CONFIG_MAX_SIGNERS) || 
            signer_account_count.ge(&commit_account_count){
            return Err(ProgramError::from(CustomError::InvalidSignerCount));
        }

        msg!("-1.5");

        sol_log_slice(commit_account_count.to_le_bytes().as_ref());

        sol_log_slice(signer_account_count.to_le_bytes().as_ref());

        sol_log_slice(other_accounts.len().to_le_bytes().as_ref());


        if other_accounts.len().ne(&usize::from(
            commit_account_count.checked_add(signer_account_count).
                ok_or_else(|| ProgramError::ArithmeticOverflow)?)){
            return Err(CustomError::InvalidAccountKeysCount.into());
        };

        msg!("-2");

        // Check if the starter account signed, they would be paying for the transaction
        is_signer(starter_account)?; 

        // Check signers
        // Checks if pda provided had their corresponding account sign for it 
        let commit_accounts = &other_accounts[..usize::from(commit_account_count)];

        let signers = &other_accounts[usize::from(commit_account_count)..];

        // no of program-signers(PDAs the signers control) = no of signers
        let program_signers = 
            &commit_accounts[usize::from(commit_account_count - signer_account_count)..];

        msg!("-3");

        verify_signers(signers, program_signers, &config_base_data.signer_bumps)?;

        // Check if the commit condition is valid
        config_base_data.condition.is_valid()
            .then_some(())
            .ok_or(ProgramError::from(CustomError::InvalidCommitConditionTag))?;

        msg!("-4");

        Ok(CreateCommit{
            commit_accounts,
            signers,
            starter_account,
            commit_account,
            config_base_data
        })
    }

    type Error = ProgramError;
}
impl<'a, 'b> CreateCommit<'a, 'b>{

pub fn process(&self) -> ProgramResult {

    // Get the commit accounts keys
    let mut commit_accounts:[&[u8];CONFIG_MAX_ACCOUNTS] = [&[];CONFIG_MAX_ACCOUNTS];
    
    for (account, commit_account) in self.commit_accounts.iter().
        zip(commit_accounts.iter_mut()){
            *commit_account = account.key().as_ref();
        }

    msg!("-5");
    
    // Take the hash
    let mut commit_accounts_hash:[u8;HASH_LENGTH] = [0;HASH_LENGTH];
    
    hashv(&commit_accounts[..usize::from(self.config_base_data.account_count[0])], 
    &mut commit_accounts_hash);

    let mut commit_seeds = self.config_base_data.
        get_commit_seeds(commit_accounts_hash.as_ref(), None)?;

    msg!("-6");

    // Check commit PDA
    let (expected_commit_account, expected_commit_bump) = 
        pubkey::find_program_address(&commit_seeds[..5], // Skip the provided bump
            &crate::ID);

    if self.commit_account.key().ne(&expected_commit_account) { // The existence of the commit account
        msg!("-6.5");
        return Err(ProgramError::InvalidSeeds); // as well as it's derivation guarantees it was created by
                                                // the expected user(s)
    }

    let new_bump_ref = &[expected_commit_bump];
    commit_seeds[5] = new_bump_ref;

    msg!("-7");

    self.create_account(&commit_seeds)?;

    msg!("-8");

    self.write_commit_account_data()?;

    Ok(())
}

#[inline(always)] // This function is only ever called once
fn create_account(&self, commit_seeds:&[&[u8]])->Result<(), ProgramError>{
    let rent = Rent::get()?;

    let required_lamports = rent.minimum_balance(Config::LEN);

    let commit_seeds:[Seed;6] = core::array::from_fn(
            |index| Seed::from(commit_seeds[index]));

    let signer = Signer::from(commit_seeds.as_ref());
    
    pinocchio_system::instructions::CreateAccount{
        from:self.starter_account,
        to:self.commit_account,
        lamports:required_lamports,
        space:Config::LEN as u64,
        owner: &crate::ID
    }.invoke_signed(&[signer])?;
        
    Ok(())    
}

#[inline(always)] // This function is only ever called once
fn write_commit_account_data(&self)->Result<(), ProgramError>{

    // Extract the config data
    let config_data = unsafe{
        bytemuck::try_from_bytes_mut::<Config>(self.commit_account.borrow_mut_data_unchecked()).
            map_err(|_| ProgramError::InvalidAccountData)?
    };

    // Set the base info
    config_data.base = *self.config_base_data;

    // Set the key that would be able to initiate the transaction
    config_data.starter_key = *self.starter_account.key();

    // Set all the signers, they would be used for making changes to the commit
    // and they are also used to derive the PDAs
    for (signer_key, config_signer_key) in self.signers.iter().
        map(|account|account.key()).zip(config_data.signer_keys.iter_mut()){
        *config_signer_key = *signer_key;
    }

    Ok(())
}
}