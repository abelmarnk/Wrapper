use solana_sdk::{
    instruction::{AccountMeta, Instruction}, pubkey::Pubkey
};

use crate::{constants, Base, CustomError, DataCommitType};

pub struct Entry{
    pub base: Base,
}
impl Entry{
     pub fn new(
        program_id: Pubkey,
        signer_commit_account_keys: Vec<Pubkey>,
        non_signer_commit_account_keys: Vec<Pubkey>,
        starter_account_key: Pubkey,
        total_accounts_count:u8,
        acct_indices: Vec<u8>,
        instruction_data_meta: DataCommitType
    ) -> Result<Self, CustomError> {

        let base = Base::new(
            program_id,
            signer_commit_account_keys,
            non_signer_commit_account_keys,
            Some(starter_account_key),
            total_accounts_count,
            acct_indices,
            instruction_data_meta
        )?;

        Ok(Self {
            base
        })
    }

    pub fn to_instruction(&mut self, mut old_instruction:Instruction) -> Result<Instruction, CustomError> {        
        
        let mut program_signers = Vec::with_capacity(
            self.base.signer_commit_account_keys.len()
        );

        // Add the signers
        for (key, index) in self.base.signer_commit_account_keys.iter().
            zip(self.base.acct_indices.iter().skip(self.base.non_signer_commit_account_keys.len())){            
            // Derive the program-signers(PDAs the signers control)
            let (key, _) = Pubkey::find_program_address(&[key.as_ref()], 
                &constants::PROGRAM_ID);
            old_instruction.accounts.get_mut(usize::from(*index)).
                ok_or(CustomError::InvalidAccountIndex)?.pubkey = key;
            program_signers.push(key);
        }
        
        let old_program_id = old_instruction.program_id;
        
        old_instruction.program_id = constants::PROGRAM_ID;
        
        old_instruction.accounts.push(AccountMeta {
            pubkey: old_program_id,
            is_signer: false,
            is_writable: false,
        });

        old_instruction.accounts.push(AccountMeta {
            pubkey: self.base.starter_account_key.
                ok_or_else(|| CustomError::StarterKeyNotProvided)?,
            is_signer: true,
            is_writable: false,
        });

        let (commit_account, _) = 
            self.base.get_commit_account(program_signers)?;

        old_instruction.accounts.push(AccountMeta {
            pubkey: commit_account,
            is_signer: true,
            is_writable: true,
        });

        old_instruction.data.push(0); // Entry instruction marker|Replace with import

        Ok(old_instruction)
    }}


