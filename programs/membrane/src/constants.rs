//Sizes
pub const FLOAT_MAX: usize = 8;
pub const PUBKEY_MAX: usize = 32;


//Consts for reward calculation
pub const NFT_PRICE: f64 = 150.0;
pub const EULER_NUMBER: f64 = 2.718_281_828_459;
pub const MAX_SIZE_REWARD: usize = FLOAT_MAX*4;

//UNIX values
pub const START: i64 = 1654797600;
pub const SEC_IN_DAY: i64 = 86400;

//Player account
pub const MAX_PLAYER_SIZE: usize = PUBKEY_MAX + (1+FLOAT_MAX);