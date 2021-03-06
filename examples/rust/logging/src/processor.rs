//! Program instruction processor

use gemachain_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    log::{gema_log_compute_units, gema_log_params, gema_log_slice},
    msg,
    pubkey::Pubkey,
};

/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Log a string
    msg!("static string");

    // Log 5 numbers as u64s in hexadecimal format
    msg!(
        instruction_data[0],
        instruction_data[1],
        instruction_data[2],
        instruction_data[3],
        instruction_data[4]
    );

    // Log a slice
    gema_log_slice(instruction_data);

    // Log a formatted message, use with caution can be expensive
    msg!("formatted {}: {:?}", "message", instruction_data);

    // Log a public key
    program_id.log();

    // Log all the program's input parameters
    gema_log_params(accounts, instruction_data);

    // Log the number of compute units remaining that the program can consume.
    gema_log_compute_units();

    Ok(())
}
