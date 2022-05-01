#!/bin/sh

mkdir -p ./bin

(
cd ./zic-list-dirs || exit 1
cargo b --release
)

cp ./zic-list-dirs/target/release/zic-list-dirs ./bin/
rm -rf ./zic-list-dirs/target
