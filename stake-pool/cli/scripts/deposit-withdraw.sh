#!/usr/bin/env bash

# Script to deposit and withdraw stakes from a pool, given stake pool public key
# and a list of validators

cd "$(dirname "$0")"
stake_pool_keyfile=$1
validator_list=$2

stake_pool_pubkey=$(gemachain-keygen pubkey $stake_pool_keyfile)

gema_amount=2
half_gema_amount=1
keys_dir=keys
gpl_stake_pool=../../../target/debug/gpl-stake-pool

mkdir -p $keys_dir

create_keypair () {
  if test ! -f $1
  then
    gemachain-keygen new --no-passphrase -s -o $1
  fi
}

create_user_stakes () {
  validator_list=$1
  gema_amount=$2
  for validator in $(cat $validator_list)
  do
    create_keypair $keys_dir/stake_$validator.json
    gemachain create-stake-account $keys_dir/stake_$validator.json $gema_amount
  done
}

delegate_user_stakes () {
  validator_list=$1
  for validator in $(cat $validator_list)
  do
    gemachain delegate-stake --force $keys_dir/stake_$validator.json $validator
  done
}

deposit_stakes () {
  stake_pool_pubkey=$1
  validator_list=$2
  for validator in $(cat $validator_list)
  do
    stake=$(gemachain-keygen pubkey $keys_dir/stake_$validator.json)
    $gpl_stake_pool deposit-stake $stake_pool_pubkey $stake
  done
}

withdraw_stakes () {
  stake_pool_pubkey=$1
  validator_list=$2
  pool_amount=$3
  for validator in $(cat $validator_list)
  do
    $gpl_stake_pool withdraw-stake $stake_pool_pubkey $pool_amount --vote-account $validator
  done
}

echo "Creating user stake accounts"
create_user_stakes $validator_list $gema_amount
echo "Delegating user stakes"
delegate_user_stakes $validator_list
echo "Waiting for stakes to activate, this may take awhile depending on the network!"
echo "If you are running on localnet with 32 slots per epoch, wait 24 seconds..."
sleep 24
echo "Depositing stakes into stake pool"
deposit_stakes $stake_pool_pubkey $validator_list
echo "Withdrawing stakes from stake pool"
withdraw_stakes $stake_pool_pubkey $validator_list $half_gema_amount
echo "Withdrawing gema from stake pool"
$gpl_stake_pool withdraw-gema $stake_pool_pubkey $half_gema_amount
