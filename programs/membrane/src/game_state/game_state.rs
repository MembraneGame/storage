use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer};
use crate::player_state;
use crate::errors;
use crate::maths;

pub fn initialize_reward(ctx: Context<maths::InitializeReward>) -> Result<()> {
    let reward_account = &mut ctx.accounts.reward;
    reward_account.calculate_reward();
    Ok(())
}

//Fn to pay player
pub fn payout(ctx: Context<Payout>, ctx2: Context<maths::Update>, placement: u8, kills: u8) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let rating_multiplier:f64 = match player.rating {
        Some(0..=100) => 0.8,
        Some(101..=200) => 0.9,
        Some(201..) => 1.0,
        None => return Err(errors::ErrorCode::RatingUndefined.into()),
        _ => return Err(errors::ErrorCode::RatingOverflow.into()),
    };

    let reward_account = &mut ctx2.accounts.reward;
    reward_account.calculate_reward(); //Calculate and update the reward account
    reward_account.reload()?;

    //Define placement_reward based on placement
    let placement_reward = match placement {
        1 => {
            player.rating = Some(player
                .rating
                .unwrap() + 10);
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
            0.0
        },
    };

    let kill_reward = kills as f64 * reward_account.kill;
    let reward = rating_multiplier * (placement_reward + kill_reward);

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
    token::transfer(cpi_ctx, reward as u64)?;

    Ok(())
}


#[derive(Accounts)]
pub struct Payout<'info> {
        #[account(mut)]
        player: Account<'info, player_state::Player>,
        pub sender: Signer<'info>,
        #[account(mut)]
        pub vault_token: Account<'info, TokenAccount>,
        #[account(mut)]
        pub player_token: Account<'info, TokenAccount>,
        pub mint: Account<'info, Mint>,
        pub token_program: Program<'info, Token>,
}
