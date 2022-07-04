use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer, Approve, Revoke};
use crate::player_state;
use crate::errors;
use crate::maths;
pub use crate::constants;

pub fn initialize_reward(ctx: Context<maths::InitializeReward>) -> Result<()> {
    let reward_account = &mut ctx.accounts.reward;
    reward_account.days = 0; //set days to 0
    reward_account.calculate_reward();
    Ok(())
}

//Fn to pay player
pub fn payout(ctx: Context<Payout>, placement: u64, kills: u64) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let rating_multiplier:u64 = match player.rating { //match rating_multiplier
        Some(0..=100) => 8, //values not final
        Some(101..=200) => 9,
        Some(201..) => 10,
        None => return Err(errors::ErrorCode::RatingUndefined.into()),
        _ => return Err(errors::ErrorCode::RatingOverflow.into()),
    };
    let reward_account = &mut ctx.accounts.reward; //define Reward account

    let unix_now = Clock::get().unwrap().unix_timestamp; //current time to compare

    if ((unix_now - constants::START)/constants::SEC_IN_DAY) != reward_account.days { //if statement to check whether next day has begun
        reward_account.calculate_reward(); //Calculate and update the reward account
        reward_account.reload()?; //update the reward account if new day begun
        reward_account.days = (unix_now - constants::START)/constants::SEC_IN_DAY;
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
    let reward = (rating_multiplier * (placement_reward + kill_reward))/10; //calculate total reward
    player.claimable = player.claimable + reward;

    // //Define Transfer account
    // let cpi_accounts = Transfer {
    //     from: ctx
    //     .accounts
    //     .vault_token
    //     .to_account_info(), 

    //     to: ctx
    //     .accounts
    //     .player_token
    //     .to_account_info(), 

    //     authority: ctx
    //     .accounts
    //     .sender
    //     .to_account_info(), 
    // };
    // //Define token program
    // let cpi_program = ctx.accounts.token_program.to_account_info();
    // //Define CpiContext<Transfer>
    // let cpi_ctx= CpiContext::new(cpi_program, cpi_accounts);
    // token::transfer(cpi_ctx, reward)?;

    Ok(())
}

pub fn user_approve(ctx: Context<PlayerApprove>) -> Result<()> {
    let player = &mut ctx.accounts.player;

    //Define Approve account
    let cpi_accounts = Approve {
        delegate: ctx
        .accounts
        .user
        .to_account_info(), 

        to: ctx
        .accounts
        .player_token
        .to_account_info(), 

        authority: ctx
        .accounts
        .authority
        .to_account_info(), 
    };
    //Define token program
    let cpi_program = ctx.accounts.token_program.to_account_info();
    //Define CpiContext<Approve>
    let cpi_ctx= CpiContext::new(cpi_program, cpi_accounts);
    token::approve(cpi_ctx, player.claimable)?;

    //not sure if revoke is necessary, since solana can automatically change the delegated_amount
    // //Define Revoke account
    // let cpi_accounts = Revoke {
    //     source: ctx
    //     .accounts
    //     .user
    //     .to_account_info(), 

    //     authority: ctx
    //     .accounts
    //     .authority
    //     .to_account_info(), 
    // };
    // let cpi_program = ctx.accounts.token_program.to_account_info();
    // let cpi_ctx= CpiContext::new(cpi_program, cpi_accounts);
    // token::revoke(cpi_ctx)?;
    
    Ok(())
}


pub fn user_claim(ctx: Context<PlayerClaim>) -> Result<()> {
    
    let player = &mut ctx.accounts.player;

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
        .user
        .to_account_info(), 
    };

    //Define token program
    let cpi_program = ctx.accounts.token_program.to_account_info();
    //Define CpiContext<Transfer>
    let cpi_ctx= CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, player.claimable)?;

    player.claimable = 0;
    Ok(())
}

#[derive(Accounts)]
pub struct Payout<'info> {
        #[account(mut)]
        pub reward: Account<'info, maths::Reward>,
        #[account(mut)]
        player: Account<'info, player_state::Player>,
        // pub sender: Signer<'info>,
        // #[account(mut)]
        // pub vault_token: Account<'info, TokenAccount>,
        // #[account(mut)]
        // pub player_token: Account<'info, TokenAccount>,
        // pub mint: Account<'info, Mint>,
        // pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PlayerApprove<'info> {
        #[account(mut)]
        pub player: Account<'info, player_state::Player>,
        ///CHECK: Safe because we do not read or write from the account
        pub user: UncheckedAccount<'info>,
        pub authority: Signer<'info>, //authority is storage
        #[account(mut)]
        pub vault_token: Account<'info, TokenAccount>,
        #[account(mut)]
        pub player_token: Account<'info, TokenAccount>,
        pub mint: Account<'info, Mint>,
        pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PlayerClaim<'info> {
        #[account(mut)]
        pub player: Account<'info, player_state::Player>,
        pub user: Signer<'info>,
        pub authority: Signer<'info>,
        #[account(mut)]
        pub vault_token: Account<'info, TokenAccount>,
        #[account(mut)]
        pub player_token: Account<'info, TokenAccount>,
        pub mint: Account<'info, Mint>,
        pub token_program: Program<'info, Token>,
}