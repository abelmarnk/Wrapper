use pinocchio::program_error::ProgramError;

#[derive(Debug, Copy, Clone)]
pub enum WrapperError {
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

    InvalidAccountKeysCount,

    InvalidDataCommitType
}

impl From<WrapperError> for ProgramError {
    fn from(error:WrapperError)->ProgramError{
        return ProgramError::Custom(error as u32);
    }
}