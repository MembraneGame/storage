use anchor_lang::prelude::*;

#[account]
pub struct Player {
    pub user: Pubkey,
    pub rating: Option<i64>,
}