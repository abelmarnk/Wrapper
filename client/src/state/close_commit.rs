pub use solana_sdk::{
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
};

use crate::{constants, Base, CustomError, DataCommitType};

#[derive(Default)]
pub struct CloseCommit{
    pub base: Base,
    pub recipient_account:Pubkey
}

impl CloseCommit {
     pub fn new(
        program_id: Pubkey,
        signer_commit_account_keys: Vec<Pubkey>,
        non_signer_commit_account_keys: Vec<Pubkey>,
        recipient_account:Pubkey,
        total_accounts_count:u8,
        acct_indices: Vec<u8>,
        instruction_data_meta: DataCommitType
    ) -> Result<Self, CustomError> {

        let base = Base::new(
            program_id,
            signer_commit_account_keys,
            non_signer_commit_account_keys,
            None,
            total_accounts_count,
            acct_indices,
            instruction_data_meta
        )?;

        Ok(Self {
            base,
            recipient_account
        })
    }

    #[inline(always)]
    pub fn make_instruction(base: &mut Base, recipient_account:Pubkey) -> Result<Instruction, CustomError> {
        let mut accounts = Vec::with_capacity(
            base.signer_commit_account_keys.len() + 
            1 + // For the recipient account
            1 // For the commit account
        );

        let mut program_signers = Vec::with_capacity(
            base.signer_commit_account_keys.len()
        );

        // Add the signers
        for key in base.signer_commit_account_keys.iter(){            
            accounts.push(AccountMeta::new_readonly(*key, true));

            // Derive the program-signers(PDAs the signers control)
            let (key, _) = Pubkey::find_program_address(&[key.as_ref()], 
                &constants::PROGRAM_ID);
            program_signers.push(key);
        }

        // Add the recipient account
        accounts.push(AccountMeta::new(recipient_account, false));

        // Add the commit account
        let (commit_account_key, _) = base.
            get_commit_account(program_signers)?;

        accounts.push(AccountMeta::new(commit_account_key, false));

        
        Ok(Instruction {
            program_id: constants::PROGRAM_ID,
            accounts,
            data:vec![6],
        })
    }

    #[inline(always)]
    pub fn to_instruction(& mut self) -> Result<Instruction, CustomError> {
        CloseCommit::make_instruction(&mut self.base, self.recipient_account)
    }
}