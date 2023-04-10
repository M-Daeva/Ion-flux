# script for building wasm

DIR=$(pwd)
DIR_NAME=$(basename `dirname $PWD`)
DIR_NAME_SNAKE=$(echo $DIR_NAME | tr '-' '_')
WASM="artifacts/$DIR_NAME_SNAKE.wasm"

cd ./artifacts
sudo rm -rf $(ls -I "$DIR_NAME_SNAKE.wasm" )
cd ../

# generate schemas
cargo schema

# fix for ts-codegen MissingPointerError
# https://github.com/CosmWasm/ts-codegen/issues/90
rm -rf ./schema/raw

# generate contract-to-client interface
cosmwasm-ts-codegen generate \
  --plugin client \
	--plugin message-composer \
  --schema ./schema \
  --out ../client/src/common/codegen \
  --name $DIR_NAME \
  --no-bundle

# build optimized binary
echo "building optimized binary..."
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13

cd ./artifacts
sudo rm -rf $(ls -I "$DIR_NAME_SNAKE.wasm" )

