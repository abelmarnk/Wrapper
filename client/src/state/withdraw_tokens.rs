use borsh::BorshSerialize;

pub use solana_sdk::{
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
};

use crate::constants;

#[derive(BorshSerialize)]
pub struct WithdrawTokenData {
    pub amount: u64,
    pub bump: u8,
    pub decimals: u8,
}

pub struct WithdrawToken {
    pub signer: Pubkey,
    pub program_signer: Pubkey,
    pub mint: Pubkey,
    pub program_signer_ata: Pubkey,
    pub token_program: Pubkey,
    pub data:WithdrawTokenData
}

impl WithdrawToken {
    pub fn new(
        signer: Pubkey,
        program_signer: Pubkey,
        mint: Pubkey,
        program_signer_ata: Pubkey,
        token_program: Pubkey,
        amount: u64,
        bump: u8,
        decimals: u8,
    ) -> Self {
        Self {
            signer,
            program_signer,
            mint,
            program_signer_ata,
            token_program,
            data: WithdrawTokenData {
            amount,
            bump,
            decimals,
            }
        }
    }

    pub fn to_instruction(&self) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(self.signer, true),          
            AccountMeta::new_readonly(self.program_signer, false),
            AccountMeta::new_readonly(self.mint, false),          
            AccountMeta::new(self.program_signer_ata, false),     
            AccountMeta::new_readonly(self.token_program, false), 
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