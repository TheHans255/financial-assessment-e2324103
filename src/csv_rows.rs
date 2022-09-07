use crate::transaction::*;
use crate::account::*;
use serde::{ Deserialize, Serialize };

/// Structure representing a raw input row. This could turn
/// into either a transaction or a dispute action
#[derive(Clone, Deserialize)]
pub struct InputRow {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

pub enum InputRowParseErr {
    UnknownType
}

impl TryFrom<InputRow> for Transaction {
    type Error = InputRowParseErr;
    fn try_from(row: InputRow) -> Result<Transaction, InputRowParseErr> {
        Ok(Transaction {
            id: row.tx,
            client_id: row.client,
            amount: match row.amount {
                Some(result) => (result * 10000.0) as u64,
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
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub frozen: bool,
}

impl From<Account> for OutputRow {
    fn from(account: Account) -> OutputRow {
        OutputRow {
            client: account.id,
            available: (account.available_balance as f64) / 10000.0,
            held: (account.held_balance as f64) / 10000.0,
            total: (account.available_balance as f64 + account.held_balance as f64) / 10000.0,
            frozen: account.is_frozen,
        }
    }
}