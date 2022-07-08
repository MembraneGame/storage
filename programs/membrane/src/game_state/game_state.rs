use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer, Approve};
use crate::player_state;
use crate::errors;
use crate::maths;
pub use crate::constants;
use crate::constants::VAULT_PDA_SEED;

pub fn initialize_reward(ctx: Context<maths::InitializeReward>) -> Result<()> {
    let reward_account = &mut ctx.accounts.reward;
    reward_account.days = 0; //set days to 0
    reward_account.calculate_reward();
    Ok(())
}

//Fn to calculate reward at the end of the game and update player account
pub fn calculate_reward(ctx: Context<CalculateReward>, placement: u64, kills: u64) -> Result<()> {
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

//Fn to delegate claimable token to the user
pub fn user_claim(ctx: Context<UserClaim>) -> Result<()> {
    let (_vault_authority, vault_authority_bump) = Pubkey::find_program_address(&[VAULT_PDA_SEED], ctx.program_id);
    let authority_seeds = &[&VAULT_PDA_SEED[..], &[vault_authority_bump]];
    let seeds = &[&authority_seeds[..]];

    let player = &mut ctx.accounts.player;

    //Define Approve account
    let cpi_accounts = Approve {
        delegate: ctx
        .accounts
        .user
        .to_account_info(), 

        to: ctx
        .accounts
        .vault_token
        .to_account_info(), 

        authority: ctx
        .accounts
        .authority
        .to_account_info(), 
    };
    //Define token program
    let cpi_program = ctx.accounts.token_program.to_account_info();
    //Define CpiContext<Approve> with PDA signer seeds
    let cpi_ctx= CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    token::approve(cpi_ctx, player.claimable)?;

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

    //not sure if revoke is necessary, since solana can automatically change the delegated_amount, test needed
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

#[derive(Accounts)]
pub struct CalculateReward<'info> {
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
pub struct UserClaim<'info> {
        #[account(mut)]
        pub player: Account<'info, player_state::Player>,
        #[account(mut)]
        pub user: Signer<'info>,
        /// CHECK: SAFE PROGRAM OWNED ACCOUNT
        #[account(mut)]
        pub authority: AccountInfo<'info>, //PDA
        #[account(mut)]
        pub vault_token: Account<'info, TokenAccount>,
        #[account(mut)]
        pub player_token: Account<'info, TokenAccount>,
        pub mint: Account<'info, Mint>,
        pub token_program: Program<'info, Token>,
}
