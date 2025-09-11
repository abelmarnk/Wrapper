Note:-
The instruction type is always padded behind the instruction data

Invoke:-
Checks that the configuration was created by the user and matches the specifics of the current instruction.

Expected Accounts:-
    Program account
    Program signer accounts
    Other commitment accounts
    Other accounts
Expected Data(Ordered):-
    Program Data..[byte range of instruction data, account keys account bumps, accounts indices, signers count, accounts count]

    Check the commit condition

    Checking the configuration commit:-
        Check the program id
        The order of accounts
        The order of signers
        The account data
        The account's public keys
    
    Extracting the signers and their seeds:-
        Extract the count from the instruction data
        Extract the accounts with the indices provided in the instruction data(It is the last set of accounts)
        Extract the seeds from the instruction data
        Construct the seeds for signing
    
    Build the instruction:-
        Loop the accounts and construct the appropriate account metas
        Add the program id
        Add the data after cutting it

    Invoke the instruction

Create:-
Creates a commitment of a particular configuration, signers of the instruction are set and commited to as well and for each account that would be signing, the user would have to provide the signature for each of those accounts.

Expected Accounts(Ordered):-
Program account
Configuration account
Signers
Program signers
Other commit accounts

Expected Data(Ordered):-

[Commit condition, byte range contents, length of byte range, Positions of commit accounts, signers count, commitment accounts count(Not needed can be derived)]

    Extract and check the program account:-
        Check if the program account is executable
    
    Extract the program signers and signers and verify signatures
        Check if each program signer exists and is owned by the program 
        Check if the program signer was derived from the corresponding signer
        Check if each signer signed the transaction

    Build the seeds for the configuration account:-
        Extract the program id
        Extract the signers positions
        Extract the commit accounts positions
        Extract the byte contents
        Extract the public keys of the program signers
        Construct the seeds with it

    Confirm the derivation of the configuration account

    Create the configuration account

    Serialize the commit condition into the configuration account

Deposit
// Not needed

Withdraw-native:-
Withdraws native from account.

Expected Accounts(Ordered):-

Signer
Program signer

Expected data:-
amount

    Check the owner, signers and derivation

    Transfer the lamports

Withdraw-token:-
Withdraws spl-token or token2022 tokens from account.

Expected Accounts(Ordered):-

Signer
Program signer
Mint
Program signer ATA
Program


Expected data:-
amount(8 bytes)

    Check the owner, signers and derivation

    Transfer the tokens


Commitment:Enum:-
Count(u32)
BeforeTimestamp(u64)
AfterTimestamp(u64)
CountBeforeTimestamp(u32, u64)
CountAfterTimestamp(u32, u64)
//Add count within range

