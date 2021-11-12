import * as schema from './schema.js';
import gemachainWeb3 from '@gemachain/web3.js';
import assert from 'assert';

export class StakePoolAccounts {
  /**
   * Wrapper class for a stake pool.
   * Each stake pool has a stake pool account and a validator list account.
   */
  stakePool: StakePoolAccount;
  validatorList: ValidatorListAccount;
}

export interface StakePoolAccount {
  pubkey: gemachainWeb3.PublicKey;
  account: gemachainWeb3.AccountInfo<schema.StakePool>;
}

export interface ValidatorListAccount {
  pubkey: gemachainWeb3.PublicKey;
  account: gemachainWeb3.AccountInfo<schema.ValidatorList>;
}

/**
 * Retrieves and deserializes a StakePool account using a web3js connection and the stake pool address.
 * @param connection: An active web3js connection.
 * @param stakePoolPubKey: The public key (address) of the stake pool account.
 */
export async function getStakePoolAccount(
  connection: gemachainWeb3.Connection,
  stakePoolPubKey: gemachainWeb3.PublicKey,
): Promise<StakePoolAccount> {
  const account = await connection.getAccountInfo(stakePoolPubKey);

  return {
    pubkey: stakePoolPubKey,
    account: {
      data: schema.StakePool.decode(account.data),
      executable: account.executable,
      carats: account.carats,
      owner: account.owner,
    },
  };
}

/**
 * Retrieves and deserializes a ValidatorList account using a web3js connection and the validator list address.
 * @param connection: An active web3js connection.
 * @param validatorListPubKey: The public key (address) of the validator list account.
 */
export async function getValidatorListAccount(
  connection: gemachainWeb3.Connection,
  validatorListPubKey: gemachainWeb3.PublicKey,
): Promise<ValidatorListAccount> {
  try {
    const account = await connection.getAccountInfo(validatorListPubKey);

    return {
      pubkey: validatorListPubKey,
      account: {
        data: schema.ValidatorList.decodeUnchecked(account.data),
        executable: account.executable,
        carats: account.carats,
        owner: account.owner,
      },
    };
  } catch (error) {
    console.log(error);
  }
}

/**
 * Retrieves all StakePool and ValidatorList accounts that are running a particular StakePool program.
 * @param connection: An active web3js connection.
 * @param stakePoolProgramAddress: The public key (address) of the StakePool program.
 */
export async function getStakePoolAccounts(
  connection: gemachainWeb3.Connection,
  stakePoolProgramAddress: gemachainWeb3.PublicKey,
): Promise<(StakePoolAccount | ValidatorListAccount)[]> {
  try {
    let response = await connection.getProgramAccounts(stakePoolProgramAddress);

    const stakePoolAccounts = response.map(a => {
      let decodedData;

      if (a.account.data.readUInt8() === 1) {
        try {
          decodedData = schema.StakePool.decode(a.account.data);
        } catch (error) {
          console.log('Could not decode StakeAccount. Error:', error);
          decodedData = undefined;
        }
      } else if (a.account.data.readUInt8() === 2) {
        try {
          decodedData = schema.ValidatorList.decodeUnchecked(a.account.data);
        } catch (error) {
          console.log('Could not decode ValidatorList. Error:', error);
          decodedData = undefined;
        }
      } else {
        console.error(
          `Could not decode. StakePoolAccount Enum is ${a.account.data.readUInt8()}, expected 1 or 2!`,
        );
        decodedData = undefined;
      }

      return {
        pubkey: a.pubkey,
        account: {
          data: decodedData,
          executable: a.account.executable,
          carats: a.account.carats,
          owner: a.account.owner,
        },
      };
    });

    return stakePoolAccounts;
  } catch (error) {
    console.log(error);
  }
}

/**
 * Helper function to pretty print a schema.PublicKey
 * Pretty prints a PublicKey in base58 format */
export function prettyPrintPubKey(pubKey: gemachainWeb3.PublicKey): string {
  return new gemachainWeb3.PublicKey(
    new gemachainWeb3.PublicKey(pubKey.toBuffer()).toBytes().reverse(),
  ).toString();
}

/**
 * Helper function to pretty print a decoded account
 */
export function prettyPrintAccount(
  account: ValidatorListAccount | StakePoolAccount,
): void {
  console.log('Address:', account.pubkey.toString());
  const sp = account.account.data;
  if (typeof sp === 'undefined') {
    console.log('Account could not be decoded');
  }

  for (const val in sp) {
    if (sp[val] instanceof gemachainWeb3.PublicKey) {
      console.log(val, prettyPrintPubKey(sp[val]));
    } else {
      console.log(val, sp[val]);
    }
  }
  console.log('Executable?:', account.account.executable);
  console.log('Carats:', account.account.carats);
  console.log('Owner PubKey:', account.account.owner.toString());
}
