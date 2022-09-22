//! # Financial Assessment e2324103
//! 
//! A terminal application that processes a list of transactions, including deposits, withdrawals,
//! and various dispute actions on the above, and outputs the final state of accounts after those
//! transactions are applied.
//! 
//! ## Usage
//! 
//!     financial-assessment-e2324103 input.csv
//! 
//! where `input.csv` is a CSV file with the following columns in order, with a header row
//! and one row per transaction:
//! 
//! - `type`: one of `deposit`, `withdrawal`, `dispute`, `resolve`, or `chargeback`
//! - `client`: the account number the transaction is applied to, from 0-65535
//! - `tx`: For `deposit` and `withdrawal` transactions, a unique ID number
//!   (from 0-4294967295) for the transaction. For `dispute`, `resolve`, or `chargeback`
//!   entries, the transaction ID under dispute.
//! - `amount`: For `deposit` and `withdrawal` transactions, the amount being withdrawn
//!   or deposited. Optional and ignored for `dispute`, `resolve`, and `chargeback`.
//! 
//! The output is a CSV file with the following columns, with a header row and one row
//! per account:
//! - `client`: The account number of the transaction
//! - `available`: The balance the account has available for withdrawals
//! - `held`: The balance the account has held in dispute
//! - `total`: The total balance the account has
//! - `locked`: Whether or not the account has been frozen by a successful chargeback
//!   (meaning that future deposits and withdrawals are disabled)
//! 
//! All amounts are accurate to four decimal places.

mod account;
mod csv_rows;
mod transaction;

use std::collections::BTreeMap;

use account::Account;
use csv::Trim;
use transaction::{ Transaction, DisputeAction, DisputeActionType };
use csv_rows::{ InputRow, OutputRow };

/// Application entry point
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- filename.csv");
        return;
    }
    let filename = &(std::env::args().collect::<Vec<String>>()[1]);
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_path(filename)
        .expect("File does not exist");

    // Keep a dictionary of accounts by account number
    let mut accounts: BTreeMap<u16, Account> = BTreeMap::new();
    // Read and process each transaction row one at a time
    for result in reader.deserialize() {
        // NOTE: This could be parallelized - multiple accounts do not interact.
        // However, since the application is mostly IO, there isn't much to gain here
        let input_row: InputRow = result.expect("IO error when reading file");
        
        // Load the account, creating it if it does not exist
        if !accounts.contains_key(&input_row.client) {
            accounts.insert(input_row.client, Account::new(input_row.client));
        }
        let account: &mut Account = accounts.get_mut(&input_row.client).expect("Account should have been ensured by previous line");

        // Attempt parsing as a transaction, then as a dispute, executing the action
        // if either parse succeeds. Ignore all lines that do not specify appropriate actions.
        if let Ok(transaction) = input_row.clone().try_into() as Result<Transaction, _> {
            account.register_transaction(transaction);
        } else if let Ok(dispute_action) = input_row.try_into() as Result<DisputeAction, _> {
            match dispute_action.action_type {
                DisputeActionType::Dispute => account.dispute_transaction(dispute_action.transaction_id),
                DisputeActionType::Resolve => account.resolve_disputed_transaction(dispute_action.transaction_id),
                DisputeActionType::Chargeback => account.chargeback_disputed_transaction(dispute_action.transaction_id)
            }
        }
    }

    // Write the final state of all accounts as a CSV to stdout
    let mut writer = csv::Writer::from_writer(std::io::stdout());
    for (_, account) in accounts.into_iter() {
        let output_row: OutputRow = account.into();
        writer.serialize(output_row).expect("Error when serializing record");
    }
}
