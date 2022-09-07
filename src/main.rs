mod account;
mod csv_rows;
mod transaction;

use std::collections::BTreeMap;

use account::Account;
use csv::Trim;
use transaction::{ Transaction, DisputeAction, DisputeActionType };
use csv_rows::{ InputRow, OutputRow };

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

    let mut accounts: BTreeMap<u16, Account> = BTreeMap::new();
    for result in reader.deserialize() {
        // NOTE: This could be parallelized - multiple accounts do not interact.
        // However, since the application is mostly IO, there isn't much to gain here
        let input_row: InputRow = result.expect("IO error when reading file");
        if !accounts.contains_key(&input_row.client) {
            accounts.insert(input_row.client, Account::new(input_row.client));
        }
        let account: &mut Account = accounts.get_mut(&input_row.client).unwrap();
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

    let mut writer = csv::Writer::from_writer(std::io::stdout());
    for (_, account) in accounts.into_iter() {
        let output_row: OutputRow = account.into();
        writer.serialize(output_row).expect("Error when serializing record");
    }
}
