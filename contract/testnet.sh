# script for storing contract on testnet

PREFIX="inj"
CHAIN_ID="injective-888"
# RPC="https://k8s.testnet.tm.injective.network:443"
RPC="https://testnet.tm.injective.network:443"
# inj1prmtvxpvdcmp3dtn6qn4hyq9gytj5ry4u28nqz
SEED_ALICE=$(jq -r '.ALICE_SEED' ../../.test-wallets/test_wallets.json)
# inj1hag3kx8f9ypnssw7aqnq9e82t2zgt0g0ac2rru
SEED_BOB=$(jq -r '.BOB_SEED' ../../.test-wallets/test_wallets.json)
# inj1amp7dv5fvjyx95ea4grld6jmu9v207awtefwce
SEED_DAPP=$(jq -r '.JOHN_SEED' ../../.test-wallets/test_wallets.json)
DAPP_ADDRESS="inj1amp7dv5fvjyx95ea4grld6jmu9v207awtefwce"

DAEMON="injectived"
DENOM="inj"

# ~0.0013 inj fee
TXFLAG="--gas-prices 1000000000$DENOM --gas auto --gas-adjustment 1.3 -y -b block --node $RPC --chain-id $CHAIN_ID"
DIR=$(pwd)
DIR_NAME=$(basename `dirname $PWD`)
DIR_NAME_SNAKE=$(echo $DIR_NAME | tr '-' '_')
WASM="artifacts/$DIR_NAME_SNAKE.wasm"


$DAEMON q bank balances $DAPP_ADDRESS --denom "inj" --node $RPC --chain-id $CHAIN_ID

# you must manually import all accounts from mnemonic via
# injectived keys add $user --recover
CONTRACT_CODE=$(yes 12345678 | $DAEMON tx wasm store $WASM --from dapp $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[1].value')
echo contract code is $CONTRACT_CODE

$DAEMON q bank balances $DAPP_ADDRESS --denom "inj" --node $RPC --chain-id $CHAIN_ID

#---------- SMART CONTRACT INTERACTION ------------------------

# instantiate smart contract
INIT='{}'
yes 12345678 | $DAEMON tx wasm instantiate $CONTRACT_CODE "$INIT" --from "dapp" --label "ion-flux-dev" $TXFLAG --admin $DAPP_ADDRESS

# get smart contract address
CONTRACT_ADDRESS=$($DAEMON query wasm list-contract-by-code $CONTRACT_CODE --node $RPC --chain-id $CHAIN_ID --output json | jq -r '.contracts[-1]')

# write data to file
cd $DIR
R="{
\"PREFIX\":\"$PREFIX\",
\"CHAIN_ID\":\"$CHAIN_ID\",
\"RPC\":\"$RPC\",
\"CONTRACT_CODE\":\"$CONTRACT_CODE\",
\"CONTRACT_ADDRESS\":\"$CONTRACT_ADDRESS\",
\"SEED_ALICE\":\"$SEED_ALICE\",
\"SEED_BOB\":\"$SEED_BOB\",
\"SEED_DAPP\":\"$SEED_DAPP\"
}"
echo $R > ../client/src/common/config/testnet-config.json
