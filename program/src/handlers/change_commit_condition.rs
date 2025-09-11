use pinocchio::{
    account_info::{
        AccountInfo, 
        RefMut
    }, log::sol_log_slice, msg, program_error::ProgramError, ProgramResult
};

use bytemuck;

use crate::{
    config::Config,
    state::{
        condition::CommitCondition, 
        error::CustomError
    }, 
    utils::{
        is_program_account, is_signer
    }
};

pub struct ChangeCommit<'a, 'b>{
    pub signers:&'a[AccountInfo],
    pub commit_account:&'a AccountInfo,
    pub config_data:RefMut<'a, Config>,
    pub new_condition:&'b CommitCondition
}

impl<'a, 'b> TryFrom<(&'a [AccountInfo], &'b[u8])> for ChangeCommit<'a, 'b> {
    fn try_from(value: (&'a [AccountInfo], &'b[u8])) -> Result<Self, Self::Error> {

        // Extract accounts/data
        let [signers@.., commit_account] = value.0 else{
                return Err(ProgramError::NotEnoughAccountKeys);
        };

        let data = value.1;

        // Check if the account belongs to the program
        is_program_account(commit_account, Config::LEN, &crate::ID)?;

        // Extract data
        let new_condition = 
            bytemuck::try_from_bytes::<CommitCondition>(&data). // Alignment is 1
                map_err(|_|ProgramError::InvalidInstructionData)?;

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
            zip(config_data.signer_keys.iter()){
            is_signer(maybe_signer)?;

            msg!("Maybe:- ");
            sol_log_slice(maybe_signer.key());

            msg!("Expected:- ");
            sol_log_slice(expected_signer);

            if !maybe_signer.key().eq(expected_signer){
                return Err(ProgramError::MissingRequiredSignature);
            }
        }

        // Check if the commit condition is valid
        new_condition.is_valid()
            .then_some(())
            .ok_or(ProgramError::from(CustomError::InvalidCommitConditionTag))?;

        let config_data = RefMut::map(
            config_data_ref, |config_data| bytemuck::from_bytes_mut(config_data));

        Ok(ChangeCommit{
            signers,
            commit_account,
            config_data,
            new_condition
        })
    }

    type Error = ProgramError;
}

impl<'a, 'b> ChangeCommit<'a, 'b>{

#[inline(always)]
pub fn process(&mut self) -> ProgramResult {

    self.config_data.base.condition = *self.new_condition;

    Ok(())
}
}