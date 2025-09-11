use bytemuck::{Pod, Zeroable};

pub const COMMIT_CONDITION_DATA_SIZE: usize = 20; // Max size of data in bytes

#[repr(u8)]
#[derive(Default)]
pub enum CommitCondition {
    #[default]
    Default = 0,
    Count(u32) = 1, // Count
    BeforeTimestamp(i64) = 2, // End
    AfterTimestamp(i64) = 3, // Start
    CountBeforeTimestamp(u32, i64) = 4, // Count, End
    CountAfterTimestamp(u32, i64) = 5, // Count, Start
    OneOffCountBetweenTimestamp(u32, u32, i64) = 6, // Count, Offset, Start
    RepeatCountBetweenTimestamp(u32, u32, i64, u32) = 7, // Count, Offset, Start
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RawCommitCondition {
    pub tag: u8,
    pub data: [u8; COMMIT_CONDITION_DATA_SIZE],
}

impl From<&CommitCondition> for RawCommitCondition {
    fn from(condition: &CommitCondition) -> Self {
        let tag = condition.get_tag();
        let data = condition.get_data();
        RawCommitCondition { tag, data }
    }
}

impl From<&mut CommitCondition> for RawCommitCondition {
    fn from(condition: &mut CommitCondition) -> Self {
        let tag = condition.get_tag();
        let data = condition.get_data();
        RawCommitCondition { tag, data }
    }
}

impl CommitCondition{
    fn get_tag(&self) -> u8 {
        match self {
            CommitCondition::Default => 0,
            CommitCondition::Count(_) => 1,
            CommitCondition::BeforeTimestamp(_) => 2,
            CommitCondition::AfterTimestamp(_) => 3,
            CommitCondition::CountBeforeTimestamp(_, _) => 4,
            CommitCondition::CountAfterTimestamp(_, _) => 5,
            CommitCondition::OneOffCountBetweenTimestamp(_, _, _) => 6,
            CommitCondition::RepeatCountBetweenTimestamp(_, _, _, _) => 7,
        }
    }

    fn get_data(&self)-> [u8;COMMIT_CONDITION_DATA_SIZE]{

        match self {
            CommitCondition::Default => [0; COMMIT_CONDITION_DATA_SIZE],
            CommitCondition::Count(count) => {
                let mut data = [0; COMMIT_CONDITION_DATA_SIZE];
                data[1..5].copy_from_slice(&count.to_le_bytes());
                data
            }
            CommitCondition::BeforeTimestamp(ts) => {
                let mut data = [0; COMMIT_CONDITION_DATA_SIZE];
                data[5..13].copy_from_slice(&ts.to_le_bytes());
                data
            }
            CommitCondition::AfterTimestamp(ts) => {
                let mut data = [0; COMMIT_CONDITION_DATA_SIZE];
                data[5..13].copy_from_slice(&ts.to_le_bytes());
                data
            }
            CommitCondition::CountBeforeTimestamp(count, ts) => {
                let mut data = [0; COMMIT_CONDITION_DATA_SIZE];
                data[1..5].copy_from_slice(&count.to_le_bytes());
                data[5..13].copy_from_slice(&ts.to_le_bytes());
                data
            }
            CommitCondition::CountAfterTimestamp(count, ts) => {
                let mut data = [0; COMMIT_CONDITION_DATA_SIZE];
                data[1..5].copy_from_slice(&count.to_le_bytes());
                data[5..13].copy_from_slice(&ts.to_le_bytes());
                data
            }
            CommitCondition::OneOffCountBetweenTimestamp(count, offset, ts) => {
                let mut data = [0; COMMIT_CONDITION_DATA_SIZE];
                data[1..5].copy_from_slice(&count.to_le_bytes());
                data[5..13].copy_from_slice(&ts.to_le_bytes());
                data[13..17].copy_from_slice(&offset.to_le_bytes());
                data
            }
            CommitCondition::RepeatCountBetweenTimestamp(count, offset, ts, repeat) => {
                let mut data = [0; COMMIT_CONDITION_DATA_SIZE];
                data[1..5].copy_from_slice(&count.to_le_bytes());
                data[5..13].copy_from_slice(&ts.to_le_bytes());
                data[13..17].copy_from_slice(&offset.to_le_bytes());
                data[17..21].copy_from_slice(&repeat.to_le_bytes());
                data
            }
        }
    }

    pub fn to_expected_bytes(&self) -> [u8; 1 + COMMIT_CONDITION_DATA_SIZE] {
        let mut data = [0u8; 
            1 + // For the tag
            COMMIT_CONDITION_DATA_SIZE // For the data
        ];

        data[0] = self.get_tag();

        data[1..].copy_from_slice(&self.get_data());

        data
    }
}