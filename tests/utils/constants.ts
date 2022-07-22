// Token
export const PLASMA_DECIMALS = 9;
export const PLASMA_INITIAL_SUPPLY = 812_500_000;
export const MEMBRANE_DECIMALS = 9;
export const MEMBRANE_INITIAL_SUPPLY = 812_500_000;

// Mocks
export const DECIMAL_PLACES = 0;
export const NFT_GRADE_MULTIPLIERS = {
  COMMON: 16_800_000_000
};

// Copy from programs/membrane/src/constants.rs

// Sizes
export const FLOAT_MAX: number = 8;
export const PUBKEY_MAX: number = 32;
export const DISCRIMINATOR: number = 8;

// Consts for reward calculation
export const MAX_SIZE_REWARD: number =
  DISCRIMINATOR +     // discriminator
  FLOAT_MAX +         // victory
  FLOAT_MAX +         // top_five
  FLOAT_MAX +         // top_ten
  FLOAT_MAX +         // kills
  FLOAT_MAX;          // days

export const NFT_PRICE: number = 150_000_000_000;
export const EULER_NUMBER: number = 2.718_281_828;
export const FEE_LAMPORTS: number = 100_000_000;

// UNIX values
export const START: number = 1654797600;
export const SEC_IN_DAY: number = 86400;

// Player account
export const MAX_PLAYER_SIZE: number =
  DISCRIMINATOR +     // discriminator
  PUBKEY_MAX +        // identity pubkey
  (1 + FLOAT_MAX) +   // rating wrapped in some
  FLOAT_MAX +         // claimable
  FLOAT_MAX;          // nft_counter

// PDA SEEDS
export const VAULT_PDA_SEED: string = 'vault';
