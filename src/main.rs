use bitcoin::{
    blockdata::{script::Script, transaction::{Transaction, TxIn, TxOut}}, locktime::absolute::LockTime, Network, OutPoint, ScriptBuf, Sequence, Witness
};
use bitcoincore_rpc::{Auth, Client, RpcApi};

fn main() {
    let rpc = Client::new(
        "http://127.0.0.1:38332",
        Auth::UserPass("bitcoin".to_string(), "bitcoin".to_string()),
    )
    .expect("Failed to connect to bitcoind");

    // Get available UTXO from the RPC client
    let utxos = rpc
        .list_unspent(None, None, None, None, None)
        .expect("Failed to get UTXOs");

    let first_utxo = utxos.first().expect("No UTXOs available");

    let empty_script: ScriptBuf = Script::empty().into();

    let address = &rpc.get_new_address(None, None).unwrap();
    let checked_address = address.clone().require_network(Network::Signet).unwrap();
    let script_pubkey = checked_address.script_pubkey();

    // Create the coinbase transaction
    let tx1 = Transaction {
        version: 2,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: first_utxo.txid,
                vout: first_utxo.vout,
            },
            script_sig: empty_script.clone(),
            sequence: Sequence::MAX,
            witness: Witness::default(),
        }],
        output: vec![TxOut {
            value: first_utxo.amount.to_sat() - 1000,
            script_pubkey: script_pubkey.clone(),
        }],
    };

    let signed_tx1 = rpc
        .sign_raw_transaction_with_wallet(&tx1, None, None)
        .expect("Failed to sign tx1")
        .hex;

    rpc.send_raw_transaction(&signed_tx1)
        .expect("Failed to broadcast tx1");

    println!("Tx1 submitted: {:?}", tx1.txid());

    // Create second transaction spending conbase tx's output
    let tx2 = Transaction {
        version: 2,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: tx1.txid(),
                vout: 0,
            },
            script_sig: empty_script.clone(),
            sequence: Sequence::MAX,
            witness: Witness::default(),
        }],
        output: vec![TxOut {
            value: first_utxo.amount.to_sat() - 2000,
            script_pubkey: script_pubkey.clone(),
        }],
    };

    let signed_tx2 = rpc
        .sign_raw_transaction_with_wallet(&tx2, None, None)
        .expect("Failed to sign tx2")
        .hex;

    rpc.send_raw_transaction(&signed_tx2)
        .expect("Failed to broadcast tx2");

    println!("Tx2 submitted: {:?}", tx2.txid());
}
