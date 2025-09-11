
mod tests{
use std::collections::HashMap;

use client::{
    ChangeCommit, 
    CloseCommit, 
    CommitCondition, 
    CreateCommit, 
    DataCommitType, 
    Entry
};
use solana_sdk::{
    account::Account, 
    instruction::Instruction, 
    program_option::COption, 
    pubkey::Pubkey, 
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::{
    state::{
        Account as TokenAccount
    },
    instruction::{
        transfer as token_transfer
    }
};
use mollusk_svm::{
    account_store::AccountStore, Mollusk, 
    program::{
        keyed_account_for_system_program
    }
};
use mollusk_svm_programs_token::token::{
        add_program as add_token_program, create_account_for_token_account, 
        keyed_account as add_token_program_keyed_account
    };
// Token transfer schema
// Source account
// Destination account
// Authority account
// Token program

#[test]
fn test_1(){
    let mut mollusk = Mollusk::default();
    let mut account_store = TransferAccountStore::new();

    let mut transfer_setup = TransferTest::new();

    transfer_setup.add_programs(&mut mollusk);

    transfer_setup.test_1_generate_initial_state(&mut account_store);

    println!("Initial state generated.\n\n");

    let create_commit_instruction = transfer_setup.
        test_1_create_commit(&mut account_store);

    let entry_instruction_1 = transfer_setup.test_1_entry();

    let change_commit_instruction = transfer_setup.test_1_change_commit();

    let entry_instruction_2 = transfer_setup.test_1_entry_2();

    let close_commit_instruction = transfer_setup.test_1_close_commit();

    let mut test_accounts = account_store.to_slice();

    println!("Beginning instruction processing.\n\n");

    let result = mollusk.process_instruction(
        &create_commit_instruction, &test_accounts);

    if result.raw_result.is_err(){
        
        println!("Create commit instruction failed:- {:?}", result.raw_result.err().unwrap());
        panic!("Create commit instruction failed:- {:?}", result.program_result);
    }

    test_accounts = result.resulting_accounts;

    println!("Create commit successful.\n\n");

    for pair in &test_accounts{
        if pair.0 == transfer_setup.commit{
            println!("Commit account:- {:?}", pair.1);
        }
    }

    
    // Entry 1
    let result = mollusk.process_instruction(
        &entry_instruction_1,
        &test_accounts,
    );
    if result.raw_result.is_err() {
        panic!(
            "Entry instruction 1 failed:- {:?}",
            result.raw_result.err().unwrap()
        );
    }
    test_accounts = result.resulting_accounts;
    
    println!("First entry successful.\n\n");
    
    
    // Change Commit
    let result = mollusk.process_instruction(
        &change_commit_instruction,
        &test_accounts,
    );
    if result.raw_result.is_err() {
        panic!(
            "Change commit instruction failed:- {:?}",
            result.raw_result.err().unwrap()
        );
    }
    test_accounts = result.resulting_accounts;
    
    println!("Change commit successful.\n\n");
    
    // Entry 2
    let result = mollusk.process_instruction(
        &entry_instruction_2,
        &test_accounts,
    );
    if result.raw_result.is_err() {
        panic!(
            "Entry instruction 2 failed:- {:?}",
            result.raw_result.err().unwrap()
        );
    }
    test_accounts = result.resulting_accounts;
    
    println!("Second entry successful.\n\n");
    
    // Close Commit
    let result = mollusk.process_instruction(
        &close_commit_instruction,
        &test_accounts,
    );
    if result.raw_result.is_err() {
        panic!(
            "Close commit instruction failed:- {:?}",
            result.raw_result.err().unwrap()
        );
    }
    
    println!("Close commit successful.\n\n");
    
    return;
}

#[test]
fn test_2(){
    // Create the accounts

    // Create the instruction

    // Modify the instruction by

    // Create the mollusk validator

    // Add in the wrapper program to the mollusk validator

    // Create the accounts for the mollusk validator

}
#[derive(Debug)]
pub struct TransferTest{
    pub source:Pubkey,
    pub program_source:Pubkey,
    pub destination:Pubkey,
    pub authority:Pubkey,
    pub program_authority:Pubkey,
    pub other_authority:Pubkey,
    pub mint:Pubkey,
    pub starter:Pubkey,
    pub commit:Pubkey
}

impl TransferTest{
    const TOKEN_PROGRAM:Pubkey = spl_token::ID;
    const SYSTEM_PROGRAM:Pubkey = solana_sdk::system_program::ID;
    const WRAPPER_PROGRAM_ID:Pubkey =  client::PROGRAM_ID;

    pub fn new()->Self{
        let authority = Pubkey::new_unique();

        let program_authority = Pubkey::find_program_address(
            &[authority.as_ref()], &Self::WRAPPER_PROGRAM_ID).0;

        let other_authority = Pubkey::new_unique();

        let mint = Pubkey::new_unique();

        let source = get_associated_token_address(
            &authority,&mint);

        let program_source = get_associated_token_address(
            &program_authority,&mint);

        let destination = get_associated_token_address(
            &other_authority,&mint);

        let starter = Pubkey::new_unique();

        TransferTest{
                authority,
                program_authority,
                source,
                program_source,
                destination,
                other_authority,
                mint,
                starter,
                commit:Pubkey::default()
        }
    }

    pub fn generate_keys(&mut self){
        self.authority = Pubkey::new_unique();

        self.program_authority = Pubkey::find_program_address(
            &[self.authority.as_ref()], &Self::WRAPPER_PROGRAM_ID).0;

        self.other_authority = Pubkey::new_unique();

        self.mint = Pubkey::new_unique();

        self.source = Pubkey::new_unique();

        self.program_source = get_associated_token_address(
            &self.source,&self.mint);

        self.destination = get_associated_token_address(
            &self.other_authority,&self.mint);
            
        self.starter = Pubkey::new_unique();
    }

    pub fn test_1_generate_initial_state(&self, store:&mut TransferAccountStore){
        let source_account = TokenAccount{
            mint:self.mint,
            owner:self.program_authority,
            amount:1_000_000_000,
            delegate:COption::None,
            state:spl_token::state::AccountState::Initialized,
            is_native:COption::None,
            delegated_amount:0,
            close_authority:COption::None
        };

        let destination_account = TokenAccount{
            mint:self.mint,
            owner:self.other_authority,
            amount:1_000,
            delegate:COption::None,
            state:spl_token::state::AccountState::Initialized,
            is_native:COption::None,
            delegated_amount:0,
            close_authority:COption::None
        };

        let authority = Account::new(
            1_000_000_000, 
            0, 
            &Self::SYSTEM_PROGRAM
        );

        let starter = Account::new(
            1_000_000_000, 
            0, 
            &Self::SYSTEM_PROGRAM
        );

        let token_program = add_token_program_keyed_account();

        let system_program = keyed_account_for_system_program();

        store.clear();

        store.store_account(self.program_source,
            create_account_for_token_account(source_account));

        store.store_account(self.destination, 
            create_account_for_token_account(destination_account));

        store.store_account(self.authority, authority);

        store.store_account(token_program.0, token_program.1);

        store.store_account(system_program.0, system_program.1);

        store.store_account(self.starter, starter);
    }

    pub fn test_1_create_commit(&mut self, store:&mut TransferAccountStore)->Instruction{
        let commit_condition = CommitCondition::Count(25);
        let non_signer_commit_keys = vec![
            self.destination,
            self.program_source,
        ];
        let signer_commit_keys = vec![
            self.authority,
        ];
        let account_count = 3;
        let account_indices = vec![1u8,0,2];
        let _instruction_data_offset = 0;
        let _instruction_data = vec![0u8];
        let starter_account_key = Some(self.starter);
        let data_commit_type = DataCommitType::AnyData;

        let mut create_commit = CreateCommit::new(
            Self::TOKEN_PROGRAM, 
            signer_commit_keys, 
            non_signer_commit_keys, 
            starter_account_key,
            account_count, 
            account_indices, 
            data_commit_type, 
            commit_condition
        ).expect("Could not create commit condition in test 1\n\n");
        
        let instruction = create_commit.to_instruction().
            expect("Error while creating create commit condition instruction");

        for account_meta in instruction.accounts.iter(){
            store.insert_if_not_inserted(
                account_meta.pubkey, 
                Account::new(0, 0,&Self::SYSTEM_PROGRAM)
            );
        }

        self.commit = instruction.accounts.last().unwrap().pubkey;

        println!("TransferTest Keys:");
        println!("  source          = {:?}", self.source.as_array());
        println!("  program source          = {:?}", self.program_source.as_array());        
        println!("  destination     = {:?}", self.destination.as_array());
        println!("  authority       = {:?}", self.authority.as_array());
        println!("  program authority       = {:?}", self.program_authority.as_array());
        println!("  other_authority = {:?}", self.other_authority.as_array());
        println!("  mint            = {:?}", self.mint.as_array());
        println!("  starter         = {:?}", self.starter.as_array());
        println!("  commit key      = {:?}", self.commit.as_array());
        println!("  token program      = {:?}", Self::TOKEN_PROGRAM.as_array());

        for account_meta in instruction.accounts.iter(){
            println!("Pubkey: {:?}", account_meta.pubkey);
            println!("Is signer: {:?}", account_meta.is_signer);
            println!("Is writable: {:?}", account_meta.is_writable);
            println!("\n\n");
        }

        instruction
    }

    pub fn test_1_entry(&self)->Instruction{
        let non_signer_commit_keys = vec![
            self.destination,
            self.program_source,
        ];
        let signer_commit_keys = vec![
            self.authority,
        ];
        let account_count = 3;
        let account_indices = vec![1u8,0,2];
        let _instruction_data_offset = 0;
        let _instruction_data = vec![0u8];
        let data_commit_type = DataCommitType::AnyData;

        let mut entry = Entry::new(
            Self::TOKEN_PROGRAM, 
            signer_commit_keys,
            non_signer_commit_keys, 
            self.starter, 
            account_count, account_indices, 
            data_commit_type
        ).expect("Could not create entry in test 1.\n\n");

        let transfer_instruction = token_transfer(
            &Self::TOKEN_PROGRAM, 
            &self.program_source,
            &self.destination, 
            &self.authority,
             &[], 
            1_000_000
        ).expect("Could not create transfer instruction");


        let modified_transaction_instruction = 
        entry.to_instruction(transfer_instruction).
            expect("Could not create entry instruction");

        modified_transaction_instruction
    }

    pub fn test_1_change_commit(&self)->Instruction{
        let commit_condition = CommitCondition::Count(10);

        let non_signer_commit_keys = vec![
            self.destination,
            self.program_source,
        ];
        let signer_commit_keys = vec![
            self.authority,
        ];
        let account_count = 3;
        let account_indices = vec![1u8,0,2];
        let _instruction_data_offset = 0;
        let _instruction_data = vec![0u8];
        let data_commit_type = DataCommitType::AnyData;

        let mut create_commit = ChangeCommit::new(
            Self::TOKEN_PROGRAM, 
            signer_commit_keys, 
            non_signer_commit_keys, 
            account_count, 
            account_indices, 
            data_commit_type, 
            commit_condition
        ).expect("Could not create change condition in test 1\n\n");
        
        let instruction = create_commit.to_instruction().
            expect("Error while creating create commit condition instruction");

        println!("TransferTest Keys:");
        println!("  source          = {}", self.source);
        println!("  destination     = {}", self.destination);
        println!("  authority       = {}", self.authority);
        println!("  other_authority = {}", self.other_authority);
        println!("  mint            = {}", self.mint);
        println!("  starter         = {}", self.starter);

        for account_meta in instruction.accounts.iter(){
            println!("Pubkey: {:?}", account_meta.pubkey);
            println!("Is signer: {:?}", account_meta.is_signer);
            println!("Is writable: {:?}", account_meta.is_writable);
            println!("\n\n");
        }

        instruction
    }

    pub fn test_1_entry_2(&self)->Instruction{
        let non_signer_commit_keys = vec![
            self.destination,
            self.program_source,
        ];
        let signer_commit_keys = vec![
            self.authority,
        ];
        let account_count = 3;
        let account_indices = vec![1u8,0,2];
        let _instruction_data_offset = 0;
        let _instruction_data = vec![0u8];
        let data_commit_type = DataCommitType::AnyData;

        let mut entry = Entry::new(
            Self::TOKEN_PROGRAM, 
            signer_commit_keys,
            non_signer_commit_keys, 
            self.starter, 
            account_count, account_indices, 
            data_commit_type
        ).expect("Could not create entry in test 1.\n\n");

        let transfer_instruction = token_transfer(
            &Self::TOKEN_PROGRAM, 
            &self.program_source, 
            &self.destination, 
            &self.authority,
             &[], 
            1_000_000
        ).expect("Could not create transfer instruction");


        let modified_transaction_instruction = 
        entry.to_instruction(transfer_instruction).
            expect("Could not create entry instruction");

        modified_transaction_instruction
    }

    pub fn test_1_close_commit(&self)->Instruction{
        let non_signer_commit_keys = vec![
            self.destination,
            self.program_source,
        ];
        let signer_commit_keys = vec![
            self.authority,
        ];
        let account_count = 3;
        let account_indices = vec![1u8,0,2];
        let _instruction_data_offset = 0;
        let _instruction_data = vec![0u8];
        let data_commit_type = DataCommitType::AnyData;

        let mut close_commit = CloseCommit::new(
            Self::TOKEN_PROGRAM, 
            signer_commit_keys,
            non_signer_commit_keys, 
            self.starter, 
            account_count, account_indices, 
            data_commit_type
        ).expect("Could not create close commit in test 1./n/n");

        close_commit.to_instruction().expect("Error while creating change commit instruction")
    }

    pub fn add_programs(&self, validator:&mut Mollusk){
        validator.add_program(
            &Self::WRAPPER_PROGRAM_ID, 
            "program", &solana_sdk::bpf_loader_upgradeable::ID);

        add_token_program(validator);


    }
}

pub struct TransferAccountStore{
    map:HashMap<Pubkey, Account>
}

impl AccountStore for TransferAccountStore{
    #[inline(always)]
    fn get_account(&self, pubkey: &Pubkey) -> Option<Account> {
        self.map.get(pubkey).cloned()
    }

    #[inline(always)]
    fn store_account(&mut self, pubkey: Pubkey, account: Account) {
        self.map.insert(pubkey, account);
    }

    #[inline(always)]
    fn default_account(&self, _pubkey: &Pubkey) -> Account {
        Account::new(0, 0, &solana_sdk::system_program::ID)
    }
}

impl TransferAccountStore{
    #[inline]
    pub fn insert_if_not_inserted(&mut self, pubkey:Pubkey, account:Account){
        if self.map.get(&pubkey).is_none(){
            self.map.insert(pubkey, account);
        }
    }

    #[inline]
    pub fn to_slice(&self)->Vec<(Pubkey,Account)>{
        let mut result:Vec<(Pubkey, Account)> = Vec::with_capacity(self.map.len());

        for pair in self.map.iter(){
            result.push((pair.0.clone(), pair.1.clone()));
        }

        result
    }

    #[inline]
    pub fn add_slice(&mut self, slice:Vec<(Pubkey, Account)>){
        for pair in slice.iter(){
            self.store_account(pair.0, pair.1.clone());
        }
    }

    #[inline(always)]
    pub fn from_slice(&mut self, slice:Vec<(Pubkey, Account)>){
        self.map.clear();
        self.add_slice(slice);
    }

    #[inline(always)]
    pub fn clear(&mut self){
        self.map.clear();
    }

    #[inline(always)]
    pub fn new()->Self{
        Self{
            map:HashMap::default()
        }
    }
}


}