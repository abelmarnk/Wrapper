use solana_sdk::{hash, instruction::AccountMeta, pubkey::Pubkey, system_program};
use crate::{constants, CustomError, CONFIG_MAX_ACCOUNTS, CONFIG_MAX_SIGNERS, CONFIG_MIN_ACCOUNTS};


#[derive(Default)]
pub enum DataCommitType{
    #[default]
    NoData,
    AnyData,
    Data(i16, Vec<u8>)
}

#[derive(Default)]
pub struct Base{
    pub program_id: Pubkey,
    pub non_signer_commit_account_keys: Vec<Pubkey>,
    pub signer_commit_account_keys: Vec<Pubkey>,
    pub starter_account_key: Option<Pubkey>,
    pub total_accounts_count: u8,
    pub acct_indices: Vec<u8>,
    pub instruction_data_meta:DataCommitType,
    // Intermidiate state
    pub signer_bumps: Vec<u8>,
    pub instruction_data_hash: [u8;32],
    pub commit_account_bump: u8,
}

impl Base {
    pub fn new(
        program_id: Pubkey,
        signer_commit_account_keys: Vec<Pubkey>,
        non_signer_commit_account_keys: Vec<Pubkey>,
        starter_account_key: Option<Pubkey>,
        total_accounts_count:u8,
        acct_indices: Vec<u8>,
        instruction_data_meta: DataCommitType
    ) -> Result<Self, CustomError> {

        let signers_count = signer_commit_account_keys.len();
        let accounts_count = signers_count + 
            non_signer_commit_account_keys.len() + 
            1; // For the program account
        
        if signers_count.eq(&0) || signers_count.gt(&CONFIG_MAX_SIGNERS){
            return Err(CustomError::InvalidSignerCount);
        }

        if accounts_count.lt(&CONFIG_MIN_ACCOUNTS) || accounts_count.gt(&CONFIG_MAX_ACCOUNTS){
            return Err(CustomError::InvalidAccountCount);
        }

        if !acct_indices.len().eq(&usize::from(accounts_count - 1)){
            return Err(CustomError::InvalidAccountKeysCount);
        }

        if acct_indices.iter().any(|&index| index.ge(&total_accounts_count)){
            return Err(CustomError::InvalidAccountIndex);
        }

        if (accounts_count - 1).gt(&usize::from(total_accounts_count)){
            return Err(CustomError::InvalidAccountCount);
        }


        Ok(Self {
            program_id,
            signer_commit_account_keys,
            non_signer_commit_account_keys,
            starter_account_key,
            total_accounts_count,
            acct_indices,
            instruction_data_meta,
            ..Default::default()
        })
    }

    pub fn get_accounts_and_mid_state(&mut self, payer_writing:bool, payer_signing:bool)-> 
        Result<Vec<AccountMeta>, CustomError>{
        let signers_count = self.signer_commit_account_keys.len();
        let accounts_count = signers_count + 
            self.non_signer_commit_account_keys.len() + 1; // For the program account

        if accounts_count > 8{
            return Err(CustomError::InvalidAccountCount); // Invalid accounts count
        }

        if signers_count == 0{
            return Err(CustomError::InvalidSignerCount); // No signers provided
        }

        if signers_count > 4{
            return Err(CustomError::InvalidSignerCount); // Too many signers provided
        }

        let mut accounts:Vec<AccountMeta> = Vec::<AccountMeta>::with_capacity(
            accounts_count +
            signers_count + // For the signers which would be used to verify the pda
            1 + // The commit account
            1 // The starter account
        );

        // Order:-
        // Non-signer commit accounts
        // Program account
        // Program-signers(PDAs the signers control)
        // Signers
        // Starter account
        // Commit account

        // Non-signer commit accounts
        for key in self.non_signer_commit_account_keys.iter() {
            accounts.push(AccountMeta::new_readonly(*key, false));
        }

        // Program account
        accounts.push(AccountMeta {
            pubkey: self.program_id,
            is_signer: false,
            is_writable: false,
        });

        self.signer_bumps = Vec::with_capacity(signers_count);

        let mut program_signer_keys:Vec<Pubkey> = Vec::with_capacity(signers_count);

        for key in self.signer_commit_account_keys.iter(){
            let (key, bump) = Pubkey::find_program_address(&[key.as_ref()], 
            &constants::PROGRAM_ID);

            self.signer_bumps.push(bump); // Bump for each signer
            program_signer_keys.push(key); // Program-signers for the pda

            // Program-signers(PDAs the signers control)
            accounts.push(AccountMeta::new_readonly(key, false));
        }
        
        // Signers
        for key in self.signer_commit_account_keys.iter(){            
            accounts.push(AccountMeta::new_readonly(*key, true));
        }

        // Starter account
        accounts.push(AccountMeta { 
            pubkey: self.starter_account_key.ok_or_else(|| 
                CustomError::StarterKeyNotProvided)?, 
            is_signer: payer_signing, 
            is_writable: payer_writing
        });

        accounts.push(AccountMeta { 
            pubkey: system_program::ID, 
            is_signer: false, 
            is_writable: false
        });

        let (commit_account, commit_account_bump) = 
            self.get_commit_account(program_signer_keys)?;

        self.commit_account_bump = commit_account_bump;

        accounts.push(
            AccountMeta { 
                pubkey: commit_account, 
                is_signer: false, 
                is_writable: true 
        });

        return Ok(accounts);
    }

    // This function does not check for bounds
    pub fn get_commit_account(&mut self, program_signer_keys:Vec<Pubkey>)-> 
        Result<(Pubkey, u8), CustomError> {

        let signers_count = self.signer_commit_account_keys.len();

        let accounts_count = signers_count + 
            self.non_signer_commit_account_keys.len() + 
            1; // For the program account

        let mut commit_seeds:Vec<&[u8]> = Vec::with_capacity(
            1 + 1 + 
            1 +
            1 + 1
        );
   
        let mut temp_account_indices = Vec::with_capacity(CONFIG_MAX_ACCOUNTS);

        temp_account_indices.extend_from_slice(
            &self.acct_indices.as_slice()[..self.non_signer_commit_account_keys.len()]);

        temp_account_indices.push(self.total_accounts_count);

        temp_account_indices.extend_from_slice(
            &self.acct_indices.as_slice()[self.non_signer_commit_account_keys.len()..]);

        temp_account_indices.resize(CONFIG_MAX_ACCOUNTS, 0);

        println!("Account indices:- {:?}", temp_account_indices);
        
        commit_seeds.push(temp_account_indices.as_slice());

        let signer_count_bytes: &[u8] = &[u8::try_from(signers_count).
            map_err(|_| CustomError::InvalidSignerCount)?];

        commit_seeds.push(signer_count_bytes);

        let mut keys_slice_vec:Vec<&[u8]> = Vec::with_capacity(accounts_count);
        
        for key in self.non_signer_commit_account_keys.iter(){
            keys_slice_vec.push(key.as_ref());
        }
        
        keys_slice_vec.push(self.program_id.as_ref());

        for key in program_signer_keys.iter(){
            keys_slice_vec.push(key.as_ref());
        }
            
        let keys_hash:[u8;32] = hash::hashv(keys_slice_vec.as_slice()).to_bytes();

        println!("Accounts key hash:- {:?}", keys_hash);
        
        commit_seeds.push(keys_hash.as_ref());

        let data_offset:i16 = -1;
        let mut data_offset_bytes = data_offset.to_le_bytes();

        let mut data_hash = [0u8;32];

        match &self.instruction_data_meta{
            DataCommitType::NoData=>{
                commit_seeds.push(&data_offset_bytes);
                
                commit_seeds.push(&data_hash);
            },
            DataCommitType::AnyData=>{
                data_offset_bytes = [0;2];
                commit_seeds.push(&data_offset_bytes);
                
                commit_seeds.push(&data_hash);
            },
            DataCommitType::Data(offset, data)=>{

                data_offset_bytes = offset.to_le_bytes();
                commit_seeds.push(&data_offset_bytes);
                
                data_hash = hash::hashv(&[data.as_slice()]).to_bytes();
                commit_seeds.push(data_hash.as_ref());
            }
        }

        self.instruction_data_hash = data_hash;

        let (commit_account , commit_account_bump) = 
            Pubkey::find_program_address(commit_seeds.as_slice(), &constants::PROGRAM_ID);

        Ok((commit_account, commit_account_bump))
    }
}