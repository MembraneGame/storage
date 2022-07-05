use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint, TokenAccount};


#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintInitialize<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
}

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
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    pub storage: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}


// #[derive(Accounts)]
// pub struct BurnToken<'info> {
//     pub mint: Account<'info, Mint>,
//     #[account(mut)]
//     pub vault_token: Account<'info, TokenAccount>,
//     /// CHECK: Safe because we don't read or write from the account
//     pub authority: AccountInfo<'info>,
//     pub token_program: Program<'info, Token>,
// }