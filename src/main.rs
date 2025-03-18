use bitcoin::{
    blockdata::{opcodes::OP_0, transaction::{Transaction, TxIn, TxOut}},
    locktime::absolute::LockTime, script::Builder, Network, OutPoint, Sequence, Witness
};
use bitcoincore_rpc::{Auth, Client, RpcApi};

fn main() {
    let rpc = Client::new(
        "http://127.0.0.1:18443/wallet/mywallet",
        Auth::UserPass("bitcoin".to_string(), "bitcoin".to_string()),
    ).expect("Failed to connect to bitcoind");

    let address = rpc.get_new_address(None, None).unwrap();
    let checked_address = address.require_network(Network::Regtest).unwrap();

    // Mine 100 blocks to generate spendable coinbase transactions
    let block_hashes = rpc.generate_to_address(100, &checked_address).expect("Failed to mine 100 blocks");
    println!("100 blocks mined.");

    // Get the first block's transaction (coinbase tx)
    let first_block = rpc.get_block(&block_hashes[0]).expect("Failed to fetch first block");
    let coinbase_txid = first_block.txdata[0].txid();
    println!("Using coinbase txid: {:?}", coinbase_txid);
    let coinbase_info = rpc.get_raw_transaction(&coinbase_txid, None).expect("Failed to fetch coinbase tx");

    let script_pubkey = checked_address.script_pubkey();

    // Create first transaction spending the coinbase output
    let tx1 = Transaction {
        version: 2,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint { txid: coinbase_txid, vout: 0 },
            script_sig: Builder::new().push_opcode(OP_0).into_script(),
            sequence: Sequence::MAX,
            witness: Witness::default(),
        }],
        output: vec![TxOut {
            value: coinbase_info.output[0].value - 1_000,
            script_pubkey: script_pubkey.clone(),
        }],
    };

    let signed_tx1 = rpc.sign_raw_transaction_with_wallet(&tx1, None, None).expect("Failed to sign tx1").hex;
    let tx1_id = rpc.send_raw_transaction(&signed_tx1).expect("Failed to broadcast tx1");
    rpc.generate_to_address(1, &checked_address).unwrap();
    println!("Tx1 submitted: {:?}", tx1_id);

    let utxos = rpc.list_unspent(None, None, None, None, None).unwrap();
    assert!(utxos.iter().any(|u| u.txid == tx1_id), "Tx1 output not found!");

    let tx1_details = rpc.get_raw_transaction(&tx1_id, None).unwrap();
    println!("Tx1 outputs: {:?}", tx1_details.output);

    // Create second transaction spending from tx1
    let tx2 = Transaction {
        version: 2,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint { txid: tx1_id, vout: 0 },
            script_sig: Builder::new().push_opcode(OP_0).into_script(),
            sequence: Sequence::MAX,
            witness: Witness::default(),
        }],
        output: vec![TxOut {
            value: tx1_details.output[0].value - 1_000,
            script_pubkey: script_pubkey.clone(),
        }],
    };

    let signed_tx2 = rpc.sign_raw_transaction_with_wallet(&tx2, None, None).expect("Failed to sign tx2").hex;
    let tx2_id = rpc.send_raw_transaction(&signed_tx2).expect("Failed to broadcast tx2");
    println!("Tx2 submitted: {:?}", tx2.txid());
    println!("Tx2 outputs: {:?}", rpc.get_raw_transaction(&tx2_id, None).unwrap().output);
}
