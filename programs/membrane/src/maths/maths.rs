pub use anchor_lang::prelude::*;
pub use crate::constants;

//Method to calculate_reward
impl Reward {
    pub fn calculate_reward(&mut self) {
        let unix_now = Clock::get().unwrap().unix_timestamp;
        let x:f64 = ((unix_now - constants::START)/constants::SEC_IN_DAY) as f64;
        let denominator = constants::EULER_NUMBER.powf(10.0-0.5*x.powf(0.4*1.0005_f64.powf(1_f64)))+1_f64;
        let multiplier = 1_f64 - 1_f64/(denominator);
        
        self.victory = (constants::NFT_PRICE / 16.8) * multiplier;
        self.top_five = (constants::NFT_PRICE / 67.2) * multiplier;
        self.top_ten = (constants::NFT_PRICE / 168.0) * multiplier;
    }
}

#[derive(Accounts)]
pub struct InitializeReward<'info> {
    #[account(init, payer = admin, space = constants::MAX_SIZE_REWARD)]
    pub reward: Account<'info, Reward>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub reward: Account<'info, Reward>,
}

#[account]
pub struct Reward {
    pub victory: f64,
    pub top_five: f64,
    pub top_ten: f64,
    pub kill: f64,
}
