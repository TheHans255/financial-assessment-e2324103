use crate::transaction::*;
use crate::account::*;
use bigdecimal::BigDecimal;
use num_traits::Zero;
use serde::{ Deserialize, Serialize };

/// Structure representing a raw input row. This could turn
/// into either a transaction or a dispute action
#[derive(Clone, Deserialize)]
pub struct InputRow {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<BigDecimal>,
}

#[derive(Debug)]
/// Simple enum type for parse errors
pub enum InputRowParseErr {
    UnknownType,
    BadAmount
}

impl TryFrom<InputRow> for Transaction {
    type Error = InputRowParseErr;
    /// Convert from an input row to a Transaction (withdrawal or deposit).
    /// The conversion will fail if the amount is negative or if the
    /// row represents a dispute action
    fn try_from(row: InputRow) -> Result<Transaction, InputRowParseErr> {
        Ok(Transaction {
            id: row.tx,
            client_id: row.client,
            amount: match row.amount {
                Some(result) => {
                    if result < BigDecimal::new(Zero::zero(), 0) { return Err(InputRowParseErr::BadAmount); }
                    result.round(4)
                },
                None => return Err(InputRowParseErr::UnknownType)
            },
            transaction_type: match row.transaction_type.as_str() {
                "deposit" => TransactionType::Deposit,
                "withdrawal" => TransactionType::Withdrawal,
                _ => return Err(InputRowParseErr::UnknownType)
            },
            dispute_state: DisputeState::Undisputed
        })
    }
}

impl TryFrom<InputRow> for DisputeAction {
    type Error = InputRowParseErr;
    /// Convert from an input row to a Dispute action (dispute, resolve, or
    /// chargeback). The conversion will fail if the row represents a
    /// transaction.
    fn try_from(row: InputRow) -> Result<DisputeAction, InputRowParseErr> {
        Ok(DisputeAction {
            transaction_id: row.tx,
            client_id: row.client,
            action_type: match row.transaction_type.as_str() {
                "dispute" => DisputeActionType::Dispute,
                "resolve" => DisputeActionType::Resolve,
                "chargeback" => DisputeActionType::Chargeback,
                _ => return Err(InputRowParseErr::UnknownType)
            }
        })
    }
}

/// A structure representing an output row.
/// This is always derived from an account
#[derive(Clone, Serialize)]
pub struct OutputRow {
    pub client: u16,
    pub available: BigDecimal,
    pub held: BigDecimal,
    pub total: BigDecimal,
    pub locked: bool,
}

impl From<Account> for OutputRow {
    /// Convert the account state to an output row
    fn from(account: Account) -> OutputRow {
        OutputRow {
            client: account.id,
            total: &account.available_balance + &account.held_balance,
            available: account.available_balance,
            held: account.held_balance,
            locked: account.is_frozen,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_row_converts_to_transaction() {
        let input_row = InputRow {
            transaction_type: "deposit".to_string(),
            client: 1,
            tx: 1,
            amount: Some(12.into()),
        };
        let transaction: Transaction = input_row.try_into().expect("Parse failed");
        assert_eq!(transaction.transaction_type, TransactionType::Deposit);
        assert_eq!(transaction.client_id, 1);
        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.amount, 12.into());
        assert_eq!(transaction.dispute_state, DisputeState::Undisputed);
    }

    #[test]
    fn dispute_row_converts_to_dispute() {
        let input_row = InputRow {
            transaction_type: "dispute".to_string(),
            client: 1,
            tx: 1,
            amount: None,
        };
        let dispute_action: DisputeAction = input_row.try_into().expect("Parse failed");
        assert_eq!(dispute_action.action_type, DisputeActionType::Dispute);
        assert_eq!(dispute_action.client_id, 1);
        assert_eq!(dispute_action.transaction_id, 1);
    }

    #[test]
    fn transaction_row_does_not_convert_to_dispute() {
        let input_row = InputRow {
            transaction_type: "deposit".to_string(),
            client: 1,
            tx: 1,
            amount: Some(12.into()),
        };
        let dispute_result: Result<DisputeAction, InputRowParseErr> = input_row.try_into();
        dispute_result.expect_err("Parse from transaction into dispute was allowed");
    }

    #[test]
    fn dispute_row_does_not_convert_to_transaction() {
        let input_row = InputRow {
            transaction_type: "dispute".to_string(),
            client: 1,
            tx: 1,
            amount: None,
        };
        let transaction_result: Result<Transaction, InputRowParseErr> = input_row.try_into();
        transaction_result.expect_err("Parse from dispute into transaction was allowed");
    }

    #[test]
    fn account_converts_to_output_row() {
        let account = Account {
            available_balance: 100.into(),
            held_balance: 10.into(),
            id: 1,
            is_frozen: false,
            transactions: std::collections::HashMap::new()
        };
        let output_row: OutputRow = account.into();
        assert_eq!(output_row.client, 1);
        assert_eq!(output_row.available, 100.into());
        assert_eq!(output_row.held, 10.into());
        assert_eq!(output_row.total, 110.into());
        assert_eq!(output_row.locked, false);
    }

    #[test]
    fn account_frozen_status_becomes_locked_entry() {
        let account = Account {
            available_balance: 100.into(),
            held_balance: 10.into(),
            id: 1,
            is_frozen: true,
            transactions: std::collections::HashMap::new()
        };
        let output_row: OutputRow = account.into();
        assert_eq!(output_row.client, 1);
        assert_eq!(output_row.locked, true);
    }
}