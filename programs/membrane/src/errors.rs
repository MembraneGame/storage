use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Player's rating is undefined")]
    RatingUndefined,
    #[msg("The player's rating overflows")]
    RatingOverflow,
    #[msg("Too many players")]
    PlayersOverflow
}