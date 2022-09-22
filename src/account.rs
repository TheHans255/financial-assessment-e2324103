use std::collections::HashMap;
use bigdecimal::BigDecimal;
use num_traits::Zero;

use crate::transaction::{Transaction, TransactionType, DisputeState};

#[derive(Clone)]
/// Structure for tracking account state
pub struct Account {
    /// The unique ID of the account
    pub id: u16,
    /// The account's current available balance. Available balance 
    /// can be utilized for withdrawals.
    pub available_balance: BigDecimal,
    /// The account's current held balance. Held balance relates to
    /// disputed transactions
    pub held_balance: BigDecimal,
    /// The total list of transactions this account has experienced,
    /// allowing us to later resolve disputes
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
            available_balance: Zero::zero(),
            held_balance: Zero::zero(),
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
                self.available_balance += &transaction.amount;
                self.transactions.insert(transaction.id, transaction);
            },
            TransactionType::Withdrawal => {
                if transaction.amount <= self.available_balance {
                    self.available_balance -= &transaction.amount;
                    self.transactions.insert(transaction.id, transaction);
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
                        // do not process if there are not enough available funds - this can happen
                        // if a person deposits money, withdraws some of that money, then disputes
                        // the original deposit
                        if transaction.amount <= self.available_balance {
                            self.available_balance -= &transaction.amount;
                            self.held_balance += &transaction.amount;
                            transaction.dispute_state = DisputeState::Disputed;
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
                        if transaction.amount <= self.held_balance {
                            self.held_balance -= &transaction.amount;
                            self.available_balance += &transaction.amount;
                            transaction.dispute_state = DisputeState::Undisputed;
                        } else {
                            // Because the held balance is always the exact sum of the deposit balances
                            // of all transactions currently under dispute, it should never go below zero
                            panic!("Held balance taken below zero - this should not happen");
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
                        if transaction.amount <= self.held_balance {
                            self.held_balance -= &transaction.amount;
                            self.is_frozen = true;
                            transaction.dispute_state = DisputeState::ChargedBack;
                        } else {
                            // Because the held balance is always the exact sum of the deposit balances
                            // of all transactions currently under dispute, it should never go below zero
                            panic!("Held balance taken below zero - this should not happen");
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