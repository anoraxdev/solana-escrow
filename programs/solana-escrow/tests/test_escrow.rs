use anchor_lang::{
    AccountDeserialize,
    InstructionData,
    ToAccountMetas,
    prelude::Pubkey,
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

#[test]
fn test_initialize_and_deposit_sol() {
    // 1. Initialize LiteSVM and load our compiled program.
    let mut svm = LiteSVM::new();

    let program_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/deploy/solana_escrow.so"
    );

    svm.add_program_from_file(
        ID,
        program_path,
    )
    .expect("Failed to load escrow program");

    // 2. Create and fund the seller.
    let seller = Keypair::new();

    svm.airdrop(
        &seller.pubkey(),
        10_000_000_000,
    )
    .expect("Failed to fund seller");

    // 3. Define the offer.
    let offer_id: u64 = 1;
    let sol_amount: u64 = 2_000_000_000; // 2 SOL
    let usdc_amount: u64 = 300_000_000;  // 300 USDC (6 decimals)

    // A placeholder mint for now.
    let usdc_mint = Pubkey::new_unique();

    // 4. Derive the escrow PDA.
    let (escrow_pda, _) = Pubkey::find_program_address(
        &[
            b"escrow",
            seller.pubkey().as_ref(),
            &offer_id.to_le_bytes(),
        ],
        &ID,
    );

    // 5. Construct the initialize instruction.
    let initialize_ix = Instruction {
        program_id: ID,
        accounts: accounts::Initialize {
            seller: seller.pubkey(),
            escrow: escrow_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),

        data: instruction::Initialize {
            offer_id,
            sol_amount,
            usdc_amount,
            usdc_mint,
        }
        .data(),
    };

    // 6. Submit the initialize transaction.
    let initialize_message = Message::new(
        &[initialize_ix],
        Some(&seller.pubkey()),
    );

    let initialize_tx = Transaction::new(
        &[&seller],
        initialize_message,
        svm.latest_blockhash(),
    );

    svm.send_transaction(initialize_tx)
        .expect("Initialize transaction failed");

    // 7. Read and verify the escrow account.
    let escrow_account = svm
        .get_account(&escrow_pda)
        .expect("Escrow account not found");

    let mut account_data = escrow_account.data.as_slice();

    let escrow = Escrow::try_deserialize(&mut account_data)
        .expect("Failed to deserialize escrow");

    assert_eq!(escrow.seller, seller.pubkey());
    assert_eq!(escrow.buyer, None);
    assert_eq!(escrow.usdc_mint, usdc_mint);
    assert_eq!(escrow.offer_id, offer_id);
    assert_eq!(escrow.sol_amount, sol_amount);
    assert_eq!(escrow.usdc_amount, usdc_amount);
    assert_eq!(escrow.status, 0);

    println!("Escrow initialized successfully");

    // 8. Record the escrow balance before depositing.
    let balance_before = escrow_account.lamports;

    // 9. Construct the deposit_sol instruction.
    let deposit_ix = Instruction {
        program_id: ID,
        accounts: accounts::DepositSol {
            seller: seller.pubkey(),
            escrow: escrow_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),

        data: instruction::DepositSol {}.data(),
    };

    // 10. Submit the deposit transaction.
    let deposit_message = Message::new(
        &[deposit_ix],
        Some(&seller.pubkey()),
    );

    let deposit_tx = Transaction::new(
        &[&seller],
        deposit_message,
        svm.latest_blockhash(),
    );

    svm.send_transaction(deposit_tx)
        .expect("Deposit transaction failed");

    // 11. Verify the resulting escrow balance.
    let escrow_account_after = svm
        .get_account(&escrow_pda)
        .expect("Escrow account not found after deposit");

    let balance_after = escrow_account_after.lamports;

    assert_eq!(
        balance_after - balance_before,
        sol_amount,
        "Escrow did not receive the expected SOL amount"
    );

    // 12. Verify the updated escrow state.
    let mut account_data_after = escrow_account_after.data.as_slice();

    let escrow_after = Escrow::try_deserialize(&mut account_data_after)
        .expect("Failed to deserialize escrow after deposit");

    assert_eq!(escrow_after.status, 1);

    println!("SOL deposited successfully");
    println!("Escrow balance increased by {} lamports", sol_amount);
}