use std::collections::HashMap;
use bigdecimal::BigDecimal;
use num_traits::Zero;

use crate::transaction::{Transaction, TransactionType, DisputeState};

#[derive(Clone, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_with_zero_balance() {
        let account = Account::new(1);
        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&Zero::zero()));
        assert!(&(account.held_balance).eq(&Zero::zero()));
        assert!(&account.transactions.is_empty());
        assert!(!account.is_frozen);
    }

    #[test]
    fn records_deposit() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&10.into()));
        assert!(&(account.held_balance).eq(&Zero::zero()));
        assert_eq!(account.transactions.len(), 1 as usize);
        assert!(!account.is_frozen);
    }

    #[test]
    fn records_withdrawal() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.register_transaction(Transaction
            {
                id: 2,
                client_id: 1,
                amount: 8.into(),
                transaction_type: TransactionType::Withdrawal,
                dispute_state: DisputeState::Undisputed,
            });

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&2.into()));
        assert!(&(account.held_balance).eq(&Zero::zero()));
        assert_eq!(account.transactions.len(), 2 as usize);
        assert!(!account.is_frozen);
    }

    #[test]
    fn records_multiple_transactions() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.register_transaction(Transaction
            {
                id: 3,
                client_id: 1,
                amount: 15.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.register_transaction(Transaction
            {
                id: 2,
                client_id: 1,
                amount: 4.into(),
                transaction_type: TransactionType::Withdrawal,
                dispute_state: DisputeState::Undisputed,
            });

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&21.into()));
        assert!(&(account.held_balance).eq(&Zero::zero()));
        assert_eq!(account.transactions.len(), 3 as usize);
        assert!(!account.is_frozen);
    }

    #[test]
    fn ignores_duplicate_transaction_numbers() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 12.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&12.into()));
        assert!(&(account.held_balance).eq(&Zero::zero()));
        assert_eq!(account.transactions.len(), 1 as usize);
        assert!(!account.is_frozen);
    }

    #[test]
    fn records_dispute() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.dispute_transaction(1);

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&0.into()));
        assert!(&(account.held_balance).eq(&10.into()));
        assert_eq!(account.transactions.len(), 1 as usize);
        assert!(!account.is_frozen);
    }

    #[test]
    fn records_dispute_resolution() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.dispute_transaction(1);
        account.resolve_disputed_transaction(1);

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&10.into()));
        assert!(&(account.held_balance).eq(&0.into()));
        assert_eq!(account.transactions.len(), 1 as usize);
        assert!(!account.is_frozen);
    }

    #[test]
    fn records_dispute_chargeback() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.dispute_transaction(1);
        account.chargeback_disputed_transaction(1);

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&0.into()));
        assert!(&(account.held_balance).eq(&0.into()));
        assert_eq!(account.transactions.len(), 1 as usize);
        assert!(account.is_frozen);
    }

    #[test]
    fn disallows_further_transactions_after_chargeback() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.dispute_transaction(1);
        account.chargeback_disputed_transaction(1);
        account.register_transaction(Transaction
            {
                id: 2,
                client_id: 1,
                amount: 15.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&0.into()));
        assert!(&(account.held_balance).eq(&0.into()));
        assert_eq!(account.transactions.len(), 1 as usize);
        assert!(account.is_frozen);
    }

    #[test]
    fn allows_further_disputes_after_chargeback() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.register_transaction(Transaction
            {
                id: 2,
                client_id: 1,
                amount: 15.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.dispute_transaction(1);
        account.chargeback_disputed_transaction(1);
        account.dispute_transaction(2);

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&0.into()));
        assert!(&(account.held_balance).eq(&15.into()));
        assert_eq!(account.transactions.len(), 2 as usize);
        assert!(account.is_frozen);
    }

    #[test]
    fn ignores_resolve_on_undisputed() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.resolve_disputed_transaction(1);

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&10.into()));
        assert!(&(account.held_balance).eq(&0.into()));
        assert_eq!(account.transactions.len(), 1 as usize);
        assert!(!account.is_frozen);
    }

    #[test]
    fn ignores_chargeback_on_undisputed() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.chargeback_disputed_transaction(1);

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&10.into()));
        assert!(&(account.held_balance).eq(&0.into()));
        assert_eq!(account.transactions.len(), 1 as usize);
        assert!(!account.is_frozen);
    }

    #[test]
    fn ignores_disputes_on_unknown_transaction_numbers() {
        let mut account = Account::new(1);
        account.register_transaction(Transaction
            {
                id: 1,
                client_id: 1,
                amount: 10.into(),
                transaction_type: TransactionType::Deposit,
                dispute_state: DisputeState::Undisputed,
            });
        account.dispute_transaction(2);
        account.resolve_disputed_transaction(2);
        account.chargeback_disputed_transaction(2);

        assert_eq!(account.id, 1);
        assert!(&(account.available_balance).eq(&10.into()));
        assert!(&(account.held_balance).eq(&Zero::zero()));
        assert_eq!(account.transactions.len(), 1 as usize);
        assert!(!account.is_frozen);
    }
}