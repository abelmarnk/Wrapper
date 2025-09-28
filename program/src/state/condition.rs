use bytemuck::{
    Pod, 
    Zeroable
};

use pinocchio::{
    msg, program_error::ProgramError, sysvars::{
        clock::Clock, 
        Sysvar
    }
};

use crate::{
    WrapperError
};

pub const COMMIT_CONDITION_DATA_SIZE:usize = 20;

#[repr(u8)]
pub enum CommitConditionTag {
    Default = 0,
    Count = 1,
    BeforeTimestamp = 2,
    AfterTimestamp = 3,
    CountBeforeTimestamp = 4,
    CountAfterTimestamp = 5,
    OneOffCountBetweenTimestamp = 6,
    RepeatCountBetweenTimestamp = 7
}

impl TryFrom<u8> for CommitConditionTag {
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CommitConditionTag::Default),
            1 => Ok(CommitConditionTag::Count),
            2 => Ok(CommitConditionTag::BeforeTimestamp),
            3 => Ok(CommitConditionTag::AfterTimestamp),
            4 => Ok(CommitConditionTag::CountBeforeTimestamp),
            5 => Ok(CommitConditionTag::CountAfterTimestamp),
            6 => Ok(CommitConditionTag::OneOffCountBetweenTimestamp),
            7 => Ok(CommitConditionTag::RepeatCountBetweenTimestamp),
            _ => Err(WrapperError::InvalidCommitConditionTag),
        }
    }

    type Error = WrapperError;
}

impl Into<u8> for CommitConditionTag{
    #[inline(always)]
    fn into(self) -> u8 {
        return self as u8
    }
}

impl CommitConditionTag {
    #[inline(always)]
    pub fn is_valid(value:u8) -> bool {
        match value {
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => true,
            _ => false,
        }
    }
    
}

/// Stores information about the time form of the commitment made
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CommitCondition {
    pub tag: u8,
    pub data: [u8; COMMIT_CONDITION_DATA_SIZE],
}

impl CommitCondition{
    pub const LEN:usize = core::mem::size_of::<CommitCondition>();
}

impl CommitCondition{

    /// Checks if the tag is valid, if so it returns it
    #[inline(always)]
    pub fn is_valid(&self)->Result<CommitConditionTag, WrapperError>{
        CommitConditionTag::try_from(self.tag)
    }

    /// Updates the count, returns an error if the count is exhausted
    #[inline(always)]
    fn update_count(count: &mut u32)->Result<(), ProgramError>{
        if (*count).gt(&0){
            *count = count.saturating_sub(1);
            return Ok(());
        }
    
        Err(WrapperError::CountExhausted.into())
    }

    /// Get all the data fields of the commit
    #[inline]
    fn get_data_fields(&mut self) -> Result<(u32, i64, u32, u32), ProgramError> {
        let (count_bytes, rest) = self.data.split_at_mut(4);

        msg!("-6.1");

        let count = u32::from_le_bytes(count_bytes.try_into().unwrap());

        msg!("-6.2");

        let (timestamp_bytes, rest) = rest.split_at_mut(8);

        msg!("-6.3");

        let timestamp = i64::from_le_bytes(timestamp_bytes.try_into().unwrap());

        msg!("-6.4");

        let (offset_bytes, repeat_count_bytes) = rest.split_at_mut(4);

        msg!("-6.5");

        let offset = u32::from_le_bytes(offset_bytes.try_into().unwrap());

        msg!("-6.6");

        let repeat_count = u32::from_le_bytes(repeat_count_bytes.try_into().unwrap());

        msg!("-6.7");

        Ok((count, timestamp, offset, repeat_count))
    }

    /// Set all the changing data fields of the commit
    #[inline]
    fn set_data_fields(
        &mut self,
        count: u32,
        timestamp: i64,
    ) -> Result<(), ProgramError> {
        let (count_bytes, rest) = self.data.
            split_at_mut(core::mem::size_of::<u32>());
        count_bytes.copy_from_slice(&count.to_le_bytes());

        msg!("-7.1");

        let (timestamp_bytes, _) = rest.
        split_at_mut(core::mem::size_of::<i64>());
        timestamp_bytes.copy_from_slice(&timestamp.to_le_bytes());

        msg!("-7.2");

        Ok(())
    }

    pub fn update(&mut self)->Result<(), ProgramError>{
        let current_timestamp = Clock::get()?.unix_timestamp;

        let tag = CommitConditionTag::try_from(self.tag)?;

        let (mut count, mut timestamp, offset, repeat_count) = 
            self.get_data_fields()?;

        match tag {
            CommitConditionTag::Count=>{
                Self::update_count(&mut count)
            },
            CommitConditionTag::BeforeTimestamp=>{
                if current_timestamp.ge(&timestamp){
                    return Err(WrapperError::TooLate.into());
                }

                Ok(())
            },
            CommitConditionTag::AfterTimestamp=>{
                if current_timestamp.le(&timestamp){
                    return Err(WrapperError::TooEarly.into());
                }

                Ok(())
            },
            CommitConditionTag::CountBeforeTimestamp=>{
                if current_timestamp.ge(&mut timestamp){
                    return Err(WrapperError::TooLate.into());
                }

                Self::update_count(&mut count)
            },
            CommitConditionTag::CountAfterTimestamp=>{
                if current_timestamp.le(&timestamp){
                    return Err(WrapperError::TooEarly.into());       
                }

                Self::update_count(&mut count)
            },
            CommitConditionTag::OneOffCountBetweenTimestamp=>{
                let start = timestamp;
                let end = start.checked_add(i64::from(offset)).
                    ok_or_else(||ProgramError::ArithmeticOverflow)?;

                if current_timestamp.lt(&start){
                    return Err(WrapperError::TooEarly.into());
                }
                else if current_timestamp.gt(&end) {
                    return Err(WrapperError::TooLate.into());   
                }

                Self::update_count(&mut count)
            },
            CommitConditionTag::RepeatCountBetweenTimestamp=>{
                let end = timestamp.checked_add(i64::from(offset)).
                    ok_or_else(||ProgramError::ArithmeticOverflow)?;

                if current_timestamp.lt(&timestamp){
                    return Err(WrapperError::TooEarly.into());
                }
                else if current_timestamp.gt(&end) {
                    timestamp = current_timestamp;
                    count = repeat_count;
                }

                Self::update_count(&mut count)
            },
            CommitConditionTag::Default=>{
                Err(WrapperError::ConditionNotSet.into())
            }
        }?;

        self.set_data_fields(count, timestamp)

    }
}