use anchor_lang::prelude::*;
// use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer, Approve};
// use crate::player_state;
// use crate::errors;
// use crate::maths;
pub use crate::constants::*;
use super::{Stats, PlayersStats};

pub fn start_game(ctx: Context<StartGame>, _identifier: u64) -> Result<()> {
    let game = &mut ctx.accounts.game;
    game.timestamp = Clock::get().unwrap().unix_timestamp;

    Ok(())
}

pub fn create_player_stats(_ctx: Context<CreatePlayerStats>) -> Result<()> {
    Ok(())
}

pub fn create_history_account(_ctx: Context<CreateHistory>) -> Result<()> {    
    Ok(())
}


pub fn end_game(ctx: Context<EndGame>, identifier: u64) -> Result<()> {
    let history = &mut ctx.accounts.history.load_mut()?;
    let stats = &mut ctx.accounts.players_stats.load_mut()?;
    let counter = history.counter; //value declared explicitly to avoid null pointer
    let unix = Clock::get().unwrap().unix_timestamp;
    let duration = unix - ctx.accounts.game.timestamp;

    let game = Game {
        identifier: identifier,
        duration: duration as u64,
        player_stats: stats.players.clone(),
    };
    
    history.games[counter] = game;
    history.counter += 1;

    if counter == 7000 {
        history.timestamp = unix;
    }
    
    Ok(())
}


#[derive(Accounts)]
#[instruction(identifier: u64)]
pub struct StartGame<'info> {
    #[account(init, seeds = [b"game".as_ref(), identifier.to_string().as_bytes()], bump, payer = storage, space = 10000)] //space will be calculated later
    pub game: Account<'info, GameStart>,
    /// CHECK: SAFE PROGRAM OWNED ACCOUNT
    #[account(mut)]
    pub storage: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(identifier: u64)]
pub struct EndGame<'info> {
    #[account(mut, seeds = [b"game".as_ref(), identifier.to_string().as_bytes()], bump, close = storage)]
    pub game: Account<'info, GameStart>,
    /// CHECK: SAFE PROGRAM OWNED ACCOUNT
    #[account(mut)]
    pub storage: Signer<'info>,
    #[account(mut)]
    history: AccountLoader<'info, History>,
    pub system_program: Program<'info, System>,
    #[account(mut, close = storage)]
    pub players_stats: AccountLoader<'info, PlayersStats>,
    //pub players_acc: Vec<Account<'info, player_state::Player>>,
}

#[derive(Accounts)]
pub struct CreateHistory<'info> {
    #[account(zero)]
    history: AccountLoader<'info, History>,
}

#[derive(Accounts)]
pub struct CreatePlayerStats<'info> {
    #[account(zero)]
    player_stats: AccountLoader<'info, PlayersStats>,
}


#[account(zero_copy)]
pub struct History { //one game is 1492 bytes // 7028 games for accout size overflow (per epoch)
    pub games: [Game; 7000], //1492 + 4
    pub timestamp: i64,
    pub counter: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct Game {
    pub identifier: u64, //8
    pub duration: u64, //8 
    pub player_stats: [Stats; 32], //1476
}

#[account]
pub struct GameStart {
    pub timestamp: i64, //8
}

