//Sizes
export const FLOAT_MAX: number = 8;
export const PUBKEY_MAX: number = 32;
export const DISCRIMINATOR: number = 8;

//Consts for reward calculation
export const NFT_PRICE: number = 150.0; //nft not implemented yet, const now
export const EULER_NUMBER: number = 2.718_281_828_459; //const e
export const MAX_SIZE_REWARD: number = DISCRIMINATOR + FLOAT_MAX * 4; //Reward account four f64 field + discriminator
export const VICTORY: number = 16.8; //calculate the rewards based on the nft price, values not final
export const TOP_FIVE: number = 67.2;
export const TOP_TEN: number = 168.0;
export const KILL: number = 120.0;


//UNIX values
export const START: number = 1654797600; //smart contract start date, program not deployed yet, const now
export const SEC_IN_DAY: number = 86400; //seconds in day to calculate current day from the start
