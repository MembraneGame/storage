use anchor_lang::prelude::*;
pub use crate::constants;

pub fn create_player(ctx: Context<InitializePlayer>, rating: Option<i64>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let user = &ctx.accounts.user;
    player.claimable = 0;

    player.user = *user.key;
    match rating {
        Some(x) => player.rating = Some(x),
        None => player.rating = Some(0),
    };
    
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    #[account(init, payer = user, space = constants::MAX_PLAYER_SIZE)]
    pub player: Account<'info, Player>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[account]
pub struct Player {
    pub user: Pubkey,
    pub rating: Option<i64>,
    pub claimable: u64,
}