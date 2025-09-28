use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey,
    ProgramResult, instruction::{Seed, Signer},
};
use pinocchio_token::instructions::TransferChecked;

/// Stores the state for the withdraw token instruction
pub struct WithdrawToken<'a> {
    pub signer: &'a AccountInfo,
    pub program_signer: &'a AccountInfo,
    pub mint: &'a AccountInfo,
    pub program_signer_ata: &'a AccountInfo,
    pub signer_ata: &'a AccountInfo,
    pub amount: u64,
    pub decimals: u8,
    pub bump: [u8; 1],
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for WithdrawToken<'a> {
    type Error = ProgramError;
    /// Extract the accounts and instruction data for token withdrawal
    /// 
    /// Signer account:- This is the account that owns the tokens in the corresponding program account
    /// 
    /// Program signer account:- This is the PDA account that stores the tokens
    /// 
    /// Mint account:- The token mint account for the SPL token being withdrawn
    /// 
    /// Program signer ATA:- The associated token account owned by the program signer PDA
    /// 
    /// Signer ATA:- The associated token account owned by the signer to receive tokens
    /// 
    fn try_from(value: (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        // Destructure accounts
        // Signer account -- Program signer account -- Mint account -- Program signer ATA -- Signer ATA
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

        // Extract data
        let amount = u64::from_le_bytes(data[..core::mem::size_of::<u64>()].try_into().unwrap());
        let decimals = data[8];
        let bump = data[9];

        // Signer must sign
        if !signer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Verify PDA derivation
        let expected_program_signer =
            pubkey::create_program_address(&[signer.key().as_ref(), &[bump]], 
                &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?;

        if program_signer.key().ne(&expected_program_signer) {
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
            bump: [bump],
        })
    }
}

impl<'a> WithdrawToken<'a> {
    /// Transfer the tokens back to the owner with decimal validation
    pub fn process(&self) -> ProgramResult {
        // Seeds for PDA signer
        let seeds: [Seed; 2] = [Seed::from(self.signer.key().as_ref()), 
            Seed::from(&self.bump)];
        let signer = Signer::from(seeds.as_ref());

        // Perform SPL Token transfer with decimals check
        TransferChecked {
            from: self.program_signer_ata,
            mint: self.mint,
            to: self.signer_ata,
            authority: self.program_signer,
            amount: self.amount,
            decimals: self.decimals,
        }
        .invoke_signed(&[signer])
    }
}