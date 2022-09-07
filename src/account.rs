use std::collections::HashMap;

use crate::transaction::{Transaction, TransactionType, DisputeState};

#[derive(Clone)]
/// Structure for tracking account state
pub struct Account {
    /// The unique ID of the account
    pub id: u16,
    /// The account's current available balance. Available balance 
    /// can be utilized for withdrawals.
    pub available_balance: u64,
    /// The account's current held balance. Held balance relates to
    /// disputed transactions
    pub held_balance: u64,
    /// The total list of transactions this account has experienced
    pub transactions: HashMap<u32, Transaction>,
    /// Whether the account has been frozen. An account is a frozen
    /// if a chargeback has been processed on it
    pub is_frozen: bool,
}

impl Account {
    /// Create a new account with zero transaction history
    pub fn new(id: u16) -> Self {
        Self {
            id, 
            available_balance: 0,
            held_balance: 0,
            transactions: HashMap::new(),
            is_frozen: false
        }
    }

    /// Register and apply a new transaction
    pub fn register_transaction(&mut self, transaction: Transaction) {
        if self.is_frozen {
            // Do not process new transactions if the account is frozen.
            // Disputes are still allowed.
            return;
        }
        if self.transactions.contains_key(&transaction.id) {
            // Do not process transactions with duplicate IDs
            return;
        }

        match transaction.transaction_type {
            TransactionType::Deposit => {
                match self.available_balance.checked_add(transaction.amount) {
                    Some(result) => { 
                        self.available_balance = result;
                        self.transactions.insert(transaction.id, transaction);
                    },
                    None => panic!("Account balance overflow")
                }
            },
            TransactionType::Withdrawal => {
                match self.available_balance.checked_sub(transaction.amount) {
                    Some(result) => { 
                        self.available_balance = result;
                        self.transactions.insert(transaction.id, transaction);
                    },
                    None => { /* do not process an overdraw */ }
                }
            }
        }
    }

    /// Indicate a transaction in dispute
    pub fn dispute_transaction(&mut self, transaction_id: u32) {
        if let Some(transaction) = self.transactions.get_mut(&transaction_id) {
            if transaction.dispute_state == DisputeState::Undisputed {
                match transaction.transaction_type {
                    TransactionType::Deposit => {
                        match self.available_balance.checked_sub(transaction.amount) {
                            Some(result) => {
                                self.available_balance = result;
                                self.held_balance = self.held_balance.checked_add(transaction.amount).unwrap();
                                transaction.dispute_state = DisputeState::Disputed;
                            },
                            None => {
                                // do not process if there are not enough available funds - this can happen
                                // if a person deposits money, withdraws some of that money, then disputes
                                // the original deposit
                                // NOTE: I'm not sure this is the correct behavior - ideally we would hold
                                // as much as we can and keep track of a deficit that we would need to manually
                                // supply from elsewhere to make things right.
                            }
                        }
                    },
                    TransactionType::Withdrawal => {
                        // do not dispute a withdrawal - there's really nothing we can do when the
                        // withdrawal has been processed, since the money is already gone
                        // NOTE: If we gave the withdrawal a holding period, then we could allow a dispute
                        // to cancel the withdrawal. This would also let us dispute deposits with not enough
                        // funds remaining by canceling interfering withdrawals
                    }
                }
            }
        }
    }
    
    /// Cancel a dispute on a transaction
    pub fn resolve_disputed_transaction(&mut self, transaction_id: u32) {
        if let Some(transaction) = self.transactions.get_mut(&transaction_id) {
            if transaction.dispute_state == DisputeState::Disputed {
                match transaction.transaction_type {
                    TransactionType::Deposit => {
                        match self.held_balance.checked_sub(transaction.amount) {
                            Some(result) => {
                                self.held_balance = result;
                                self.available_balance = self.available_balance.checked_add(transaction.amount).unwrap();
                                transaction.dispute_state = DisputeState::Undisputed;
                            },
                            None => { 
                                panic!("Held balance taken below zero - this should not happen");
                            }
                        }
                    },
                    TransactionType::Withdrawal => {
                        /* withdrawals can't be disputed, so do nothing */
                    }
                }
            }
        }
    }

    /// Charge back a disputed transaction and freeze the account
    pub fn chargeback_disputed_transaction(&mut self, transaction_id: u32) {
        if let Some(transaction) = self.transactions.get_mut(&transaction_id) {
            if transaction.dispute_state == DisputeState::Disputed {
                match transaction.transaction_type {
                    TransactionType::Deposit => {
                        match self.held_balance.checked_sub(transaction.amount) {
                            Some(result) => {
                                self.held_balance = result;
                                self.is_frozen = true;
                                transaction.dispute_state = DisputeState::ChargedBack;
                            },
                            None => { 
                                panic!("Held balance taken below zero - this should not happen");
                            }
                        }
                    },
                    TransactionType::Withdrawal => {
                        /* withdrawals can't be disputed, so do nothing */
                    }
                }
            }
        }        
    }
}