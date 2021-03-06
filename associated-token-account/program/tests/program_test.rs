use gemachain_program::pubkey::Pubkey;
use gemachain_program_test::ProgramTest;

use gemachain_program_test::*;
use gpl_associated_token_account::{id, processor::process_instruction};

pub fn program_test(token_mint_address: Pubkey, use_latest_gpl_token: bool) -> ProgramTest {
    let mut pc = ProgramTest::new(
        "gpl_associated_token_account",
        id(),
        processor!(process_instruction),
    );

    if use_latest_gpl_token {
        // TODO: Remove after Token >3.2.0 is released
        pc.add_program(
            "gpl_token",
            gpl_token::id(),
            processor!(gpl_token::processor::Processor::process),
        );
    }

    // Add a token mint account
    //
    // The account data was generated by running:
    //      $ gemachain account EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v \
    //                       --output-file tests/fixtures/token-mint-data.bin
    //
    pc.add_account_with_file_data(
        token_mint_address,
        1461600,
        gpl_token::id(),
        "token-mint-data.bin",
    );

    // Dial down the BPF compute budget to detect if the program gets bloated in the future
    pc.set_bpf_compute_max_units(50_000);

    pc
}
