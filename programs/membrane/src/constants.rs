//Sizes
pub const FLOAT_MAX: usize = 8;
pub const PUBKEY_MAX: usize = 32;
pub const DISCRIMINATOR: usize = 8;
pub const INT_SMALL: usize = 1;
pub const VECTOR_SIZE: usize = 4;
pub const MAX_PLAYERS: usize = 32;

//Consts for reward calculation
pub const NFT_PRICE: u64 = 150000000000; //nft not implemented yet, const now
pub const EULER_NUMBER: f64 = 2.718_281_828; //const e
pub const MAX_SIZE_REWARD: usize = FLOAT_MAX*5 + DISCRIMINATOR; //Reward account four u64 fields and one i64 + discriminator
pub const VICTORY: u64 = 16800000000; //calculate the rewards based on the nft price, values not final
pub const TOP_FIVE: u64 = 67200000000;
pub const TOP_TEN: u64= 168000000000;
pub const KILL: u64 = 120000000000;
pub const FEE_LAMPORTS: u64 = 100_000_000;


//UNIX values
pub const START: i64 = 1654797600; //smart contract start date, program not deployed yet, const now
pub const SEC_IN_DAY: i64 = 86400; //seconds in day to calculate current day from the start

//Player account
pub const MAX_PLAYER_SIZE: usize = FLOAT_MAX + (1+FLOAT_MAX) + DISCRIMINATOR + FLOAT_MAX + PUBKEY_MAX + 5*FLOAT_MAX; //u64 + rating wrapped in some + discriminator for Player account + claimable i64 + pubkey
pub const MAX_ACCOUNT_SIZE: usize = 10485760;

//PDA SEEDS
pub const VAULT_PDA_SEED: &[u8] = b"vault";