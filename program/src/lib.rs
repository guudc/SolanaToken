use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::{
    instruction::{initialize_account, initialize_mint, mint_to, transfer},
    state::{Account, Mint},
};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Parse accounts
    let mint_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let initializer = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;
    let rent_sysvar_account = next_account_info(accounts_iter)?;

    // Ensure the program is the correct program for the given token accounts
    if token_program.key != &spl_token::id() {
        msg!("Token program account is incorrect");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check if the initializer account is signing the transaction
    if !initializer.is_signer {
        msg!("Initializer account must sign the transaction");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check if the initializer account is rent exempt
    let rent = &Rent::from_account_info(rent_sysvar_account)?;
    if !rent.is_exempt(initializer.lamports(), initializer.data_len()) {
        msg!("Initializer account is not rent-exempt");
        return Err(ProgramError::AccountNotRentExempt);
    }

    // Validate instruction data
    if instruction_data.is_empty() {
        msg!("No instruction data provided");
        return Err(ProgramError::InvalidInstructionData);
    }

    match instruction_data[0] {
        // Initialize the token mint
        0 => {
            initialize_mint(
                token_program.key,
                mint_account.key,
                mint_authority.key,
                None,
                0,
            )?;
        }
        // Initialize a token account
        1 => {
            let owner_pubkey = owner.key;
            let token_account_pubkey = accounts_iter
                .next()
                .ok_or_else(|| ProgramError::NotEnoughAccountKeys)?;

            initialize_account(
                token_program.key,
                token_account_pubkey.key,
                mint_account.key,
                owner_pubkey,
            )?;
        }
        // Mint new tokens to an account
        2 => {
            let amount = instruction_data[1..]
                .iter()
                .copied()
                .take(8)
                .fold(0u64, |acc, byte| (acc << 8) + u64::from(byte));

            let destination_account = accounts_iter
                .next()
                .ok_or_else(|| ProgramError::NotEnoughAccountKeys)?;

            mint_to(
                token_program.key,
                mint_account.key,
                destination_account.key,
                initializer.key,
                &[],
                amount,
            )?;
        }
        // Transfer tokens between accounts
        3 => {
            let source_account = accounts_iter
                .next()
                .ok_or_else(|| ProgramError::NotEnoughAccountKeys)?;

            let destination_account = accounts_iter
                .next()
                .ok_or_else(|| ProgramError::NotEnoughAccountKeys)?;

            let amount = instruction_data[1..]
                .iter()
                .copied()
                .take(8)
                .fold(0u64, |acc, byte| (acc << 8) + u64::from(byte));

            transfer(
                token_program.key,
                source_account.key,
                destination_account.key,
                initializer.key,
                &[],
                amount,
            )?;
        }
        _ => {
            msg!("Invalid instruction");
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    Ok(())
}
