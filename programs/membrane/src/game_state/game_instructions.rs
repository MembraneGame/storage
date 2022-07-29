use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer, Approve};
use crate::player_state;
use crate::errors;
use crate::maths;
pub use crate::constants;
use crate::constants::VAULT_PDA_SEED;

pub fn initialize_nft_multiplier(ctx: Context<InitializeMultiplier>) -> Result<()> {
    let nft_multipler = &mut ctx.accounts.nft_multiplier;
    nft_multipler.common = constants::VICTORY; //value at the beginning of the game when no user statistics is available

    Ok(())
}

pub fn initialize_reward(ctx: Context<maths::InitializeReward>) -> Result<()> {
    let reward_account = &mut ctx.accounts.reward;
    let nft_multipler = &mut ctx.accounts.nft_multiplier;

    reward_account.days = 0; //set days to 0
    reward_account.calculate_reward(nft_multipler.common);

    Ok(())
}

//payback is a multiplier of how much a user should receive upon fully exhausting the nft based on its quality (e.g 1.2 for common, 1.5 for epic, 2 for leg)
pub fn update_nft_multiplier(ctx: Context<UpdateMultiplier>, stats: AvgStats, nfts: NftQualities) -> Result<()> { //TODO: Change payback and durability from number to struct for each quality
    let nft_multiplier = &mut ctx.accounts.nft_multiplier;

    let stats_coefficient = stats.league * (0.25 * stats.topfive + 0.1 * stats.topten + stats.victory + 0.0467 * stats.kills);
    let nft_coefficient = (nfts.common.durability as f64) / (nfts.common.payback);

    nft_multiplier.common = (stats_coefficient * nft_coefficient * 10.0_f64.powf(9.0)) as u64;

    Ok(())
}

//Fn to calculate reward at the end of the game and update player account
// pub fn calculate_reward(ctx: Context<CalculateReward>, placement: u64, kills: u64, _identifier: u64) -> Result<()> {
//     let stat = Stats {
//         placement: placement as u8,
//         kills: kills as u8,
//         reward: 1
//     };
//     msg!("Stat: {:?}", stat);
//
//     let stats = &mut ctx.accounts.players_stats.load_mut()?;
//     msg!("PlayersStats pubkey: {}", ctx.accounts.players_stats.key());
//     msg!("Before: {}", stats.counter);
//     msg!("PlayersStats before: {:?}", stats.players[0]);
//     stats.append(stat);
//     msg!("After: {}", stats.counter);
//     msg!("PlayersStats after: {:?}", stats.players);
//
//     Ok(())
// }

pub fn calculate_reward(
    ctx: Context<CalculateReward>,
    placement: u64,
    kills: u64,
    _identifier: u64
) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let rating_multiplier:u64 = match player.rating { //match rating_multiplier
        Some(0..=100) => 8, //values not final
        Some(101..=200) => 9,
        Some(201..) => 10,
        None => return Err(errors::ErrorCode::RatingUndefined.into()),
        _ => return Err(errors::ErrorCode::RatingOverflow.into()),
    };
    let reward_account = &mut ctx.accounts.reward; //define Reward account
    let nft_multiplier = &mut ctx.accounts.nft_multiplier;

    let unix_now = Clock::get().unwrap().unix_timestamp; //current time to compare

    if ((unix_now - constants::START)/constants::SEC_IN_DAY) != reward_account.days { //if statement to check whether next day has begun
        reward_account.calculate_reward(nft_multiplier.common); //Calculate and update the reward account //only common quality at the moment
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

    let stat = Stats {
        id: player.identity,
        placement: placement as u8,
        kills: kills as u8,
        // survival_duration: game.timestamp, //change later not implemented yet
        reward,

    };
    msg!("Reward: {}", reward);
    msg!("Stat: {:?}", stat);

    let stats = &mut ctx.accounts.players_stats.load_mut()?;
    stats.append(stat);

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
pub struct InitializeMultiplier<'info> {
        #[account(init, payer = payer, space = 5000)] //change space after all qualities are added
        pub nft_multiplier: Account<'info, maths::QualityMultiplier>,
        #[account(mut)]
        pub payer: Signer<'info>,
        pub system_program: Program<'info, System>,
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
    #[account(mut)]
    pub players_stats: AccountLoader<'info, PlayersStats>,
    pub nft_multiplier: Account<'info, maths::QualityMultiplier>,
    pub system_program: Program<'info, System>,
}

// #[derive(Accounts)]
// pub struct CalculateReward<'info> {
//     // #[account(mut)]
//     // pub reward: Account<'info, Reward>,
//     #[account(mut)]
//     pub players_stats: AccountLoader<'info, PlayersStats>,
//     #[account(mut)]
//     storage: Signer<'info>,
//     pub system_program: Program<'info, System>,
// }

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

#[account(zero_copy)]
pub struct PlayersStats {
    pub players: [Stats; 32], //4 + 1472
    pub counter: u64, //1
}

impl PlayersStats {
    fn append(&mut self, stat: Stats) {
        self.players[PlayersStats::index_of(self.counter)] = stat;
        self.counter = self.counter + 1;
    }

    fn index_of(counter: u64) -> usize {
        std::convert::TryInto::try_into(counter).unwrap()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct Stats { //(32 + 1 + 1 + 8) * 32 = 1472
    // pub survival_duration: u32, //4
    pub reward: u64, //8
    pub id: Pubkey, //32
    pub placement: u8, //1
    pub kills: u8, //1
}

// #[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
// pub struct Stats { //(32 + 1 + 1 + 8) * 32 = 1472
//     pub placement: u8, //1
//     pub kills: u8, //1
// }

#[derive(Accounts)]
pub struct UpdateMultiplier<'info> {
    #[account(mut)]
    pub nft_multiplier: Account<'info, maths::QualityMultiplier>,
    #[account(mut)]
    pub storage: Signer<'info>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct AvgStats { //shows the chance for placement and avg kills per game
    pub league: f64, //average rating league multiplier
    pub victory: f64, //1
    pub topfive: f64, //2-5
    pub topten: f64, //6-10
    pub kills: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct NftQualities {
    pub common: NftStats,
    //add other qualities
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct NftStats {
    pub durability: u64,
    pub payback: f64,
}
