pub use anchor_lang::prelude::*;
pub use crate::constants;

//Method to calculate_reward
impl Reward {
    pub fn calculate_reward(&mut self) {
        let unix_now = Clock::get().unwrap().unix_timestamp; //current time
        let x:f64 = ((unix_now - constants::START)/constants::SEC_IN_DAY) as f64; //calculate number of days passed since the start
        let denominator = constants::EULER_NUMBER.powf(10.0-0.5*x.powf(0.4*1.0005_f64.powf(1_f64)))+1_f64; //denominator of math formula
        let multiplier = 1_f64 - 1_f64/(denominator); //math formula

        self.victory = round_down((constants::NFT_PRICE / constants::VICTORY) * multiplier); //value of reward given for victory
        self.top_five = round_down((constants::NFT_PRICE / constants::TOP_FIVE) * multiplier); //value of reward given for top 2 - top 5
        self.top_ten = round_down((constants::NFT_PRICE / constants::TOP_TEN) * multiplier); //value of reward given for top 6 - top 10
        self.kill = round_down((constants::NFT_PRICE / constants::KILL ) * multiplier); //value of reward given for kill
    }
}

fn round_down(value: f64) -> f64 { //function to round down to 12 decimals
    (value *(10.0_f64.powf(12.0))).floor() / (10.0_f64.powf(12.0))
}

#[derive(Accounts)]
pub struct InitializeReward<'info> {
    #[account(init, payer = payer, space = constants::MAX_SIZE_REWARD)]
    pub reward: Account<'info, Reward>,
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
    pub victory: f64,
    pub top_five: f64,
    pub top_ten: f64,
    pub kill: f64,
}
