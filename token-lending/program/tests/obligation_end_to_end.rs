#![cfg(feature = "test-bpf")]

mod helpers;

use helpers::*;
use gemachain_program_test::*;
use gemachain_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use gpl_token::{instruction::approve, gemachain_program::program_pack::Pack};
use gpl_token_lending::{
    instruction::{
        borrow_obligation_liquidity, deposit_obligation_collateral, init_obligation,
        refresh_obligation, refresh_reserve, repay_obligation_liquidity,
        withdraw_obligation_collateral,
    },
    math::Decimal,
    processor::process_instruction,
    state::{Obligation, INITIAL_COLLATERAL_RATIO},
};

#[tokio::test]
async fn test_success() {
    let mut test = ProgramTest::new(
        "gpl_token_lending",
        gpl_token_lending::id(),
        processor!(process_instruction),
    );

    // limit to track compute unit increase
    test.set_bpf_compute_max_units(170_000);

    const FEE_AMOUNT: u64 = 100;
    const HOST_FEE_AMOUNT: u64 = 20;

    const GEMA_DEPOSIT_AMOUNT_CARATS: u64 = 100 * CARATS_TO_GEMA * INITIAL_COLLATERAL_RATIO;
    const GEMA_RESERVE_COLLATERAL_CARATS: u64 = GEMA_DEPOSIT_AMOUNT_CARATS;

    const USDC_RESERVE_LIQUIDITY_FRACTIONAL: u64 = 1_000 * FRACTIONAL_TO_USDC;
    const USDC_BORROW_AMOUNT_FRACTIONAL: u64 = USDC_RESERVE_LIQUIDITY_FRACTIONAL - FEE_AMOUNT;
    const USDC_REPAY_AMOUNT_FRACTIONAL: u64 = USDC_RESERVE_LIQUIDITY_FRACTIONAL;

    let user_accounts_owner = Keypair::new();
    let user_accounts_owner_pubkey = user_accounts_owner.pubkey();

    let user_transfer_authority = Keypair::new();
    let user_transfer_authority_pubkey = user_transfer_authority.pubkey();

    let obligation_keypair = Keypair::new();
    let obligation_pubkey = obligation_keypair.pubkey();

    let lending_market = add_lending_market(&mut test);

    let mut reserve_config = TEST_RESERVE_CONFIG;
    reserve_config.loan_to_value_ratio = 50;

    let gema_oracle = add_gema_oracle(&mut test);
    let gema_test_reserve = add_reserve(
        &mut test,
        &lending_market,
        &gema_oracle,
        &user_accounts_owner,
        AddReserveArgs {
            user_liquidity_amount: GEMA_RESERVE_COLLATERAL_CARATS,
            liquidity_amount: GEMA_RESERVE_COLLATERAL_CARATS,
            liquidity_mint_pubkey: gpl_token::native_mint::id(),
            liquidity_mint_decimals: 9,
            config: reserve_config,
            ..AddReserveArgs::default()
        },
    );

    let usdc_mint = add_usdc_mint(&mut test);
    let usdc_oracle = add_usdc_oracle(&mut test);
    let usdc_test_reserve = add_reserve(
        &mut test,
        &lending_market,
        &usdc_oracle,
        &user_accounts_owner,
        AddReserveArgs {
            user_liquidity_amount: FEE_AMOUNT,
            liquidity_amount: USDC_RESERVE_LIQUIDITY_FRACTIONAL,
            liquidity_mint_pubkey: usdc_mint.pubkey,
            liquidity_mint_decimals: usdc_mint.decimals,
            config: reserve_config,
            ..AddReserveArgs::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = test.start().await;
    let payer_pubkey = payer.pubkey();

    let initial_collateral_supply_balance =
        get_token_balance(&mut banks_client, gema_test_reserve.collateral_supply_pubkey).await;
    let initial_user_collateral_balance =
        get_token_balance(&mut banks_client, gema_test_reserve.user_collateral_pubkey).await;
    let initial_liquidity_supply =
        get_token_balance(&mut banks_client, usdc_test_reserve.liquidity_supply_pubkey).await;
    let initial_user_liquidity_balance =
        get_token_balance(&mut banks_client, usdc_test_reserve.user_liquidity_pubkey).await;

    let rent = banks_client.get_rent().await.unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[
            // 0
            create_account(
                &payer.pubkey(),
                &obligation_keypair.pubkey(),
                rent.minimum_balance(Obligation::LEN),
                Obligation::LEN as u64,
                &gpl_token_lending::id(),
            ),
            // 1
            init_obligation(
                gpl_token_lending::id(),
                obligation_pubkey,
                lending_market.pubkey,
                user_accounts_owner_pubkey,
            ),
            // 2
            refresh_reserve(
                gpl_token_lending::id(),
                gema_test_reserve.pubkey,
                gema_oracle.price_pubkey,
            ),
            // 3
            approve(
                &gpl_token::id(),
                &gema_test_reserve.user_collateral_pubkey,
                &user_transfer_authority_pubkey,
                &user_accounts_owner_pubkey,
                &[],
                GEMA_DEPOSIT_AMOUNT_CARATS,
            )
            .unwrap(),
            // 4
            deposit_obligation_collateral(
                gpl_token_lending::id(),
                GEMA_DEPOSIT_AMOUNT_CARATS,
                gema_test_reserve.user_collateral_pubkey,
                gema_test_reserve.collateral_supply_pubkey,
                gema_test_reserve.pubkey,
                obligation_pubkey,
                lending_market.pubkey,
                user_accounts_owner_pubkey,
                user_transfer_authority_pubkey,
            ),
            // 5
            refresh_obligation(
                gpl_token_lending::id(),
                obligation_pubkey,
                vec![gema_test_reserve.pubkey],
            ),
            // 6
            refresh_reserve(
                gpl_token_lending::id(),
                usdc_test_reserve.pubkey,
                usdc_oracle.price_pubkey,
            ),
            // 7
            borrow_obligation_liquidity(
                gpl_token_lending::id(),
                USDC_BORROW_AMOUNT_FRACTIONAL,
                usdc_test_reserve.liquidity_supply_pubkey,
                usdc_test_reserve.user_liquidity_pubkey,
                usdc_test_reserve.pubkey,
                usdc_test_reserve.liquidity_fee_receiver_pubkey,
                obligation_pubkey,
                lending_market.pubkey,
                user_accounts_owner_pubkey,
                Some(usdc_test_reserve.liquidity_host_pubkey),
            ),
            // 8
            refresh_reserve(
                gpl_token_lending::id(),
                usdc_test_reserve.pubkey,
                usdc_oracle.price_pubkey,
            ),
            // 9
            refresh_obligation(
                gpl_token_lending::id(),
                obligation_pubkey,
                vec![gema_test_reserve.pubkey, usdc_test_reserve.pubkey],
            ),
            // 10
            approve(
                &gpl_token::id(),
                &usdc_test_reserve.user_liquidity_pubkey,
                &user_transfer_authority_pubkey,
                &user_accounts_owner_pubkey,
                &[],
                USDC_REPAY_AMOUNT_FRACTIONAL,
            )
            .unwrap(),
            // 11
            repay_obligation_liquidity(
                gpl_token_lending::id(),
                USDC_REPAY_AMOUNT_FRACTIONAL,
                usdc_test_reserve.user_liquidity_pubkey,
                usdc_test_reserve.liquidity_supply_pubkey,
                usdc_test_reserve.pubkey,
                obligation_pubkey,
                lending_market.pubkey,
                user_transfer_authority_pubkey,
            ),
            // 12
            refresh_obligation(
                gpl_token_lending::id(),
                obligation_pubkey,
                vec![gema_test_reserve.pubkey],
            ),
            // 13
            withdraw_obligation_collateral(
                gpl_token_lending::id(),
                GEMA_DEPOSIT_AMOUNT_CARATS,
                gema_test_reserve.collateral_supply_pubkey,
                gema_test_reserve.user_collateral_pubkey,
                gema_test_reserve.pubkey,
                obligation_pubkey,
                lending_market.pubkey,
                user_accounts_owner_pubkey,
            ),
        ],
        Some(&payer_pubkey),
    );

    transaction.sign(
        &vec![
            &payer,
            &obligation_keypair,
            &user_accounts_owner,
            &user_transfer_authority,
        ],
        recent_blockhash,
    );
    assert!(banks_client.process_transaction(transaction).await.is_ok());

    let usdc_reserve = usdc_test_reserve.get_state(&mut banks_client).await;

    let obligation = {
        let obligation_account: Account = banks_client
            .get_account(obligation_pubkey)
            .await
            .unwrap()
            .unwrap();
        Obligation::unpack(&obligation_account.data[..]).unwrap()
    };

    let collateral_supply_balance =
        get_token_balance(&mut banks_client, gema_test_reserve.collateral_supply_pubkey).await;
    let user_collateral_balance =
        get_token_balance(&mut banks_client, gema_test_reserve.user_collateral_pubkey).await;
    assert_eq!(collateral_supply_balance, initial_collateral_supply_balance);
    assert_eq!(user_collateral_balance, initial_user_collateral_balance);

    let liquidity_supply =
        get_token_balance(&mut banks_client, usdc_test_reserve.liquidity_supply_pubkey).await;
    let user_liquidity_balance =
        get_token_balance(&mut banks_client, usdc_test_reserve.user_liquidity_pubkey).await;
    assert_eq!(liquidity_supply, initial_liquidity_supply);
    assert_eq!(
        user_liquidity_balance,
        initial_user_liquidity_balance - FEE_AMOUNT
    );
    assert_eq!(usdc_reserve.liquidity.borrowed_amount_wads, Decimal::zero());
    assert_eq!(
        usdc_reserve.liquidity.available_amount,
        initial_liquidity_supply
    );

    assert_eq!(obligation.deposits.len(), 0);
    assert_eq!(obligation.borrows.len(), 0);

    let fee_balance = get_token_balance(
        &mut banks_client,
        usdc_test_reserve.liquidity_fee_receiver_pubkey,
    )
    .await;
    assert_eq!(fee_balance, FEE_AMOUNT - HOST_FEE_AMOUNT);

    let host_fee_balance =
        get_token_balance(&mut banks_client, usdc_test_reserve.liquidity_host_pubkey).await;
    assert_eq!(host_fee_balance, HOST_FEE_AMOUNT);
}
