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

pub fn end_game(ctx: Context<EndGame>, identifier: u64, _bump: u8) -> Result<()> {
    let stats = &mut ctx.accounts.players_stats;
    let unix = Clock::get().unwrap().unix_timestamp;
    let duration = unix - ctx.accounts.game.timestamp;

    stats.duration = duration as u64;
    stats.identifier = identifier;
    
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
#[instruction(identifier: u64, bump: u8)]
pub struct EndGame<'info> {
    #[account(mut, seeds = [b"game".as_ref(), identifier.to_string().as_bytes()], bump, close = storage)]
    pub game: Account<'info, GameStart>,
    /// CHECK: SAFE PROGRAM OWNED ACCOUNT
    #[account(mut)]
    pub storage: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut, seeds = [identifier.to_string().as_bytes()], bump = bump)]
    pub players_stats: Account<'info, PlayersStats>,
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

