pub use anchor_lang::prelude::*;
pub use crate::constants;

//Method to calculate_reward
impl Reward {
    pub fn calculate_reward(&mut self, nft_multiplier: u64) {
        let unix_now = Clock::get().unwrap().unix_timestamp; //current time
        let x:f64 = ((unix_now - constants::START)/constants::SEC_IN_DAY) as f64; //calculate number of days passed since the start
        let denominator = constants::EULER_NUMBER.powf(10.0-0.5*x.powf(0.4*1.0005_f64.powf(1_f64)))+1_f64; //denominator of math formula
        let multiplier: u64 = ((1_f64 - 1_f64/(denominator)) *10.0_f64.powf(9.0)) as u64; //math formula

        self.victory = ((constants::NFT_PRICE as f64 / nft_multiplier as f64) * multiplier as f64) as u64; //value of reward given for victory
        self.top_five = ((constants::NFT_PRICE as f64 / (4.0 * nft_multiplier as f64)) * multiplier as f64) as u64; //value of reward given for top 2 - top 5
        self.top_ten = ((constants::NFT_PRICE as f64 / (10.0 * nft_multiplier as f64)) * multiplier as f64) as u64; //value of reward given for top 6 - top 10
        self.kill = ((7.0 * constants::NFT_PRICE as f64 / (150.0 * nft_multiplier as f64)) * multiplier as f64) as u64; //value of reward given for kill
    }
}

#[derive(Accounts)]
pub struct InitializeReward<'info> {
    #[account(init, payer = payer, space = constants::MAX_SIZE_REWARD)]
    pub reward: Account<'info, Reward>,
    pub nft_multiplier: Account<'info, QualityMultiplier>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub reward: Account<'info, Reward>,
}

#[account]
pub struct Reward {
    pub victory: u64,
    pub top_five: u64,
    pub top_ten: u64,
    pub kill: u64,
    pub days: i64,
}

#[account]
pub struct QualityMultiplier {
    pub common: u64, //TODO: add other qualities
}