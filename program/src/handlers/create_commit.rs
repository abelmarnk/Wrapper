use pinocchio::{
    account_info::AccountInfo, instruction::{
        Seed, 
        Signer
    }, log::{sol_log_slice}, msg, program_error::ProgramError, pubkey, sysvars::{
        rent::Rent, 
        Sysvar
    }, ProgramResult
};

use bytemuck;

use crate::{
    config::{
        Config, ConfigBase
    }, 
    constants::{
        CONFIG_MAX_ACCOUNTS, COMMIT_SEEDS_LEN, CONFIG_MAX_SIGNERS, 
        CONFIG_MIN_ACCOUNTS, HASH_LENGTH
    }, 
    state::{
        error::WrapperError
    }, utils::{
        hashv, 
        is_signer, 
        verify_signers
    }
};

/// Stores state for the create commit instruction
pub struct CreateCommit<'a, 'b>{
    pub commit_accounts:&'a[AccountInfo],
    pub signers:&'a[AccountInfo],
    pub starter_account:&'a AccountInfo,
    pub commit_account:&'a AccountInfo,
    pub config_base_data:&'b ConfigBase,
}

impl<'a, 'b> TryFrom<(&'a [AccountInfo], &'b[u8])> for CreateCommit<'a, 'b> {
    /// Extract the commit data and accounts, checking the bounds contraints and signers
    /// It expects the instruction data to contain the commit data only and the accounts to be
    /// in the following order:- [Accounts to commit to -- Signers] -- Starter account 
    /// -- System program -- Commit account
    /// [Accounts to commit]:-
    /// It's divided in two sets of accounts:-
    /// [Non program signers]:- This includes the program account(account of the program being commited to)
    /// and other non signer accounts, they come in first
    /// 
    /// [Program signers]:- These are accounts that would be signing in place of the actual signers, they
    /// are derived from the signer's address and would control their funds.
    /// 
    /// [Signers]:-These are accounts that would normally be signing but have given out their funds to the program
    /// accounts to handle, there is one program account for each signer
    /// 
    /// Starter account:- This is the account that would be required to call a transaction conforming to a 
    /// particular commit form, it would also be paying for the commit account creation here
    /// 
    /// System program:- This is requied to create accounts
    /// 
    /// Commit account:- This is the account that stores relevant data on the instruction form commited to.
    /// 
    fn try_from(value: (&'a [AccountInfo], &'b[u8])) -> Result<Self, Self::Error> {

        msg!("--1");

        // Extract accounts/data

        // [Accounts to commit to -- Signers] -- Starter account -- System program --
        // Commit account
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
            return Err(ProgramError::from(WrapperError::InvalidAccountCount));
        }

        msg!("-1");

        if signer_account_count.eq(&0) || usize::from(signer_account_count).gt(&CONFIG_MAX_SIGNERS) || 
            signer_account_count.ge(&commit_account_count){
            return Err(ProgramError::from(WrapperError::InvalidSignerCount));
        }

        msg!("-1.5");

        sol_log_slice(commit_account_count.to_le_bytes().as_ref());

        sol_log_slice(signer_account_count.to_le_bytes().as_ref());

        sol_log_slice(other_accounts.len().to_le_bytes().as_ref());

        // Check that the accounts to commit to are of the same count
        let expected_account_count = usize::from(
            commit_account_count.checked_add(signer_account_count).
                ok_or(ProgramError::ArithmeticOverflow)?);

        if other_accounts.len().ne(&expected_account_count){
            return Err(WrapperError::InvalidAccountKeysCount.into());
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
        config_base_data.condition.is_valid()?;

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

/// Process the create commit instruction
pub fn process(&self) -> ProgramResult {

    // Get the commit accounts keys
    let mut commit_accounts:[&[u8];CONFIG_MAX_ACCOUNTS] = [&[];CONFIG_MAX_ACCOUNTS];
    
    for (account, commit_account) in 
        self.commit_accounts.iter().zip(commit_accounts.iter_mut()){
            *commit_account = account.key().as_ref();
        }

    msg!("-5");
    
    // Take the hash of the accounts to commit to.
    let mut commit_accounts_hash:[u8;HASH_LENGTH] = [0;HASH_LENGTH];
    
    hashv(&commit_accounts[..usize::from(self.config_base_data.account_count[0])], 
    &mut commit_accounts_hash);

    // Get the seeds used to create the commit account
    let mut commit_seeds = self.config_base_data.
        get_commit_seeds(commit_accounts_hash.as_ref(), None)?;

    msg!("-6");

    // Check commit PDA
    let (expected_commit_account, expected_commit_bump) = 
        pubkey::find_program_address(&commit_seeds[..(commit_seeds.len() - 1)], // Skip the provided bump
            &crate::ID);

    if self.commit_account.key().ne(&expected_commit_account) { 
        msg!("-6.5");
        return Err(ProgramError::InvalidSeeds); 
                                                
    }

    // Set the canonical bump
    let expected_commit_bump_ = [expected_commit_bump];

    commit_seeds[commit_seeds.len() - 1] = &expected_commit_bump_;

    msg!("-7");

    // Setup the create commit account
    self.create_account(&commit_seeds)?;

    msg!("-8");

    self.write_commit_account_data()?;

    Ok(())
}

// This function is only ever called once and is seperated for readability
/// Create the commit account
#[inline(always)] 
fn create_account(&self, commit_seeds:&[&[u8];COMMIT_SEEDS_LEN])->Result<(), ProgramError>{
    let rent = Rent::get()?;

    let required_lamports = rent.minimum_balance(Config::LEN);

    let commit_seeds:[Seed;COMMIT_SEEDS_LEN] = core::array::from_fn(
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

// This function is only ever called once and is seperated for readability
/// Write the data of the commit account
#[inline(always)] 
fn write_commit_account_data(&self)->Result<(), ProgramError>{

    // Extract the config data
    let config_data = unsafe{
        // SAFETY: No account data besides this one is borrowed during the call of this instruction
        // as an additional guarantee the commit account is guaranteed to be unique due to the nature
        // of it's construction
        let commit_account_data = self.commit_account.borrow_mut_data_unchecked();

        bytemuck::try_from_bytes_mut::<Config>(commit_account_data).map_err(|_| ProgramError::InvalidAccountData)?
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