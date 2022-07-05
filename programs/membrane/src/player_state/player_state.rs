use anchor_lang::prelude::*;
pub use crate::constants;

pub fn create_player(ctx: Context<InitializePlayer>, rating: Option<i64>) -> Result<()> {
    let player = &mut ctx.accounts.player;

    player.claimable = 0;
    player.nft_counter = 1; //account is created when user buys their first nft
    player.identity = ctx.accounts.identity.key();

    match rating {
        Some(x) => player.rating = Some(x),
        None => player.rating = Some(0),
    };
    
    Ok(())
}

pub fn update_player(ctx: Context<UpdatePlayer>, _bump: u8) -> Result<()> {
    let player = &mut ctx.accounts.player;
    player.nft_counter = player.nft_counter + 1; //updated when user buys another nft
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    #[account(init, payer = user, space = constants::MAX_PLAYER_SIZE, seeds = [b"player".as_ref(), user.key().as_ref()], bump)]
    pub player: Box<Account<'info, Player>>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub identity: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct UpdatePlayer<'info> {
    #[account(mut, has_one = identity, seeds = [b"player".as_ref(), identity.key().as_ref()], bump = bump)]
    pub player: Box<Account<'info, Player>>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub identity: Signer<'info>,
}


#[account]
pub struct Player {
    pub identity: Pubkey,
    //pub bump: u8,
    pub rating: Option<i64>,
    pub claimable: u64,
    nft_counter: u64,
}