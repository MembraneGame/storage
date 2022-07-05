use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Burn, Transfer, SetAuthority};
use super::{MintToken, SellAndBurn, TransferAuthority};
use crate::constants::*;
pub use spl_token;

//Fn to initialize mint
// pub fn initialize_mint(ctx:Context<MintInitialize>) -> Result<()> {
//     let cpi_accounts = InitializeMint {
//         mint: ctx.accounts.mint.to_account_info(),
//         rent: ctx.accounts.rent.to_account_info(),
//     };
//     let cpi_program = ctx.accounts.token_program.to_account_info();
//     let cpi_ctx= CpiContext::new(cpi_program, cpi_accounts);
//     token::initialize_mint(cpi_ctx, 9, ctx.accounts.authority.key, Some(ctx.accounts.authority.key))?;
//     Ok(())
// }

pub fn transfer_authority(ctx: Context<TransferAuthority>) -> Result<()> {
    let cpi_accounts = SetAuthority {
        current_authority: ctx.accounts.storage.to_account_info(),
        account_or_mint: ctx.accounts.mint.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    let (pda_authority, _bump) = Pubkey::find_program_address(&[VAULT_PDA_SEED], ctx.program_id);    
    token::set_authority(cpi_ctx, spl_token::instruction::AuthorityType::MintTokens, Some(pda_authority))?;
    
    let cpi_accounts = SetAuthority {
        current_authority: ctx.accounts.storage.to_account_info(),
        account_or_mint: ctx.accounts.storage_token_account.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::set_authority(cpi_ctx, spl_token::instruction::AuthorityType::AccountOwner, Some(pda_authority))?;

    Ok(())
}

//Fn to mint tokens to ATA
pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {

    let (_vault_authority, vault_authority_bump) = Pubkey::find_program_address(&[VAULT_PDA_SEED], ctx.program_id);
    let authority_seeds = &[&VAULT_PDA_SEED[..], &[vault_authority_bump]];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_program = ctx
    .accounts
    .token_program
    .to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::mint_to(cpi_ctx.with_signer(&[&authority_seeds[..]]), amount)?;
    Ok(())
}

//Fn when user sells tokens back to the storage
pub fn user_sell(ctx: Context<SellAndBurn>, amount: u64) -> Result<()> { //signer is user, authority is the storage, amount in 10^9

    let (_vault_authority, vault_authority_bump) = Pubkey::find_program_address(&[VAULT_PDA_SEED], ctx.program_id);
    let authority_seeds = &[&VAULT_PDA_SEED[..], &[vault_authority_bump]];

    //Define Transfer account
    let cpi_accounts = Transfer {
        from: ctx
        .accounts
        .player_token
        .to_account_info(),

        to: ctx
        .accounts
        .vault_token
        .to_account_info(),

        authority: ctx
        .accounts
        .player
        .to_account_info(),
    };

    //Define Transfer token program
    let cpi_program = ctx
    .accounts
    .token_program
    .to_account_info();

    //Define CpiContext<Tranfer>
    let cpi_ctx= CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    //Define Burn account
    let cpi_burn_accounts = Burn {
        mint: ctx
        .accounts
        .mint
        .to_account_info(),

        from: ctx
        .accounts
        .vault_token
        .to_account_info(),

        authority: ctx
        .accounts
        .authority
        .to_account_info(),
    };

    //Define burn token program
    let cpi_burn_program = ctx
    .accounts
    .token_program
    .to_account_info();

    //Define CpiContext<Burn>
    let cpi_burn_ctx = CpiContext::new(cpi_burn_program, cpi_burn_accounts);
    token::burn(cpi_burn_ctx.with_signer(&[&authority_seeds[..]]), amount/2)?;

    Ok(())
}

// pub fn burn_token(ctx_burn: Context<BurnToken>, amount:u64) -> Result<()> {
//         //Define Burn account
//         let cpi_burn_accounts = Burn {
//             mint: ctx_burn
//             .accounts
//             .mint
//             .to_account_info(),

//             from: ctx_burn
//             .accounts
//             .vault_token
//             .to_account_info(),

//             authority: ctx_burn
//             .accounts
//             .authority
//             .to_account_info(),
//         };

//         //Define burn token program
//         let cpi_burn_program = ctx_burn
//         .accounts
//         .token_program
//         .to_account_info();

//         //Define CpiContext<Burn>
//         let cpi_burn_ctx = CpiContext::new(cpi_burn_program, cpi_burn_accounts);
//         token::burn(cpi_burn_ctx, amount)?;

//         Ok(())
// }
