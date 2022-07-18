use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint, TokenAccount};


#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    /// CHECK: SAFE PROGRAM OWNED ACCOUNT
    #[account(mut)]
    pub authority: AccountInfo<'info>,
}

// #[derive(Accounts)]
// pub struct MintInitialize<'info> {
//     #[account(mut)]
//     pub mint: Account<'info, Mint>,
//     pub token_program: Program<'info, Token>,
//     #[account(mut)]
//     pub authority: Signer<'info>,
//     pub rent: Sysvar<'info, Rent>,
// }

#[derive(Accounts)]
pub struct SellAndBurn<'info> {
    // /// CHECK: Safe because we don't read or write from the account
    // pub program_signer: AccountInfo<'info>,
    pub player: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub player_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    /// CHECK: SAFE PROGRAM OWNED ACCOUNT
    pub authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(mut)]
    pub storage: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub storage_token_account: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct FreezeStorage<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub storage_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub storage: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReturnAuthority<'info> {
    ///CHECK: Safe, owned by team
    #[account(mut)]
    pub storage: AccountInfo<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub storage_token_account: Account<'info, TokenAccount>,
    ///CHECK: SAFE PROGRAM OWNED ACCOUNT
    #[account(mut)]
    pub pda: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}