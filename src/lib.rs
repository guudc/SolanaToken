use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Parse accounts
    let voter_account = next_account_info(accounts_iter)?;
    let vote_account = next_account_info(accounts_iter)?;
    let rent_sysvar_account = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    // Ensure the program is the correct program for the given vote account
    if vote_account.owner != program_id {
        msg!("Vote account has incorrect owner");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check if the voter account is signing the transaction
    if !voter_account.is_signer {
        msg!("Voter account must sign the transaction");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check if the voter account is rent exempt
    let rent = &Rent::from_account_info(rent_sysvar_account)?;
    if !rent.is_exempt(voter_account.lamports(), voter_account.data_len()) {
        msg!("Voter account is not rent-exempt");
        return Err(ProgramError::AccountNotRentExempt);
    }

    // Validate instruction data  
    // For simplicity, let's assume the instruction data is a single u8 representing the vote
    if instruction_data.len() != 1 {
        msg!("Invalid instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Perform the vote
    let vote_value = instruction_data[0];
    let mut vote_data = vote_account.try_borrow_mut_data()?;
    vote_data[0] = vote_value;

    // Transfer a small amount from the voter to the program (for demonstration purposes)
    let transfer_instruction = system_instruction::transfer(
        voter_account.key,
        system_program_account.key,
        1, // transfer 1 lamport
    );
    solana_program::program::invoke(
        &transfer_instruction,
        &[voter_account.clone(), system_program_account.clone()],
    )?;

    Ok(())
}
