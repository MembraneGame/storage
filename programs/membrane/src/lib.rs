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

declare_id!("FUbwV7PHj34RaBkifLAkcQ4zdtK6heSWhHVG6qWz5M1o");

#[program]
pub mod membrane {

    use super::*;

    pub fn initialize_nft_multiplier(ctx: Context<InitializeMultiplier>) -> Result<()> {
        game_state::initialize_nft_multiplier(ctx)
    }

    pub fn initialize_reward(ctx: Context<InitializeReward>) -> Result<()> {
        game_state::initialize_reward(ctx)
    }

    pub fn update_nft_multiplier(ctx: Context<UpdateMultiplier>, stats: AvgStats, nfts: NftQualities) -> Result<()> {
        game_state::update_nft_multiplier(ctx, stats, nfts)
    }
    pub fn create_history_account(ctx: Context<CreateHistory>) -> Result<()> {
        game_state::create_history_account(ctx)
    }

    // pub fn start_game(ctx: Context<StartGame>, identifier: u64) -> Result<()> {
    //     game_state::start_game(ctx, identifier)
    // }

    pub fn create_player_stats(ctx: Context<CreatePlayerStats>) -> Result<()> {
        game_state::create_player_stats(ctx)
    }

    // pub fn end_game(ctx: Context<EndGame>, identifier: u64) -> Result<()> {
    //     game_state::end_game(ctx, identifier)
    // }

    pub fn transfer_authority(ctx: Context<TransferAuthority>) -> Result<()> { //transfer authority to mint tokens to PDA
        token_state::transfer_authority(ctx)
    }

    pub fn initialize_player(ctx: Context<InitializePlayer>, rating: Option<i64>) -> Result<()> { //authority is user
        player_state::create_player(ctx, rating)
    }

    pub fn update_player(ctx: Context<UpdatePlayer>, bump: u8) -> Result<()> {
        player_state::update_player(ctx, bump)
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        token_state::mint_token(ctx, amount)
    }

    pub fn user_sell(ctx: Context<SellAndBurn>, amount: u64) -> Result<()> { //signer is user, authority is the storage
        token_state::user_sell(ctx, amount)
    }

    pub fn calculate_reward(ctx: Context<CalculateReward>, placement: u64, kills: u64, identifier: u64) -> Result<()> {
        game_state::calculate_reward(ctx, placement, kills, identifier)
    }

    pub fn user_approve(ctx: Context<UserClaim>) -> Result<()> {
        game_state::user_claim(ctx)
    }

    pub fn user_claim(ctx: Context<UserClaim>) -> Result<()> {
        game_state::user_claim(ctx)
    }

    pub fn freeze_storage(ctx: Context<FreezeStorage>) -> Result<()> {
        token_state::freeze_storage(ctx)
    }

    pub fn return_authority(ctx: Context<ReturnAuthority>) -> Result<()> {
        token_state::return_authority(ctx)
    }

}


