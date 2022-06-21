//Sizes
pub const FLOAT_MAX: usize = 8;
pub const PUBKEY_MAX: usize = 32;
pub const DISCRIMINATOR: usize = 8;

//Consts for reward calculation
pub const NFT_PRICE: f64 = 150.0; //nft not implemented yet, const now
pub const EULER_NUMBER: f64 = 2.718_281_828_459; //const e
pub const MAX_SIZE_REWARD: usize = FLOAT_MAX*4 + DISCRIMINATOR; //Reward account four f64 field + discriminator

//UNIX values
pub const START: i64 = 1654797600; //smart contract start date, program not deployed yet, const now
pub const SEC_IN_DAY: i64 = 86400; //seconds in day to calculate current day from the start

//Player account
pub const MAX_PLAYER_SIZE: usize = PUBKEY_MAX + (1+FLOAT_MAX) + DISCRIMINATOR; //pubkey + rating wrapped in some + discriminator for Player account
