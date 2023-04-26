# script for storing cw20-base contract on testnet

PREFIX="inj"
CHAIN_ID="injective-888"
# RPC="https://k8s.testnet.tm.injective.network:443"
RPC="https://testnet.tm.injective.network:443"
DAPP_ADDRESS="inj1amp7dv5fvjyx95ea4grld6jmu9v207awtefwce"

DAEMON="injectived"
DENOM="inj"

# ~0.0013 inj fee
TXFLAG="--gas-prices 1000000000$DENOM --gas auto --gas-adjustment 1.3 -y -b block --node $RPC --chain-id $CHAIN_ID"
DIR=$(pwd)
DIR_NAME=$(basename `dirname $PWD`)
WASM="packages/cw20_base.wasm"

# you must manually import all accounts from mnemonic via
# injectived keys add $user --recover
CONTRACT_CODE=$(yes 12345678 | $DAEMON tx wasm store $WASM --from dapp $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[1].value')
echo contract code is $CONTRACT_CODE

#---------- SMART CONTRACT INTERACTION ------------------------

#instantiate smart contract
INIT_ATOM='{
"name":"ATOM test token",
"symbol":"ATOM",
"decimals":6,
"initial_balances":[{"address":"'$DAPP_ADDRESS'","amount":"1000000000000"}],
"mint":{"minter":"'$DAPP_ADDRESS'"},
"marketing":{}
}'

INIT_LUNA='{
"name":"LUNA test token",
"symbol":"LUNA",
"decimals":6,
"initial_balances":[{"address":"'$DAPP_ADDRESS'","amount":"1100000000000"}],
"mint":{"minter":"'$DAPP_ADDRESS'"},
"marketing":{}
}'

INIT_USDC='{
"name":"USDC test token",
"symbol":"USDC",
"decimals":6,
"initial_balances":[{"address":"'$DAPP_ADDRESS'","amount":"1200000000000"}],
"mint":{"minter":"'$DAPP_ADDRESS'"},
"marketing":{}
}'

INIT_OSMO='{
"name":"OSMO test token",
"symbol":"OSMO",
"decimals":6,
"initial_balances":[{"address":"'$DAPP_ADDRESS'","amount":"1300000000000"}],
"mint":{"minter":"'$DAPP_ADDRESS'"},
"marketing":{}
}'

yes 12345678 | $DAEMON tx wasm instantiate $CONTRACT_CODE "$INIT_ATOM" --from "dapp" --label "test ATOM" $TXFLAG --admin $DAPP_ADDRESS
ATOM_CONTRACT=$($DAEMON query wasm list-contract-by-code $CONTRACT_CODE --node $RPC --chain-id $CHAIN_ID --output json | jq -r '.contracts[-1]')

yes 12345678 | $DAEMON tx wasm instantiate $CONTRACT_CODE "$INIT_LUNA" --from "dapp" --label "test LUNA" $TXFLAG --admin $DAPP_ADDRESS
LUNA_CONTRACT=$($DAEMON query wasm list-contract-by-code $CONTRACT_CODE --node $RPC --chain-id $CHAIN_ID --output json | jq -r '.contracts[-1]')

yes 12345678 | $DAEMON tx wasm instantiate $CONTRACT_CODE "$INIT_USDC" --from "dapp" --label "test USDC" $TXFLAG --admin $DAPP_ADDRESS
USDC_CONTRACT=$($DAEMON query wasm list-contract-by-code $CONTRACT_CODE --node $RPC --chain-id $CHAIN_ID --output json | jq -r '.contracts[-1]')

yes 12345678 | $DAEMON tx wasm instantiate $CONTRACT_CODE "$INIT_OSMO" --from "dapp" --label "test OSMO" $TXFLAG --admin $DAPP_ADDRESS
OSMO_CONTRACT=$($DAEMON query wasm list-contract-by-code $CONTRACT_CODE --node $RPC --chain-id $CHAIN_ID --output json | jq -r '.contracts[-1]')

# write data to file
cd $DIR
R="{
\"CONTRACT_CODE\":\"$CONTRACT_CODE\",
\"ATOM_CONTRACT\":\"$ATOM_CONTRACT\",
\"LUNA_CONTRACT\":\"$LUNA_CONTRACT\",
\"USDC_CONTRACT\":\"$USDC_CONTRACT\",
\"OSMO_CONTRACT\":\"$OSMO_CONTRACT\"
}"
echo $R > ../client/src/common/config/tokens.json

$DAEMON query wasm list-contract-by-code $CONTRACT_CODE --node $RPC --chain-id $CHAIN_ID --output json | jq -r '.contracts'