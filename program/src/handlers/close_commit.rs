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
        error::WrapperError
    }, 
    utils::{
        is_program_account, is_signer
    }
};

/// Stores state for the close commit instruction
pub struct CloseCommit<'a>{
    pub commit_account:&'a AccountInfo,
    pub recipient_account:&'a AccountInfo
}

impl<'a, 'b> TryFrom<&'a [AccountInfo]> for CloseCommit<'a> {


    /// Extracts the accounts and check the signers
    /// [Signers]:- These are the signers that are bound to the commit account, 
    /// in that their corresponding PDAs sign for the transaction confirming to that
    /// commit form
    /// 
    /// Recipient account:- This is the account that would be recieving the rent of the commit account
    /// 
    /// Commit account:- This is the account storing the commit configuration
    /// 
    fn try_from(value: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // Extract the accounts & check signers

        // [Signers] -- Recipient account -- Commit account
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

        if signers.len().ne(&usize::from(signer_account_count)){
            return Err(WrapperError::InvalidAccountKeysCount.into());
        };

        // Check signers
        for (maybe_signer, expected_signer) in signers.iter().
            zip(config_data.signer_keys.iter()){ // The number of signers provided here should be
            is_signer(maybe_signer)?;    // <= MAX_SIGNERS, which is the no of public keys 
                                                  // there is space allocated for in 'config_data.signer_keys',
                                                  // this constraint is checked above
            if maybe_signer.key().ne(expected_signer){
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
/// This closes a commit account storing all the necessary information to 
/// call this program(the Entry instruction) with a given commit configuration
pub fn process(&self) -> ProgramResult {

    *self.recipient_account.try_borrow_mut_lamports()? += self.commit_account.lamports();

    // Sets the data, owner and lamports to zero
    self.commit_account.close()   
}

}