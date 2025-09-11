use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey,
    ProgramResult, instruction::{Seed, Signer},
};

use pinocchio_system::instructions::Transfer;
pub struct WithdrawNative<'a> {
    pub signer: &'a AccountInfo,
    pub program_signer: &'a AccountInfo,
    pub amount: u64,
    pub bump: u8,
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for WithdrawNative<'a> {
    type Error = ProgramError;
    #[inline(always)]
    fn try_from(value: (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        // Destructure accounts
        let [signer, program_signer] = value.0 else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let data = value.1;

        // Require enough bytes for u64 + u8
        let required_len = core::mem::size_of::<u64>() + core::mem::size_of::<u8>();
        if data.len() < required_len {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Decode fields
        let amount = u64::from_le_bytes(
            data[..core::mem::size_of::<u64>()].try_into().unwrap());
        let bump = *data.last().unwrap();

        // Signer must sign
        if !signer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Verify PDA derivation
        let expected_program_signer =
            pubkey::create_program_address(&[signer.key().as_ref(), &[bump]], 
                &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?;

        if !program_signer.key().eq(&expected_program_signer) {
            return Err(ProgramError::InvalidSeeds);
        }

        Ok(WithdrawNative {
            signer,
            program_signer,
            amount,
            bump:bump,
        })
    }
}

impl<'a> WithdrawNative<'a> {
    #[inline(always)]
    pub fn process(&self) -> ProgramResult {
        // Seeds for PDA signer
        let seeds: [&[u8]; 2] = [self.signer.key().as_ref(), &[self.bump]];
        let seeds: [Seed; 2] = seeds.map(Seed::from);
        let signer = Signer::from(seeds.as_ref());

        // Perform transfer
        Transfer {
            from: self.program_signer,
            to: self.signer,
            lamports: self.amount,
        }
        .invoke_signed(&[signer])
    }
}
