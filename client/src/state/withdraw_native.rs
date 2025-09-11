use borsh::BorshSerialize;

pub use solana_sdk::{
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
};

use crate::constants;

#[derive(BorshSerialize)]
pub struct WithdrawNativeData {
    pub amount: u64,
    pub bump: u8,
}

pub struct WithdrawNative {
    pub signer: Pubkey,
    pub program_signer: Pubkey,
    pub system_program: Pubkey,
    pub data: WithdrawNativeData,
}

impl WithdrawNative {
    pub fn new(
        signer: Pubkey,
        program_signer: Pubkey,
        system_program: Pubkey,
        amount: u64,
        bump: u8,
    ) -> Self {
        Self {
            signer,
            program_signer,
            system_program,
            data: WithdrawNativeData {
                amount,
                bump,
            },
        }
    }

    pub fn to_instruction(&self) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(self.signer, true),              // signer (signer)
            AccountMeta::new(self.program_signer, false),     // program_signer (writable)
            AccountMeta::new_readonly(self.system_program, false), // system_program
        ];

        let mut data: Vec<u8> = vec![];
        self.data.serialize(&mut data)?;

        Ok(Instruction {
            program_id: constants::PROGRAM_ID,
            accounts,
            data,
        })
    }
}