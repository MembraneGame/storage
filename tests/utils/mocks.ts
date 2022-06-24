import { BN } from '@project-serum/anchor';
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
import { adjustSupply, createToken, mintToken } from './web3';

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
    adjustSupply(PLASMA_INITIAL_SUPPLY, PLASMA_DECIMALS)
  );

  return {
    mintAddress,
    associatedTokenAddress: tokenAccount.address
  };
};

export type RewardParams = {
  victory: BN;
  topFive: BN;
  topTen: BN;
  kill: BN;
};

export const calculateInitialRewardParams = (
  decimals: number
): RewardParams => {
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
  const multiplier = new Decimal(1)
    .sub(new Decimal(1).div(denominator))
    .mul(new Decimal(10).pow(decimals))
    .toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN);

  const victory = nftPrice.div(victoryParam).mul(multiplier);
  const topFive = nftPrice.div(topFiveParam).mul(multiplier);
  const topTen = nftPrice.div(topTenParam).mul(multiplier);
  const kill = nftPrice.div(killParam).mul(multiplier);

  return {
    victory: new BN(
      victory.toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN).toNumber()
    ),
    topFive: new BN(
      topFive.toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN).toNumber()
    ),
    topTen: new BN(
      topTen.toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN).toNumber()
    ),
    kill: new BN(
      kill.toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN).toNumber()
    )
  };
};

export type LevelRangeValueTuple<T> = [T, number];

export const getLevelValue = <T extends Array<unknown>>(
  levelRanges: LevelRangeValueTuple<T>[],
  level: number
): number => {
  let result: number;
  for (const [range, value] of levelRanges) {
    const [start, end] = range;
    if (start && end) {
      if (start <= level && level <= end) {
        result = value;
        break;
      }
    } else if (start) {
      if (start === level) {
        result = value;
        break;
      }
    } else {
      result = value;
    }
  }
  console.log(result);
  return result;
};

export type RatingLevel = [number, number?];

export type PlacementLevel = [number?, number?];

export const calculatePlayerPayout = (
  placement: BN,
  kills: BN,
  rating: BN,
  rewardAccount: RewardParams
) => {
  const RATING_LEVEL_MULTIPLIERS: LevelRangeValueTuple<RatingLevel>[] = [
    [[0, 100], 0.8],
    [[101, 200], 0.9],
    [[201, Infinity], 1]
  ];
  const RATING_CHANGE: LevelRangeValueTuple<PlacementLevel>[] = [
    [[1], 10],
    [[2, 5], 5],
    [[6, 10], 2],
    [[], -2]
  ];
  const PLACEMENT_REWARDS: LevelRangeValueTuple<PlacementLevel>[] = [
    [[1], 10],
    [[2, 5], 5],
    [[6, 10], 2],
    [[], -2]
  ];
  const ratingMultiplier = getLevelValue(RATING_LEVEL_MULTIPLIERS, rating.toNumber());
  const ratingChange = getLevelValue(PLACEMENT_REWARDS, placement.toNumber());
  const placementReward = 0;
  const killReward = kills.mul(rewardAccount.kill);
};
