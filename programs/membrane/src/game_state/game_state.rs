use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer};
use crate::maths::DaysPassed;
use crate::player_state;
use crate::errors;
use crate::maths;
pub use crate::constants;

pub fn initialize_reward(ctx: Context<maths::InitializeReward>) -> Result<()> {
    let reward_account = &mut ctx.accounts.reward;
    let days = &mut ctx.accounts.days; //define days account
    days.days = 0; //set days to 0
    reward_account.calculate_reward();
    Ok(())
}

//Fn to pay player
pub fn payout(ctx: Context<Payout>, placement: u64, kills: u64) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let rating_multiplier:f64 = match player.rating { //match rating_multiplier
        Some(0..=100) => 0.8, //values not final
        Some(101..=200) => 0.9,
        Some(201..) => 1.0,
        None => return Err(errors::ErrorCode::RatingUndefined.into()),
        _ => return Err(errors::ErrorCode::RatingOverflow.into()),
    };
    let reward_account = &mut ctx.accounts.reward;

    let days = &mut ctx.accounts.days; //define days account

    let unix_now = Clock::get().unwrap().unix_timestamp; //current time to compare

    if ((unix_now - constants::START)/constants::SEC_IN_DAY) != days.days { //if statement to check whether next day has begun
        reward_account.calculate_reward(); //Calculate and update the reward account
        reward_account.reload()?; //update the reward account if new day begun
        days.days = (unix_now - constants::START)/constants::SEC_IN_DAY;
    }



    //Define placement_reward based on placement
    let placement_reward = match placement { //match the player placement and update player account
        1 => {
            player.rating = Some(player
                .rating
                .unwrap() + 10); //values not final
            reward_account.victory
        },
        2..=5 => {
            player.rating = Some(player
                .rating
                .unwrap() + 5);
            reward_account.top_five
        },
        6..=10 => {
            player.rating = Some(player
                .rating
                .unwrap() + 2);
            reward_account.top_ten
        },
        _ => {
            player.rating = Some(player
                .rating
                .unwrap() - 2);
            if player
            .rating
            .unwrap() < 0 {
                player.rating = Some(0);
            }
            0
        },
    };

    let kill_reward = kills * reward_account.kill; //calculate total reward for kills
    let reward = (rating_multiplier * (placement_reward + kill_reward) as f64) as u64; //calculate total reward

    //Define Transfer account
    let cpi_accounts = Transfer {
        from: ctx
        .accounts
        .vault_token
        .to_account_info(), 

        to: ctx
        .accounts
        .player_token
        .to_account_info(), 

        authority: ctx
        .accounts
        .sender
        .to_account_info(), 
    };
    //Define token program
    let cpi_program = ctx.accounts.token_program.to_account_info();
    //Define CpiContext<Transfer>
    let cpi_ctx= CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, reward)?;

    Ok(())
}

#[derive(Accounts)]
pub struct Payout<'info> {
        #[account(mut)]
        pub reward: Account<'info, maths::Reward>,
        #[account(mut)]
        player: Account<'info, player_state::Player>,
        pub sender: Signer<'info>,
        #[account(mut)]
        pub vault_token: Account<'info, TokenAccount>,
        #[account(mut)]
        pub player_token: Account<'info, TokenAccount>,
        #[account(mut)]
        pub days: Account<'info, DaysPassed>,
        pub mint: Account<'info, Mint>,
        pub token_program: Program<'info, Token>,
}
