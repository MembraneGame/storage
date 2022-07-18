use anchor_lang::prelude::*;
// use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer, Approve};
// use crate::player_state;
// use crate::errors;
// use crate::maths;
pub use crate::constants;
// use crate::constants::VAULT_PDA_SEED;

pub fn start_game(ctx: Context<StartGame>, _identifier: u64) -> Result<()> {
    let game = &mut ctx.accounts.game;
    game.timestamp = Clock::get().unwrap().unix_timestamp;

    Ok(())
}

pub fn end_game(ctx: Context<EndGame>, _epoch: u64, identifier: u64, _bump_players : u8) -> Result<()> {
    let history = &mut ctx.accounts.history;
    
    let unix = Clock::get().unwrap().unix_timestamp;
    let duration = unix - ctx.accounts.game.timestamp;

    let game = Game {
        identifier: identifier,
        duration: duration as u64,
        player_stats: ctx.accounts.players_stats.players.clone(),
    };

    history.games.push(game);
    
    Ok(())
}


#[derive(Accounts)]
#[instruction(identifier: u64)]
pub struct StartGame<'info> {
    #[account(init, seeds = [b"game".as_ref(), identifier.to_string().as_bytes()], bump, payer = pda, space = 10000)] //space will be calculated later
    pub game: Account<'info, GameStart>,
    /// CHECK: SAFE PROGRAM OWNED ACCOUNT
    #[account(mut, signer)]
    pub pda: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(epoch: u64, bump_players: u8, identifier: u64)]
pub struct EndGame<'info> {
    #[account(mut, seeds = [b"game".as_ref(), identifier.to_string().as_bytes()], bump, close = pda)]
    pub game: Account<'info, GameStart>,
    /// CHECK: SAFE PROGRAM OWNED ACCOUNT
    #[account(mut)]
    pub pda: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [b"epoch".as_ref(), epoch.to_string().as_bytes()], bump, payer = pda, space = 10485760)] //max space
    pub history: Account<'info, History>,
    pub system_program: Program<'info, System>,
    #[account(mut, seeds = [b"players".as_ref(), identifier.to_string().as_bytes()], bump, close = pda)]
    pub players_stats: Account<'info, super::PlayersStats>,
    //pub players_acc: Vec<Account<'info, player_state::Player>>,
}

#[account]
#[derive(Default)]
pub struct History { //one game is 1492 bytes // 7028 games for accout size overflow (per epoch)
    pub games: Vec<Game>, //1492 + 4
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct Game {
    pub identifier: u64, //8
    pub duration: u64, //8 
    pub player_stats: Vec<super::Stats>, //1476
}

#[account]
pub struct GameStart {
    pub timestamp: i64, //8
}

