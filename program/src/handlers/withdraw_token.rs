use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey,
    ProgramResult, instruction::{Seed, Signer},
};
use pinocchio_token::instructions::TransferChecked;



pub struct WithdrawToken<'a> {
    pub signer: &'a AccountInfo,
    pub program_signer: &'a AccountInfo,
    pub mint: &'a AccountInfo,
    pub program_signer_ata: &'a AccountInfo,
    pub signer_ata: &'a AccountInfo,
    pub amount: u64,
    pub decimals: u8,
    pub bump: u8,
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for WithdrawToken<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        // Destructure accounts: signer, program_signer, mint, program_signer_ata, signer_ata
        let [signer, program_signer, mint, program_signer_ata, signer_ata] = value.0 else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let data = value.1;

        // Require enough bytes for u64 + u8 + u8
        let required_len = core::mem::size_of::<u64>() 
                         + core::mem::size_of::<u8>() 
                         + core::mem::size_of::<u8>();
        if data.len() < required_len {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Decode fields
        let amount_bytes: [u8; 8] = data[0..8].try_into().unwrap();
        let amount = u64::from_le_bytes(amount_bytes);
        let decimals = data[8];
        let bump = data[9];

        // Signer must sign
        if !signer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Verify PDA derivation
        let expected_program_signer =
            pubkey::create_program_address(
                &[signer.key().as_ref(), &[bump]], &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?;

        if !program_signer.key().eq(&expected_program_signer) {
            return Err(ProgramError::InvalidSeeds);
        }

        Ok(WithdrawToken {
            signer,
            program_signer,
            mint,
            program_signer_ata,
            signer_ata,
            amount,
            decimals,
            bump,
        })
    }
}

impl<'a> WithdrawToken<'a> {
    #[inline(always)]
    pub fn process(&self) -> ProgramResult {
        // Seeds for PDA signer
        let seeds: [&[u8]; 2] = [self.signer.key().as_ref(), &[self.bump]];
        let pin_seeds: [Seed; 2] = seeds.map(Seed::from);
        let signer_meta = Signer::from(pin_seeds.as_ref());

        // Perform SPL Token transfer with decimals check
        TransferChecked {
            from: self.program_signer_ata,
            mint: self.mint,
            to: self.signer_ata,
            authority: self.program_signer,
            amount: self.amount,
            decimals: self.decimals,
        }
        .invoke_signed(&[signer_meta])
    }
}
