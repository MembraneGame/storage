import { BN } from '@project-serum/anchor';
import Decimal from 'decimal.js';
import {
  DECIMAL_PLACES,
  EULER_NUMBER,
  NFT_PRICE,
  PLASMA_DECIMALS,
  PLASMA_INITIAL_SUPPLY,
  SEC_IN_DAY,
  START
} from './constants';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { adjustSupply, createToken, mintToken } from './web3';

export type Stat = {
  id: PublicKey;
  placement: number;
  kills: number;
  reward: BN;
};

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
    storage.publicKey,
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
  nftGradeMultiplier: number,
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
  const victoryParam = new Decimal(1);
  const topFiveParam = new Decimal(4);
  const topTenParam = new Decimal(10);
  const killParam = new Decimal(150);

  const unixNow = dateNow.divToInt(new Decimal(1000)); // Should be an integer
  const x = unixNow.sub(start).divToInt(secInDay); // Should be an integer
  const xExp = c.mul(d.pow(currentUsers.div(initialUsers)));
  const denominatorExp = a.sub(b.mul(x.pow(xExp)));
  const denominator = eulerNumber.pow(denominatorExp).add(new Decimal(1));
  const multiplier = new Decimal(1)
    .sub(new Decimal(1).div(denominator))
    .mul(new Decimal(10).pow(decimals))
    .toDecimalPlaces(DECIMAL_PLACES, Decimal.ROUND_DOWN);

  const victory = nftPrice.div(victoryParam.mul(nftGradeMultiplier)).mul(multiplier);
  const topFive = nftPrice.div(topFiveParam.mul(nftGradeMultiplier)).mul(multiplier);
  const topTen = nftPrice.div(topTenParam.mul(nftGradeMultiplier)).mul(multiplier);
  const kill = nftPrice.mul(7).div(killParam.mul(nftGradeMultiplier)).mul(multiplier);

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

export const getRandomInt = (min: number, max: number): number => {
  min = Math.ceil(min);
  max = Math.floor(max);
  return Math.floor(Math.random() * (max - min + 1)) + min;
};

export type LevelRangeValueTuple<T> = [T, Decimal];

export const getLevelValue = <T extends Array<unknown>>(
  levelRanges: LevelRangeValueTuple<T>[],
  level: number
): Decimal => {
  let result: Decimal;
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
  return result;
};

export type RatingLevel = [number, number?];

export type PlacementLevel = [number?, number?];

export type PlayerPayoutResult = {
  rewardAmount: BN;
  ratingChange: BN;
};

export const RATING_LEVEL_MULTIPLIERS: LevelRangeValueTuple<RatingLevel>[] = [
  [[0, 100], new Decimal(8)],
  [[101, 200], new Decimal(9)],
  [[201, Infinity], new Decimal(10)]
];
export const RATING_CHANGE: LevelRangeValueTuple<PlacementLevel>[] = [
  [[1], new Decimal(10)],
  [[2, 5], new Decimal(5)],
  [[6, 10], new Decimal(2)],
  [[], new Decimal(-2)]
];

export const calculatePlayerPayout = (
  placement: BN,
  kills: BN,
  rating: BN,
  rewardAccount: RewardParams
): PlayerPayoutResult => {
  const PLACEMENT_REWARDS: LevelRangeValueTuple<PlacementLevel>[] = [
    [[1], new Decimal(rewardAccount.victory.toNumber())],
    [[2, 5], new Decimal(rewardAccount.topFive.toNumber())],
    [[6, 10], new Decimal(rewardAccount.topTen.toNumber())],
    [[], new Decimal(0)]
  ];
  const ratingMultiplier = getLevelValue(
    RATING_LEVEL_MULTIPLIERS,
    rating.toNumber()
  );
  const ratingChange = getLevelValue(RATING_CHANGE, placement.toNumber());
  const placementReward = getLevelValue(
    PLACEMENT_REWARDS,
    placement.toNumber()
  );
  const killReward = new Decimal(kills.mul(rewardAccount.kill).toNumber());

  const reward = ratingMultiplier
    .mul(placementReward.add(killReward))
    .div(10)
    .toDecimalPlaces(0, Decimal.ROUND_DOWN);

  return {
    rewardAmount: new BN(reward.toNumber()),
    ratingChange: new BN(ratingChange.toNumber())
  };
};

export type GameResult = {
  placement: BN;
  kills: BN;
};

export const generateRandomGameResult = (players: number = 15): GameResult => {
  const placement = getRandomInt(1, players);
  const min = placement > 3 ? 0 : 1;
  const kills = getRandomInt(min, players - placement);
  return {
    placement: new BN(placement),
    kills: new BN(kills)
  };
};

export const generateGameResults = (players: number = 15): GameResult[] => {
  let totalKills = players;
  const results = [];

  for (let p = 1; p <= players; p++) {
    const min = p > 3 ? 0 : 1;
    const max = totalKills - min;
    const k = getRandomInt(min, max);
    totalKills = totalKills - k;
    results.push({
      placement: new BN(p),
      kills: new BN(k)
    });
  }

  return results;
};
