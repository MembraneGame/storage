import Decimal from 'decimal.js';
import {
  DECIMAL_PLACES,
  EULER_NUMBER,
  KILL,
  NFT_PRICE,
  PLASMA_DECIMALS,
  PLASMA_INITIAL_SUPPLY,
  SEC_IN_DAY,
  START,
  TOP_FIVE,
  TOP_TEN,
  VICTORY
} from './constants';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { createToken, mintToken } from './web3';

export type MintResult = {
  mintAddress: PublicKey;
  associatedTokenAddress: PublicKey;
};

export const initializeMint = async (
  connection: Connection,
  storage: Keypair
): Promise<MintResult> => {
  const mintAddress = await createToken(
    connection,
    storage,
    storage.publicKey,
    null,
    PLASMA_DECIMALS
  );

  const tokenAccount = await mintToken(
    connection,
    mintAddress,
    storage,
    storage.publicKey,
    storage.publicKey,
    PLASMA_INITIAL_SUPPLY
  );

  return {
    mintAddress,
    associatedTokenAddress: tokenAccount.address
  };
};

export type RewardParams = {
  victory: number;
  topFive: number;
  topTen: number;
  kill: number;
};

export const calculateInitialRewardParams = (): RewardParams => {
  // Use Decimal for operations with numbers
  const dateNow = new Decimal(Date.now());
  const start = new Decimal(START);
  const secInDay = new Decimal(SEC_IN_DAY);
  const eulerNumber = new Decimal(EULER_NUMBER);
  const a = new Decimal(10);
  const b = new Decimal(0.5);
  const c = new Decimal(0.4);
  const d = new Decimal(1.0005);
  const currentUsers = new Decimal(1); // Value is not final
  const initialUsers = new Decimal(1); // Value is not final
  const nftPrice = new Decimal(NFT_PRICE);
  const victoryParam = new Decimal(VICTORY);
  const topFiveParam = new Decimal(TOP_FIVE);
  const topTenParam = new Decimal(TOP_TEN);
  const killParam = new Decimal(KILL);

  const unixNow = dateNow.divToInt(new Decimal(1000)); // Should be an integer
  const x = unixNow.sub(start).divToInt(secInDay); // Should be an integer
  const xExp = c.mul(d.pow(currentUsers.div(initialUsers)));
  const denominatorExp = a.sub(b.mul(x.pow(xExp)));
  const denominator = eulerNumber.pow(denominatorExp).add(new Decimal(1));
  const multiplier = new Decimal(1).sub(new Decimal(1).div(denominator));

  const victory = nftPrice.div(victoryParam).mul(multiplier);
  const topFive = nftPrice.div(topFiveParam).mul(multiplier);
  const topTen = nftPrice.div(topTenParam).mul(multiplier);
  const kill = nftPrice.div(killParam).mul(multiplier);

  return {
    victory: victory
      .toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN)
      .toNumber(),
    topFive: topFive
      .toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN)
      .toNumber(),
    topTen: topTen
      .toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN)
      .toNumber(),
    kill: kill.toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN).toNumber()
  };
};
