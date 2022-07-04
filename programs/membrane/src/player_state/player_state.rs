use anchor_lang::prelude::*;
pub use crate::constants;

pub fn create_player(ctx: Context<InitializePlayer>, rating: Option<i64>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let user = &ctx.accounts.user;
    player.claimable = 0;
    player.nft_counter = 1; //account is created when user buys their first nft
    player.user = *user.key;
    player.bump = *ctx.bumps.get("player_account").unwrap();
    match rating {
        Some(x) => player.rating = Some(x),
        None => player.rating = Some(0),
    };
    
    Ok(())
}

pub fn update_player(ctx: Context<UpdatePlayer>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    player.nft_counter = player.nft_counter + 1; //updated when user buys another nft
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    #[account(init, payer = user, space = constants::MAX_PLAYER_SIZE, seeds = [b"player_account".as_ref(), user.key().as_ref()], bump)]
    pub player: Account<'info, Player>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct UpdatePlayer<'info> {
    #[account(mut, has_one = user, seeds = [b"player_account".as_ref(), user.key().as_ref()], bump = bump)]
    pub player: Account<'info, Player>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[account]
pub struct Player {
    pub user: Pubkey,
    pub bump: u8,
    pub rating: Option<i64>,
    pub claimable: u64,
    nft_counter: u64,
}