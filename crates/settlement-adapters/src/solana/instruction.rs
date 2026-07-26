//! Construcción de instrucción Anchor `process_payment`.

use borsh::BorshSerialize;
use sha2::{Digest, Sha256};
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TOKEN_PROGRAM_ID;

pub const SETTLEMENT_SEED: &[u8] = b"settlement";

/// Cuentas requeridas por `ProcessPayment` (orden Anchor).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessPaymentAccounts {
    pub payer: Pubkey,
    pub payer_token_account: Pubkey,
    pub merchant_token_account: Pubkey,
    pub merchant: Pubkey,
    pub settlement_state: Pubkey,
}

/// Deriva cuentas SPL y PDA para `process_payment`.
pub fn derive_process_payment_accounts(
    program_id: &Pubkey,
    payer: &Pubkey,
    merchant: &Pubkey,
    token_mint: &Pubkey,
) -> ProcessPaymentAccounts {
    let (settlement_state, _bump) =
        Pubkey::find_program_address(&[SETTLEMENT_SEED, merchant.as_ref()], program_id);

    ProcessPaymentAccounts {
        payer: *payer,
        payer_token_account: get_associated_token_address(payer, token_mint),
        merchant_token_account: get_associated_token_address(merchant, token_mint),
        merchant: *merchant,
        settlement_state,
    }
}

/// Construye la instrucción `process_payment` del programa Anchor.
pub fn build_process_payment_instruction(
    program_id: &Pubkey,
    accounts: &ProcessPaymentAccounts,
    amount: u64,
    brand_code: u8,
    settlement_rail_id: u64,
) -> Instruction {
    let mut data = process_payment_discriminator().to_vec();
    let args = ProcessPaymentArgs {
        amount,
        brand_code,
        settlement_rail_id,
    };
    args.serialize(&mut data)
        .expect("ProcessPaymentArgs debe serializarse");

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(accounts.payer, true),
            AccountMeta::new(accounts.payer_token_account, false),
            AccountMeta::new(accounts.merchant_token_account, false),
            AccountMeta::new_readonly(accounts.merchant, false),
            AccountMeta::new(accounts.settlement_state, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data,
    }
}

#[derive(BorshSerialize)]
struct ProcessPaymentArgs {
    amount: u64,
    brand_code: u8,
    settlement_rail_id: u64,
}

/// Discriminador Anchor: primeros 8 bytes de `sha256("global:process_payment")`.
pub fn process_payment_discriminator() -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(b"global:process_payment");
    let hash = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn discriminator_is_eight_bytes() {
        assert_eq!(process_payment_discriminator().len(), 8);
    }

    #[test]
    fn instruction_includes_program_and_six_accounts() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let merchant = Pubkey::from_str("11111111111111111111111111111111").expect("pubkey");
        let mint = Pubkey::new_unique();

        let accounts = derive_process_payment_accounts(&program_id, &payer, &merchant, &mint);
        let ix = build_process_payment_instruction(
            &program_id,
            &accounts,
            250_000,
            1,
            3,
        );

        assert_eq!(ix.program_id, program_id);
        assert_eq!(ix.accounts.len(), 6);
        assert!(ix.data.len() > 8);
        assert_eq!(&ix.data[..8], process_payment_discriminator());
    }
}
