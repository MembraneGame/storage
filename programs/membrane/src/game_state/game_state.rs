use anchor_lang::prelude::*;
// use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer, Approve};
// use crate::player_state;
use crate::errors;
// use crate::maths;
pub use crate::constants;
use super::Game;

pub fn start_game(ctx: Context<StartGame>, identifier: u64, players: u8) -> Result<()> {

    if players > constants::MAX_PLAYERS as u8 {
        return Err(errors::ErrorCode::PlayersOverflow.into())
    }

    let game = &mut ctx.accounts.game;
    game.start_timestamp = Clock::get().unwrap().unix_timestamp;
    game.identifier = identifier;

    Ok(())
}

pub fn end_game(ctx: Context<EndGame>, _identifier: u64, _bump: u8) -> Result<()> {
    let game = &mut ctx.accounts.game;
    game.end_timestamp = Clock::get().unwrap().unix_timestamp;
    
    Ok(())
}


#[derive(Accounts)]
#[instruction(identifier: u64, players: u8)]
pub struct StartGame<'info> {
    #[account(init, seeds = [b"game".as_ref(), identifier.to_string().as_bytes()],
    bump, 
    payer = storage, 
    space = players as usize*Game::STATS_LEN + Game::MISC_LEN)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub storage: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(identifier: u64, bump: u8)]
pub struct EndGame<'info> {
    #[account(mut)]
    pub storage: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut, seeds = [b"game".as_ref(), identifier.to_string().as_bytes()], bump = bump)]
    pub game: Account<'info, Game>,
}
