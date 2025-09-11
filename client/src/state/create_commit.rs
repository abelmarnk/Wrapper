
use bytemuck::{Pod, Zeroable};
use solana_sdk::{
    instruction::Instruction, 
    pubkey::Pubkey
};

use crate::{
    condition::CommitCondition, 
    constants, 
    Base, 
    CustomError, 
    DataCommitType, 
    RawCommitCondition, 
    CONFIG_MAX_ACCOUNTS, 
    CONFIG_MAX_SIGNERS
};


#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RawConfigBase{
    account_count: [u8;1], // This is a u8
    account_indices: [u8; CONFIG_MAX_ACCOUNTS],
    signer_count: [u8;1], // This is a u8
    signer_bumps: [u8; CONFIG_MAX_SIGNERS],
    instruction_data_length: [u8;1], // This is a u8
    instruction_data_offset: [u8; 2], // This is an i16
    instruction_data_hash: [u8; 32], // This is a hash
    commit_bump: [u8;1], // This is a u8
    condition: RawCommitCondition,
}

impl RawConfigBase{
    pub const LEN:usize = core::mem::size_of::<RawConfigBase>();
}


pub struct CreateCommit{
    base: Base,
    condition: CommitCondition,
}
impl CreateCommit{
    pub fn new(
        program_id: Pubkey,
        signer_commit_account_keys: Vec<Pubkey>,
        non_signer_commit_account_keys: Vec<Pubkey>,
        starter_account_key: Option<Pubkey>,
        total_accounts_count:u8,
        acct_indices: Vec<u8>,
        instruction_data_meta: DataCommitType,
        condition: CommitCondition,
    ) -> Result<Self, CustomError> {

        let base = Base::new(
            program_id,
            signer_commit_account_keys,
            non_signer_commit_account_keys,
            starter_account_key,
            total_accounts_count,
            acct_indices,
            instruction_data_meta
        )?;

        Ok(Self {
            base,
            condition
        })
    }

    pub fn new_from_base(base:Base, condition: CommitCondition)->Self{
        Self { 
            base, 
            condition
        }
    }

    pub fn make_instruction(base: &mut Base, condition: &mut CommitCondition)->
            Result<Instruction, CustomError> {
        println!("Signer count:- {:?}", base.signer_commit_account_keys.len());

        println!("non-Signer count:- {:?}", base.non_signer_commit_account_keys.len());

        let accounts = base.get_accounts_and_mid_state(true, true)?;

        let mut instruction_data = vec![0;RawConfigBase::LEN + 1];

        {
            let raw_config_base_mut_ref = 
                bytemuck::try_from_bytes_mut::<RawConfigBase>(
                    &mut instruction_data.as_mut_slice()[0..RawConfigBase::LEN]
                ).map_err(|_| CustomError::InvalidData)?;

            raw_config_base_mut_ref.account_count = u8::try_from(base.non_signer_commit_account_keys.len() + 
                base.signer_commit_account_keys.len() + 1).unwrap().to_le_bytes();

            for (position, index) in base.acct_indices.iter().enumerate().
                take(base.non_signer_commit_account_keys.len()){
                raw_config_base_mut_ref.account_indices[position] = *index;
            }

            raw_config_base_mut_ref.account_indices[base.non_signer_commit_account_keys.len()] = base.total_accounts_count;

            for (position, index) in base.acct_indices.iter().enumerate().
                skip(base.non_signer_commit_account_keys.len()){
                raw_config_base_mut_ref.account_indices[position + 1] = *index;
            }

            println!("Account indices::- {:?}", raw_config_base_mut_ref.account_indices);

            raw_config_base_mut_ref.signer_count = u8::try_from(base.signer_commit_account_keys.len()).
            map_err(|_| CustomError::ArithmeticError)?.to_le_bytes();

            println!("Signer count:- {:?}", base.signer_commit_account_keys.len());

            for (position, bump) in base.signer_bumps.iter().enumerate(){
                raw_config_base_mut_ref.signer_bumps[position] = *bump;
            }

            match &base.instruction_data_meta{
                DataCommitType::NoData=>{
                    raw_config_base_mut_ref.instruction_data_length = 0u8.to_le_bytes();
                    raw_config_base_mut_ref.instruction_data_offset = (-1i16).to_le_bytes();
                    raw_config_base_mut_ref.instruction_data_hash = [0u8;32];
                },
                DataCommitType::AnyData=>{
                    raw_config_base_mut_ref.instruction_data_length = 0u8.to_le_bytes();
                    raw_config_base_mut_ref.instruction_data_offset = 0u16.to_le_bytes();
                    raw_config_base_mut_ref.instruction_data_hash = [0u8;32];
                },
                DataCommitType::Data(offset, data)=>{
                    raw_config_base_mut_ref.instruction_data_length = 
                        u8::try_from(data.len()).map_err(|_| CustomError::ArithmeticError)?.to_le_bytes();
                    raw_config_base_mut_ref.instruction_data_offset = offset.to_le_bytes();
                    raw_config_base_mut_ref.instruction_data_hash = base.instruction_data_hash;
                }
            }

            raw_config_base_mut_ref.commit_bump = base.commit_account_bump.to_le_bytes();

            raw_config_base_mut_ref.condition = RawCommitCondition::from(condition);
        }

        let discriminator_position = instruction_data.len() - 1;
        instruction_data[discriminator_position] = 1; // Create commit instruction discriminator

        println!("Instruction data size: {:?}", instruction_data.len());

        Ok(Instruction{
            program_id: constants::PROGRAM_ID,
            accounts,
            data: instruction_data,
        })



    }

    pub fn to_instruction(&mut self) -> 
            Result<Instruction, CustomError> {
            CreateCommit::make_instruction(&mut self.base, &mut self.condition)
    }
}


