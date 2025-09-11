
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

    InvalidAccountKeysCount,

    InvalidAccountIndex,

    StarterKeyNotProvided,

    ArithmeticError,

    InvalidData
}