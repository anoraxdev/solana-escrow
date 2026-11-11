use anchor_lang::{
    prelude::Pubkey,
    AccountDeserialize,
    InstructionData,
    ToAccountMetas,
};

use litesvm::LiteSVM;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

use solana_escrow::{
    accounts,
    instruction,
    state::Escrow,
    ID,
};

struct TestContext {
    svm: LiteSVM,
    seller: Keypair,
    escrow_pda: Pubkey,
    usdc_mint: Pubkey,
    offer_id: u64,
    sol_amount: u64,
    usdc_amount: u64,
}

// Creates an isolated test environment.
fn setup() -> TestContext {
    let mut svm = LiteSVM::new();

    let program_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/deploy/solana_escrow.so"
    );

    svm.add_program_from_file(ID, program_path)
        .expect("Failed to load escrow program");

    let seller = Keypair::new();

    svm.airdrop(&seller.pubkey(), 10_000_000_000)
        .expect("Failed to fund seller");

    let offer_id:u64 = 1;
    let sol_amount = 2_000_000_000;
    let usdc_amount = 300_000_000;
    let usdc_mint = Pubkey::new_unique();

    let (escrow_pda, _) = Pubkey::find_program_address(
        &[
            b"escrow",
            seller.pubkey().as_ref(),
            &offer_id.to_le_bytes(),
        ],
        &ID,
    );

    TestContext {
        svm,
        seller,
        escrow_pda,
        usdc_mint,
        offer_id,
        sol_amount,
        usdc_amount,
    }
}

// Initializes the escrow offer.
fn initialize_offer(ctx: &mut TestContext) {
    let initialize_ix = Instruction {
        program_id: ID,
        accounts: accounts::Initialize {
            seller: ctx.seller.pubkey(),
            escrow: ctx.escrow_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),

        data: instruction::Initialize {
            offer_id: ctx.offer_id,
            sol_amount: ctx.sol_amount,
            usdc_amount: ctx.usdc_amount,
            usdc_mint: ctx.usdc_mint,
        }
        .data(),
    };

    let message = Message::new(
        &[initialize_ix],
        Some(&ctx.seller.pubkey()),
    );

    let transaction = Transaction::new(
        &[&ctx.seller],
        message,
        ctx.svm.latest_blockhash(),
    );

    ctx.svm
        .send_transaction(transaction)
        .expect("Initialize transaction failed");
}

// Constructs a deposit instruction.
fn create_deposit_instruction(
    seller: Pubkey,
    escrow_pda: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::DepositSol {
            seller,
            escrow: escrow_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),

        data: instruction::DepositSol {}.data(),
    }
}

// Executes a SOL deposit.
fn deposit_sol(ctx: &mut TestContext) {
    let deposit_ix = create_deposit_instruction(
        ctx.seller.pubkey(),
        ctx.escrow_pda,
    );

    let message = Message::new(
        &[deposit_ix],
        Some(&ctx.seller.pubkey()),
    );

    let transaction = Transaction::new(
        &[&ctx.seller],
        message,
        ctx.svm.latest_blockhash(),
    );

    ctx.svm
        .send_transaction(transaction)
        .expect("Deposit transaction failed");
}

// Reads and deserializes the escrow account.
fn read_escrow(ctx: &TestContext) -> Escrow {
    let account = ctx
        .svm
        .get_account(&ctx.escrow_pda)
        .expect("Escrow account not found");

    let mut data = account.data.as_slice();

    Escrow::try_deserialize(&mut data)
        .expect("Failed to deserialize escrow")
}

#[test]
fn test_initialize_and_deposit_sol() {
    let mut ctx = setup();

    initialize_offer(&mut ctx);

    let escrow = read_escrow(&ctx);

    assert_eq!(escrow.seller, ctx.seller.pubkey());
    assert_eq!(escrow.buyer, None);
    assert_eq!(escrow.usdc_mint, ctx.usdc_mint);
    assert_eq!(escrow.offer_id, ctx.offer_id);
    assert_eq!(escrow.sol_amount, ctx.sol_amount);
    assert_eq!(escrow.usdc_amount, ctx.usdc_amount);
    assert_eq!(escrow.status, 0);

    println!("Escrow initialized successfully");

    let balance_before = ctx
        .svm
        .get_account(&ctx.escrow_pda)
        .unwrap()
        .lamports;

    deposit_sol(&mut ctx);

    let balance_after = ctx
        .svm
        .get_account(&ctx.escrow_pda)
        .unwrap()
        .lamports;

    assert_eq!(
        balance_after - balance_before,
        ctx.sol_amount,
        "Escrow did not receive the expected SOL amount"
    );

    let escrow = read_escrow(&ctx);

    assert_eq!(escrow.status, 1);

    println!("SOL deposited successfully");
}

#[test]
fn test_unauthorized_deposit() {
    let mut ctx = setup();

    initialize_offer(&mut ctx);

    let balance_before = ctx
        .svm
        .get_account(&ctx.escrow_pda)
        .unwrap()
        .lamports;

    let attacker = Keypair::new();

    ctx.svm
        .airdrop(&attacker.pubkey(), 1_000_000_000)
        .expect("Failed to fund unauthorized wallet");

    let deposit_ix = create_deposit_instruction(
        attacker.pubkey(),
        ctx.escrow_pda,
    );

    let message = Message::new(
        &[deposit_ix],
        Some(&attacker.pubkey()),
    );

    let transaction = Transaction::new(
        &[&attacker],
        message,
        ctx.svm.latest_blockhash(),
    );

    assert!(
        ctx.svm.send_transaction(transaction).is_err(),
        "Unauthorized deposit should fail"
    );

    println!("Unauthorized deposit correctly rejected");

    let escrow_account = ctx
        .svm
        .get_account(&ctx.escrow_pda)
        .expect("Escrow account not found");

    assert_eq!(
        escrow_account.lamports,
        balance_before,
        "Escrow balance changed after an unauthorized deposit"
    );
}

#[test]
fn test_duplicate_deposit() {
    let mut ctx = setup();

    initialize_offer(&mut ctx);
    deposit_sol(&mut ctx);

    let balance_before = ctx
        .svm
        .get_account(&ctx.escrow_pda)
        .unwrap()
        .lamports;

    let deposit_ix = create_deposit_instruction(
        ctx.seller.pubkey(),
        ctx.escrow_pda,
    );

    let message = Message::new(
        &[deposit_ix],
        Some(&ctx.seller.pubkey()),
    );

    let transaction = Transaction::new(
        &[&ctx.seller],
        message,
        ctx.svm.latest_blockhash(),
    );

    assert!(
        ctx.svm.send_transaction(transaction).is_err(),
        "Duplicate deposit should fail"
    );

    println!("Duplicate deposit correctly rejected");

    let balance_after = ctx
        .svm
        .get_account(&ctx.escrow_pda)
        .unwrap()
        .lamports;

    assert_eq!(
        balance_after,
        balance_before,
        "Escrow balance changed after a failed deposit"
    );

    let escrow = read_escrow(&ctx);

    assert_eq!(
        escrow.status,
        1,
        "Escrow status changed after a failed deposit"
    );
}