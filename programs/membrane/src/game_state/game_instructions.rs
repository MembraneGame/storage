use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer, Approve};
use crate::player_state;
use crate::errors;
use crate::maths;
pub use crate::constants;
use crate::constants::VAULT_PDA_SEED;

pub fn initialize_reward(ctx: Context<maths::InitializeReward>) -> Result<()> {
    let reward_account = &mut ctx.accounts.reward;
    let nft_multipler = &mut ctx.accounts.nft_multiplier;
    nft_multipler.common = constants::VICTORY;

    reward_account.days = 0; //set days to 0
    reward_account.calculate_reward(nft_multipler.common);

    Ok(())
}


//Fn to calculate reward at the end of the game and update player account
pub fn calculate_reward(ctx: Context<CalculateReward>, placement: u64, kills: u64, _identifier: u64) -> Result<()> {
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
        reward_account.calculate_reward(constants::VICTORY); //Calculate and update the reward account
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
    // let game = &ctx.accounts.game;

    
    
    //make trait later
    let stat = Stats {
        id: player.identity,
        placement: placement as u8,
        kills: kills as u8,
        // survival_duration: game.timestamp, //change later not implemented yet
        reward: reward,
    };
    
    let stats = &mut ctx.accounts.players_stats;
    stats.players.push(stat);
    


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
#[instruction(placement: u64, kills: u64, identifier: u64)]
pub struct CalculateReward<'info> {
        #[account(mut)]
        pub reward: Account<'info, maths::Reward>,
        #[account(mut)]
        player: Account<'info, player_state::Player>,
        #[account(mut)]
        pub storage: Signer<'info>,
        #[account(init_if_needed, seeds = [b"players".as_ref(), identifier.to_string().as_bytes()], bump, payer = storage, space = 10000)]
        pub players_stats: Account<'info, PlayersStats>,
        pub system_program: Program<'info, System>,
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

#[account]
pub struct PlayersStats {
    pub players: Vec<Stats>, //4 + 1472
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct Stats { //(32 + 1 + 1 + 8) * 32 = 1472
    pub id: Pubkey, //32
    pub placement: u8, //1
    pub kills: u8, //1
    // pub survival_duration: u32, //4
    pub reward: u64, //8
}
