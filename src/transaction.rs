/// The type of transaction being executed, either a deposit or withdrawal
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdrawal
}

/// The state of dispute a transaction is in
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DisputeState {
    /// The transaction has either never been disputed, or has been disputed or resolved
    Undisputed,
    /// The transaction is under dispute
    Disputed,
    /// The disputed transaction has been charged back to the account holder
    ChargedBack
}

/// A state transition for a transaction dispute
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DisputeActionType {
    /// Take an undisputed transaction into dispute
    Dispute,
    /// Cancel the dispute on a transaction
    Resolve,
    /// Charge a disputed transaction back to the account holder
    Chargeback
}

/// A structure representing a transaction
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// A globally unique transaction ID
    pub id: u32,
    /// The client ID of the account the transaction is acting on
    pub client_id: u16,
    /// The amount of the transaction in 1/10000 currency units
    /// (this is used instead of f64 to avoid rounding errors)
    pub amount: u64,
    /// Whether the transaction is a deposit or a withdrawal
    pub transaction_type: TransactionType,
    /// Whether a transaction is OK, under dispute, or charged back
    pub dispute_state: DisputeState
}

/// A structure representing a change in the dispute state for
/// a transaction
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DisputeAction {
    /// The desired action for the transaction
    pub action_type: DisputeActionType,
    /// The client ID of the account of concern
    pub client_id: u16,
    /// The transaction ID of the transaction of concern
    pub transaction_id: u32,
}