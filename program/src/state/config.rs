use bytemuck::{
    Pod, 
    Zeroable
};
use pinocchio::{
    log::sol_log_slice, program_error::ProgramError, pubkey::Pubkey
};
use crate::{
    constants::HASH_LENGTH, 
    state::condition::CommitCondition, 
    utils::hashv
};

pub const CONFIG_MAX_SIGNERS:usize = 4;
pub const CONFIG_MAX_ACCOUNTS:usize = 8;
pub const CONFIG_MIN_ACCOUNTS:usize = 3;

type HashType = [u8;32];

pub enum DataCommitType{
    NoData,
    AnyData,
    Data
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Config{
    pub base:ConfigBase,
    pub starter_key:Pubkey,
    pub signer_keys: [Pubkey; CONFIG_MAX_SIGNERS]
}

impl Config{
    // Unsafe helpers

}

impl Config{
    pub const LEN:usize = core::mem::size_of::<Config>();
}

impl ConfigBase{
    pub const LEN:usize = core::mem::size_of::<ConfigBase>();
}

impl ConfigBase{

    pub fn length_matches_commit_type(&self, length:usize)->bool{
        let instruction_data_offset = isize::from(
            i16::from_le_bytes(self.instruction_data_offset));

        let instruction_data_length = usize::from(
            u8::from_le_bytes(self.instruction_data_length));

        if instruction_data_offset.lt(&0){
            if !length.eq(&0){
                return false;
            }
        }
        else if instruction_data_offset.gt(&0){
            if length.lt(
                &(usize::try_from(instruction_data_offset).unwrap() + instruction_data_length - 1)){
                    return false;
            }
        }
        true
    }

    #[inline(always)]
    pub fn update_condition(&mut self)->Result<(), ProgramError>{
        self.condition.update()
    }


    pub fn get_commit_seeds<'a, 'b>(&'a self, commit_accounts_hash:&'b[u8], 
        maybe_data:Option<&[u8]>)->Result<[&'b[u8];6], ProgramError>
    where
        'a:'b,
    {
    
    let mut instruction_data_hash:[u8;HASH_LENGTH] = [0;HASH_LENGTH];

    match maybe_data{
        Some(data)=>{ 
            let instruction_data_offset = i16::from_le_bytes(self.instruction_data_offset);

            if instruction_data_offset.gt(&0){
                let instruction_data_offset = usize::try_from(instruction_data_offset).unwrap();
                let instruction_data_length = usize::from(u8::from_le_bytes(self.instruction_data_length));
                
                let commit_instruction_data = [
                    data[instruction_data_offset..(instruction_data_offset + instruction_data_length)].
                    as_ref()];
                
                hashv(&commit_instruction_data, 
                &mut instruction_data_hash);

                if instruction_data_hash != self.instruction_data_hash {
                    return Err(ProgramError::InvalidInstructionData);
                }
            }
        }
        None =>{

        }
    };

    // Declare Commit seeds
    /*
        account_indices,
        signer_count,
        accounts_hash
        instruction_data_offset,
        data_hash,
        commit_bump
    */ 
    sol_log_slice(self.account_indices.as_ref());

    sol_log_slice(commit_accounts_hash.as_ref());

    let commit_seeds:[&[u8];6] = [
                                    self.account_indices.as_ref(),
                                    &self.signer_count, 
                                    commit_accounts_hash.as_ref(),
                                    self.instruction_data_offset.as_ref(),
                                    self.instruction_data_hash.as_ref(),
                                    &self.commit_bump
                                ];
    
    Ok(commit_seeds)

    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ConfigBase{
    pub account_count: [u8;1], // This is a u8
    pub account_indices: [u8; CONFIG_MAX_ACCOUNTS],
    pub signer_count: [u8;1], // This is a u8
    pub signer_bumps: [u8; CONFIG_MAX_SIGNERS],
    pub instruction_data_length: [u8;1], // This is a u8
    pub instruction_data_offset: [u8; 2], // This is an i16
    pub instruction_data_hash: HashType,
    pub commit_bump: [u8;1], // This is a u8
    pub condition: CommitCondition,
}

