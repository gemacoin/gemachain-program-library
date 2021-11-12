#!/usr/bin/env bash

# Script to setup a local gemachain-test-validator with the stake pool program

cd "$(dirname "$0")"
max_validators=$1
validator_list=$2

keys_dir=keys
mkdir -p $keys_dir
if test -f $validator_list
then
  rm $validator_list
fi

create_keypair () {
  if test ! -f $1
  then
    gemachain-keygen new --no-passphrase -s -o $1
  fi
}

build_program () {
  cargo build-bpf --manifest-path ../../program/Cargo.toml
}

setup_validator() {
  gemachain-test-validator --bpf-program SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy ../../../target/deploy/gpl_stake_pool.so --quiet --reset --slots-per-epoch 32 &
  pid=$!
  gemachain config set --url http://127.0.0.1:8899
  gemachain config set --commitment confirmed
  echo "waiting for gemachain-test-validator, pid: $pid"
  sleep 5
}

create_vote_accounts () {
  max_validators=$1
  validator_list=$2
  for number in $(seq 1 $max_validators)
  do
    create_keypair $keys_dir/identity_$number.json
    create_keypair $keys_dir/vote_$number.json
    gemachain create-vote-account $keys_dir/vote_$number.json $keys_dir/identity_$number.json --commission 1
    vote_pubkey=$(gemachain-keygen pubkey $keys_dir/vote_$number.json)
    echo $vote_pubkey >> $validator_list
  done
}

echo "Building on-chain program"
build_program

echo "Setting up local validator"
setup_validator

echo "Creating vote accounts"
create_vote_accounts $max_validators $validator_list
