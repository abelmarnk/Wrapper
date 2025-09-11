use pinocchio::{
    account_info::{
        AccountInfo    
    }, 
    program_error::ProgramError, 
    ProgramResult,
};

use bytemuck;

use crate::{
    config::Config,
    state::{
        error::CustomError
    }, 
    utils::{
        is_program_account, is_signer
    }
};

pub struct CloseCommit<'a>{
    pub commit_account:&'a AccountInfo,
    pub recipient_account:&'a AccountInfo
}

impl<'a, 'b> TryFrom<&'a [AccountInfo]> for CloseCommit<'a> {
    #[inline(always)] // This function is only called once
    fn try_from(value: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // Extract the accounts
        let [signers@.., recipient_account,
            commit_account] = value else{
                return Err(ProgramError::NotEnoughAccountKeys);
            };

        // Check if the account belongs to the program
        is_program_account(commit_account, Config::LEN, &crate::ID)?;

        // Extract the data
        let mut config_data_ref = commit_account.try_borrow_mut_data().
            map_err(|_| ProgramError::InvalidAccountData)?;

        let config_data = 
            bytemuck::try_from_bytes_mut::<Config>(&mut config_data_ref).
            map_err(|_| ProgramError::InvalidAccountData)?;

        // Check bounds constraints
        let signer_account_count = u8::from_le_bytes(config_data.base.signer_count);

        if !signers.len().eq(&usize::from(signer_account_count)){
            return Err(CustomError::InvalidAccountKeysCount.into());
        };

        // Check signers
        for (maybe_signer, expected_signer) in signers.iter().
            zip(config_data.signer_keys.iter()){ // The number of signers provided here should be
            is_signer(maybe_signer)?;    // <= MAX_SIGNERS, which is the no of public keys 
                                                  // there is space allocated for in 'config_data.signer_keys'
            if !maybe_signer.key().eq(expected_signer){
                return Err(ProgramError::MissingRequiredSignature);
            }
        }

        Ok(CloseCommit{
            recipient_account,
            commit_account
        })
    }

    type Error = ProgramError;
}

impl<'a> CloseCommit<'a>{
#[inline(always)]
pub fn process(&self) -> ProgramResult {

    *self.recipient_account.try_borrow_mut_lamports()? += self.commit_account.lamports();
    self.commit_account.close()   
}

}