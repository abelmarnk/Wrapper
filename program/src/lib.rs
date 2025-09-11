#![no_std]
use pinocchio::{
    account_info::AccountInfo, default_allocator, msg, nostd_panic_handler, program_entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult
};

use pinocchio_pubkey::{
    declare_id
};

pub mod handlers;
pub use handlers::*;

pub mod state;
pub use state::*;

pub mod helpers;
pub use helpers::*;

default_allocator!();
nostd_panic_handler!();

program_entrypoint!(process_instruction);

declare_id!("GQnJT5HuuV4FUqQu5WS2WisHHSWLosC3WBtJS7fcVMTM");

pub enum InstructionTag{
    Entry,
    CreateCommit,
    ChangeCommit,
    CreatePda,
    WithdrawNative,
    WithdrawToken,
    CloseCommit
}

impl TryFrom<u8> for InstructionTag{
    fn try_from(tag:u8)-> Result<Self, Self::Error>{
        match tag{
            0 => Ok(InstructionTag::Entry),
            1 => Ok(InstructionTag::CreateCommit),
            2 => Ok(InstructionTag::ChangeCommit),
            3 => Ok(InstructionTag::CreatePda),
            4 => Ok(InstructionTag::WithdrawNative),
            5 => Ok(InstructionTag::WithdrawToken),
            6 => Ok(InstructionTag::CloseCommit),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }

    type Error = ProgramError;
}

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {


    let (tag, data) = instruction_data.split_last().
        ok_or_else(|| ProgramError::InvalidInstructionData)?;


    match InstructionTag::try_from(*tag)? {
        InstructionTag::Entry =>{
            msg!("Entered entry instruction");

            let entry = Entry::try_from((accounts, data))?;

            entry.process()
        },
        InstructionTag::CreateCommit =>{

            msg!("Entered create commit instruction");

            let create_commit = CreateCommit::try_from((accounts, data))?;

            create_commit.process()
        },
        InstructionTag::ChangeCommit =>{
            let mut change_commit = ChangeCommit::try_from((accounts, data))?;

            change_commit.process()
        },
        InstructionTag::CreatePda=>{
            // This instruction is not necessary since accounts only hold SOL, though that might change in the future.
            Ok(())
        },
        InstructionTag::WithdrawNative=>{
            let withdraw_native = WithdrawNative::try_from((accounts, data))?;

            withdraw_native.process()
        },
        InstructionTag::WithdrawToken=>{
            let withdraw_token = WithdrawToken::try_from((accounts, data))?;

            withdraw_token.process()
        },
        InstructionTag::CloseCommit=>{
            let close_commit = CloseCommit::try_from(accounts)?;

            close_commit.process()
        }
    }
}
