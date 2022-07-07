use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use crate::constants;
use crate::constants::FEE_LAMPORTS;

pub fn create_player(ctx: Context<InitializePlayer>, rating: Option<i64>) -> Result<()> {
    
    invoke(
        &system_instruction::transfer(ctx.accounts.user.key, ctx.accounts.authority.key, FEE_LAMPORTS),
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    let player = &mut ctx.accounts.player;

    player.claimable = 0;
    player.nft_counter = 1; //account is created when user buys their first nft
    player.identity = ctx.accounts.user.key();

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
    /// CHECK: SAFE PROGRAM OWNED ACCOUNT
    #[account(mut)]
    pub authority: AccountInfo<'info>, //PDA
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
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