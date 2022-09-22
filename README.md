# Financial Assessment e2324103

This project is my solution to a take-home Rust programming assessment, and describes a financial
transaction resolver that handles deposits, withdrawals, and disputes.

## Usage

    financial-assessment-e2324103 input.csv

where `input.csv` is a CSV file with the following columns in order, with a header row
and one row per transaction:

- `type`: one of `deposit`, `withdrawal`, `dispute`, `resolve`, or `chargeback`
- `client`: the account number the transaction is applied to, from 0-65535
- `tx`: For `deposit` and `withdrawal` transactions, a unique ID number
  (from 0-4294967295) for the transaction. For `dispute`, `resolve`, or `chargeback`
  entries, the transaction ID under dispute.
- `amount`: For `deposit` and `withdrawal` transactions, the amount being withdrawn
  or deposited. Optional and ignored for `dispute`, `resolve`, and `chargeback`.

The output is a CSV file with the following columns, with a header row and one row
per account:
- `client`: The account number of the transaction
- `available`: The balance the account has available for withdrawals
- `held`: The balance the account has held in dispute
- `total`: The total balance the account has
- `locked`: Whether or not the account has been frozen by a successful chargeback
  (meaning that future deposits and withdrawals are disabled)

All amounts are accurate to four decimal places.

## Transaction Types

### Deposit

A `deposit` transaction represents an addition of funds to an account. The transaction
is recorded and the amount is added to the available funds.

### Withdrawal

A `withdrawal` transaction represents a removal of funds from an account. If there
is enough available balance for the transaction, it is recorded and the amount is
added to the available funds. Otherwise, the transaction is ignored.

### Dispute

A `dispute` action represents a dispute against a previous deposit. The available
funds from that deposit are held to be made unavailable for withdrawals, until the
transaction is later resolved or charged back.

If there is not enough available balance, or the transaction referred to is unknown,
the transaction is ignored. Under the current business requirements, withdrawals
cannot be disputed.

### Resolve

A `resolve` action cancels a dispute, making the held funds available again. If
the transaction referred to is not in dispute or does not exist, it is ignored.

### Chargeback

A `chargeback` action completes a dispute, removing the deposited funds and returning
them to the account holder. When a transaction is charged back, no further action
can be taken on it. In addition, in order to protect the account, it will be frozen
to ignore all future deposits and withdrawals (though disputes are still available).

# Error Conditions and Edge Cases

- Balances use the BigDecimal crates, which allow an arbitrary number of integer digits
  but only allows up to 2^63 possible decimal places, leading to a maximum mantissa
  of 10^2^63. Since there are only 10^186 Planck length cubes in the area of the observable
  universe, it is unlikely that this solution would overflow in real-world usage.
- If a withdrawal is ordered for more money than is available, the withdrawal is ignored.
- As per the business requirements, withdrawals process instantly. Because of this,
  withdrawals cannot be disputed (since there is no additional balance that is available
  to rectify them), and deposits cannot be disputed if there are not enough available
  funds remaining.

# Testing/Correctness

- Account functions and CSV input/output functions are unit tested, including
  state management, dispute resolution, and number conversion
- BigDecimal is used to ensure numerical correctness and prevent rounding errors.
- A handful of test files are used for integration testing. A Node.js script, `stress_gen.js`,
  is also provided to generate a long list of transactions and ensure stability.