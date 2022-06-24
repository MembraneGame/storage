use anchor_lang::prelude::*;
use maths::*;
use game_state::*;
use token_state::*;
use player_state::*;
pub mod constants;
pub mod errors;
pub mod maths;
pub mod player_state;
pub mod game_state;
pub mod token_state;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod membrane {

    //authority is always storage unless specified otherwise
    use super::*;

    pub fn initialize_reward(ctx: Context<InitializeReward>) -> Result<()> {
        game_state::initialize_reward(ctx)
    }

    pub fn initialize_mint(ctx:Context<MintInitialize>) -> Result<()> {
        token_state::initialize_mint(ctx)
    }

    pub fn initialize_player(ctx: Context<InitializePlayer>) -> Result<()> { //authority is user
        player_state::create_player(ctx)
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        token_state::mint_token(ctx, amount)
    }

    pub fn user_sell(ctx: Context<SellAndBurn>, amount: u64) -> Result<()> { //authority is user, NOT the storage
        token_state::user_sell(ctx, amount)
    }

    pub fn payout(ctx: Context<Payout>, placement: u64, kills: u64) -> Result<()> {
        game_state::payout(ctx, placement, kills)
    }

    // pub fn burn_token(ctx_burn: Context<BurnToken>, amount:u64) -> Result<()> {
    //     token_state::burn_token(ctx_burn, amount)
    // }

}


