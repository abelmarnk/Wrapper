use bytemuck::{
    Pod, 
    Zeroable
};
use pinocchio::{
    log::sol_log_slice, program_error::ProgramError, pubkey::Pubkey
};
use crate::{
    WrapperError, constants::{COMMIT_SEEDS_LEN, CONFIG_MAX_ACCOUNTS, CONFIG_MAX_SIGNERS, HASH_LENGTH}, state::condition::CommitCondition, utils::hashv
};

type HashType = [u8;32];

/// Stores the instruction data commit type for
/// a commit, it can either be `NoData`, in that no 
/// data is commited to, `Data`, in that some specific
/// data is commited to and `AnyData` in that any data
/// is valid. 
/// NoData => 0
/// Data => 1
/// AnyData => 2
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DataCommitType{
    commit_type:[u8;1]
}

/// Enum for the data commit type
/// NoData => 0
/// Data => 1
/// AnyData => 2
pub enum DataCommitTypeEnum{
    NoData,
    Data,
    AnyData
}


impl From<DataCommitTypeEnum> for DataCommitType {
    fn from(value: DataCommitTypeEnum) -> Self {
        let byte = match value {
            DataCommitTypeEnum::NoData => 0,
            DataCommitTypeEnum::Data => 1,
            DataCommitTypeEnum::AnyData => 2,
        };
        DataCommitType { commit_type: [byte] }
    }
}

impl TryFrom<DataCommitType> for DataCommitTypeEnum {
    
    fn try_from(value: DataCommitType) -> Result<Self, Self::Error> {
        match value.commit_type {
            [0] => Ok(DataCommitTypeEnum::NoData),
            [1] => Ok(DataCommitTypeEnum::Data),
            [2] => Ok(DataCommitTypeEnum::AnyData),
            _ => Err(WrapperError::InvalidDataCommitType),
        }
    }

    type Error = WrapperError;
}

/// Stores information about the configuration as well as the
/// signers and the starter
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Config{
    pub base:ConfigBase,
    pub starter_key:Pubkey,
    pub signer_keys: [Pubkey; CONFIG_MAX_SIGNERS]
}

impl Config{
    pub const LEN:usize = core::mem::size_of::<Config>();
}

impl ConfigBase{
    pub const LEN:usize = core::mem::size_of::<ConfigBase>();
}

impl ConfigBase{
    /// Checks whether the provided instruction data length matches the commit type
    /// recorded in this configuration.
    ///
    /// - `NoData`: length must be exactly zero.
    /// - `AnyData`: any length is accepted.
    /// - `Data`: length must match the committed instruction data length stored
    ///   in this configuration.
    pub fn length_matches_commit_type(&self, length: usize) -> bool {

        // Extract enum from the commit type
        let commit_type = 
            DataCommitTypeEnum::try_from(self.instruction_data_commit_type);

        // Committed data length stored in config
        let committed_length =
            u16::from_le_bytes(self.instruction_data_length) as usize;

        match commit_type {
            Ok(DataCommitTypeEnum::NoData) => length == 0,
            Ok(DataCommitTypeEnum::AnyData) => true,
            Ok(DataCommitTypeEnum::Data) => length == committed_length,
            Err(_) => false, // Invalid commit type
        }
    }

    /// Updates the commit condition, see CommitCondition::update for more
    /// information
    #[inline(always)]
    pub fn update_condition(&mut self)->Result<(), ProgramError>{
        self.condition.update()
    }

    /// Gets the seeds used to create the commit account:-
    /// 
    /// account_indices,
    /// signer_count,
    /// accounts_hash,
    /// instruction_data_commit_type,
    /// instruction_data_offset,
    /// data_hash,
    /// commit_bump
    /// 
    /// Based on the different forms that the data commitment can take different actions would need to be
    /// performed, when the data is passed and the data commit type is `Data`, then we would need to confirm
    /// the hash, if data is passed and the commit type is set to `NoData`, then we confirm no data is passed
    /// if the commit type is set to `AnyData`, we we check nothing.
    /// If no data is passed we make no checks.
    pub fn get_commit_seeds<'a, 'b>(&'a self, commit_accounts_hash:&'b[u8], 
        maybe_data:Option<&[u8]>)->Result<[&'b[u8]; COMMIT_SEEDS_LEN], ProgramError>
    where
        'a:'b,
    {
    
        
        match maybe_data{
            Some(data)=>{
                
            let mut instruction_data_hash:[u8;HASH_LENGTH] = [0;HASH_LENGTH];

            match DataCommitTypeEnum::try_from(self.instruction_data_commit_type)?{
                // If some data is passed we extract it and compare it with the hash
                DataCommitTypeEnum::Data=>{
                    let instruction_data_offset = usize::from(u16::from_le_bytes(self.instruction_data_offset));
                    let instruction_data_length = usize::from(u16::from_le_bytes(self.instruction_data_length));
                    
                    if data.len().lt(&(instruction_data_offset + instruction_data_length)){
                        return Err(ProgramError::InvalidInstructionData);
                    }

                    let commit_instruction_data = [
                        data[instruction_data_offset..(instruction_data_offset + instruction_data_length)].
                        as_ref()];
                    
                    hashv(&commit_instruction_data, 
                    &mut instruction_data_hash);

                    if instruction_data_hash != self.instruction_data_hash {
                        return Err(ProgramError::InvalidInstructionData);
                    }
                },
                DataCommitTypeEnum::NoData => { 
                    if data.len().ne(&0){ // We confirm no data is passed
                        return Err(ProgramError::InvalidInstructionData);
                    }
                },
                DataCommitTypeEnum::AnyData => {} // Ok
            }
        }
        None => {}
    };

    sol_log_slice(self.account_indices.as_ref());
    
    sol_log_slice(commit_accounts_hash.as_ref());
    
    // account_indices,
    // signer_count,
    // accounts_hash,
    // instruction_data_commit_type,
    // instruction_data_offset,
    // data_hash,
    // commit_bump
    let commit_seeds:[&[u8];COMMIT_SEEDS_LEN] = [
                                    self.account_indices.as_ref(),
                                    &self.signer_count, 
                                    commit_accounts_hash.as_ref(),
                                    self.instruction_data_commit_type.commit_type.as_ref(),
                                    self.instruction_data_offset.as_ref(),
                                    self.instruction_data_hash.as_ref(),
                                    &self.commit_bump
                                ];
    
    Ok(commit_seeds)

    }
}

/// Stores information about the configuration
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ConfigBase{
    pub account_count: [u8;1], 
    pub account_indices: [u8; CONFIG_MAX_ACCOUNTS],
    pub signer_count: [u8;1],
    pub signer_bumps: [u8; CONFIG_MAX_SIGNERS],
    pub instruction_data_commit_type:DataCommitType,
    pub instruction_data_length: [u8;2], 
    pub instruction_data_offset: [u8; 2],
    pub instruction_data_hash: HashType,
    pub commit_bump: [u8;1],
    pub condition: CommitCondition,
}


