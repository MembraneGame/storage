use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer, Approve};
use crate::player_state;
use crate::errors;
use crate::maths;
pub use crate::constants;
use crate::constants::VAULT_PDA_SEED;
use player_state::IndStats;

pub fn initialize_reward(ctx: Context<maths::InitializeReward>) -> Result<()> {
    let reward_account = &mut ctx.accounts.reward;
    reward_account.days = 0; //set days to 0
    reward_account.calculate_reward();

    Ok(())
}


pub fn calculate_values(_ctx: Context<RewardValues>, stats: Vec<IndStats>) -> Result<()> {
    
    #[derive(Default)]
    struct Counter {
        vic_count: u64,
        top_five_count: u64,
        top_ten_count: u64,
        kills_count: u64,
    }

    let mut counter: Counter  = Default::default();

    for c in &stats {
        let vic = c.wins * 1000 / c.games;
        let top_five = c.top_five * 1000 / c.games ;
        let top_ten = c.top_ten * 1000 / c.games ;
        let kills = c.kills * 1000 / c.games ;

        counter.vic_count = counter.vic_count + vic;
        counter.top_five_count = counter.top_five_count + top_five;
        counter.top_ten_count = counter.top_ten_count + top_ten;
        counter.kills_count = counter.kills_count + kills;
    }
    
    let _vic_prob = counter.vic_count as f64 / (1000.0 * stats.len() as f64);
    let _top_five_prob = counter.top_five_count as f64 / (1000.0 * stats.len() as f64);
    let _top_ten_prob = counter.top_ten_count as f64 / (1000.0 * stats.len() as f64);
    let _kills_prob = counter.kills_count as f64 / (1000.0 * stats.len() as f64);


    Ok(())
}

pub fn calculate_values_single(ctx: Context<SingleReward>) -> Result<()> { //2 - 100, 2 - 101
    let player = &mut ctx.accounts.player;

    let vic = player.stats.wins as f64 / player.stats.games as f64;
    let top_five = player.stats.top_five as f64/ player.stats.games as f64;
    let top_ten = player.stats.top_ten as f64/ player.stats.games as f64;
    let kills = player.stats.kills as f64/ player.stats.games as f64;
    
    let accumulated = &mut ctx.accounts.stats;

    accumulated.accum_vic += vic;
    accumulated.accum_top_five += top_five;
    accumulated.accum_top_ten += top_ten;
    accumulated.accum_kills += kills;
    accumulated.counter += 1;

    //TODO: add rating league for easier calculation of the avg stats

    Ok(())
}

//Fn to calculate reward at the end of the game and update player account
pub fn calculate_reward(ctx: Context<CalculateReward>, placement: u64, kills: u64, _bump: u8, _identifier: u64) -> Result<()> {
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
    
    //make trait later
    let stat = Stats {
        id: player.identity,
        placement: placement as u8,
        kills: kills as u8,
        survival_duration: 10, //change later not implemented yet
        reward: reward,
    };
    
    let stats = &mut ctx.accounts.players_stats;
    stats.players.push(stat);
    
    //let game = &mut ctx.accounts.game;

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
#[instruction(bump: u8, identifier: u64)]
pub struct CalculateReward<'info> {
        #[account(mut)]
        pub reward: Account<'info, maths::Reward>,
        #[account(mut)]
        player: Account<'info, player_state::Player>,
        #[account(seeds = [b"game".as_ref(), identifier.to_string().as_bytes()], bump = bump)]
        pub game: Account<'info, super::GameStart>,
        /// CHECK: SAFE PROGRAM OWNED ACCOUNT
        #[account(mut, signer)]
        pub pda: AccountInfo<'info>,
        #[account(init_if_needed, seeds = [b"players".as_ref(), identifier.to_string().as_bytes()], bump, payer = pda, space = 10000)]
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

#[derive(Accounts)]
pub struct RewardValues<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
}

#[account]
#[derive(Default)] //not sure if needed
pub struct PlayersStats {
    pub players: Vec<Stats>, //4 + 1472
}

#[derive(Default, AnchorDeserialize, AnchorSerialize, Clone)]
pub struct Stats { //(32 + 1 + 1 + 4 + 8) * 32 = 1472
    pub id: Pubkey, //32
    pub placement: u8, //1
    pub kills: u8, //1
    pub survival_duration: u32, //4
    pub reward: u64, //8
}

#[derive(Accounts)]
pub struct SingleReward<'info> {
    pub player: Account<'info, player_state::Player>,
    #[account(init_if_needed, payer = pda, space = 10000)]
    pub stats: Account<'info, AccumulatedStatistics>,
    /// CHECK: SAFE PROGRAM OWNED ACCOUNT
    #[account(mut, signer)]
    pub pda: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct AccumulatedStatistics {
    pub accum_vic: f64,
    pub accum_top_five: f64,
    pub accum_top_ten: f64,
    pub accum_kills: f64,
    pub counter: u64,
}