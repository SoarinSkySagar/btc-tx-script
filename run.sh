bitcoind -daemon -regtest
until bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin getblockchaininfo > /dev/null 2>&1; do
    echo "Waiting for bitcoind..."
    sleep 2
done
bitcoin-cli -regtest createwallet "mywallet"
cargo run --release
bitcoin-cli -regtest stop
echo "Done."