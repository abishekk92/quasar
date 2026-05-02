extern crate std;
use {
    quasar_svm::{Account, Instruction, Pubkey, QuasarSvm},
    quasar_vault_client::*,
    std::{println, vec},
};

const USER: Pubkey = Pubkey::new_from_array([1; 32]);

fn setup() -> QuasarSvm {
    let elf = std::fs::read("../../target/deploy/quasar_vault.so").unwrap();
    QuasarSvm::new().with_program(&crate::ID, &elf)
}

fn signer(address: Pubkey) -> Account {
    quasar_svm::token::create_keyed_system_account(&address, 10_000_000_000)
}

fn empty(address: Pubkey) -> Account {
    Account {
        address,
        lamports: 0,
        data: vec![],
        owner: quasar_svm::system_program::ID,
        executable: false,
    }
}

#[test]
fn test_deposit() {
    let mut svm = setup();

    let user = USER;
    let system_program = quasar_svm::system_program::ID;
    let (vault, _) = Pubkey::find_program_address(&[b"vault", user.as_ref()], &crate::ID);

    let deposit_amount: u64 = 1_000_000_000;

    let instruction: Instruction = DepositInstruction {
        user,
        vault,
        system_program,
        amount: deposit_amount,
    }
    .into();

    let result = svm.process_instruction(&instruction, &[signer(user), empty(vault)]);

    assert!(result.is_ok(), "deposit failed: {:?}", result.raw_result);

    let user_after = result.account(&user).unwrap().lamports;
    let vault_after = result.account(&vault).unwrap().lamports;

    assert_eq!(
        user_after,
        10_000_000_000 - deposit_amount,
        "user lamports after deposit"
    );
    assert_eq!(vault_after, deposit_amount, "vault lamports after deposit");

    println!("  DEPOSIT CU: {}", result.compute_units_consumed);
}

#[test]
fn test_deposit_then_withdraw_round_trip() {
    // Regression test for the deposit→withdraw round-trip. Without the
    // PDA-signed system.transfer fix, this test would fail with
    // ExternalAccountLamportSpend on the withdraw step (vault is
    // system-owned post-deposit, set_lamports requires program ownership).
    let mut svm = setup();

    let user = USER;
    let system_program = quasar_svm::system_program::ID;
    let (vault, _) = Pubkey::find_program_address(&[b"vault", user.as_ref()], &crate::ID);

    let deposit_amount: u64 = 1_000_000_000;
    let withdraw_amount: u64 = 500_000_000;

    let deposit_ix: Instruction = DepositInstruction {
        user,
        vault,
        system_program,
        amount: deposit_amount,
    }
    .into();
    let after_deposit = svm.process_instruction(&deposit_ix, &[signer(user), empty(vault)]);
    assert!(
        after_deposit.is_ok(),
        "deposit failed: {:?}",
        after_deposit.raw_result
    );

    let withdraw_ix: Instruction = WithdrawInstruction {
        user,
        vault,
        system_program,
        amount: withdraw_amount,
    }
    .into();
    let after_withdraw = svm.process_instruction(
        &withdraw_ix,
        &[
            after_deposit.account(&user).unwrap().clone(),
            after_deposit.account(&vault).unwrap().clone(),
        ],
    );
    assert!(
        after_withdraw.is_ok(),
        "withdraw failed: {:?}",
        after_withdraw.raw_result
    );

    let user_final = after_withdraw.account(&user).unwrap().lamports;
    assert_eq!(
        user_final,
        10_000_000_000 - deposit_amount + withdraw_amount
    );
}
