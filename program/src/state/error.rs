use pinocchio::program_error::ProgramError;

#[derive(Debug, Copy, Clone)]
pub enum CustomError {
    OutOfTimeRange,

    ConditionNotSet,

    TooEarly,

    TooLate,

    CountExhausted,

    InvalidConfiguration,

    InvalidAccountCount,

    InvalidSignerCount,

    SignerCountGreaterThanAccountCount,

    InvalidSignerSeeds,

    InvalidCommitCondition,

    InvalidInstructionDataLength,

    InvalidCommitConditionTag,

    InvalidAccountKeysCount
}

impl From<CustomError> for ProgramError {
    fn from(error:CustomError)->ProgramError{
        return ProgramError::Custom(error as u32);
    }
}