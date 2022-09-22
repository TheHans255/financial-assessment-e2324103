// Small Node.js script file to produce a long stress test file
// full of viable, but not necessarily cohesive transactions.
// The console output is a CSV file that can be subsequently input
// into the assessment executable.

const NUM_RECORDS = 1_000_000;
const TX_MAX = 4000000000;
const CLIENT_MAX = 60000;
const AMOUNT_MAX = 100000;
const DISPUTE_CHANCE = 0.2;

const TX_TYPES = ["deposit", "withdrawal"];
const DISPUTE_TYPES = ["dispute", "resolve", "chargeback"];

console.log("type, client, tx, amount");

for (let i = 0; i < NUM_RECORDS; i++) {
    let tx = Math.floor(Math.sqrt(Math.random() * TX_MAX * TX_MAX));
    let client = Math.floor(Math.sqrt(Math.random() * CLIENT_MAX * CLIENT_MAX));
    if (Math.random() < DISPUTE_CHANCE) {
        let type = DISPUTE_TYPES[Math.floor(Math.random() * DISPUTE_TYPES.length)]
        console.log(`${type},${client},${tx}`);
    } else {
        let type = TX_TYPES[Math.floor(Math.random() * TX_TYPES.length)]
        let amount = Math.random() * AMOUNT_MAX;
        console.log(`${type},${client},${tx},${amount}`);
    }
}