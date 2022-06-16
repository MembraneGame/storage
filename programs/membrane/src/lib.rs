use anchor_lang::prelude::*;

use maths::*;
use game_state::*;
use token_state::*;

pub mod constants;
pub mod errors;
pub mod maths;
pub mod player_state;
pub mod game_state;
pub mod token_state;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod membrane {

    use super::*;

    pub fn initialize_reward(ctx: Context<InitializeReward>) -> Result<()> {
        game_state::initialize_reward(ctx)
    }

    pub fn initialize_mint(ctx:Context<MintInitialize>) -> Result<()> {
        token_state::initialize_mint(ctx)
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        token_state::mint_token(ctx, amount)
    }

    pub fn user_sell<'info>(ctx: Context<InitializeUser>, ctx_burn: Context<BurnToken>, amount: u64) -> Result<()> {
        token_state::user_sell(ctx, ctx_burn, amount)
    }

    pub fn payout<'info>(ctx: Context<Payout>, ctx2: Context<maths::Update>, placement: u8, kills: u8) -> Result<()> {
        game_state::payout(ctx, ctx2, placement, kills)
    }

}


