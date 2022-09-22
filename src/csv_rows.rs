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