use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::state::Config;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'Info> {
    #[account(mut)]
    pub initializer: Signer<'Info>,
    pub mint_x: Account<'Info, Mint>,
    pub mint_y: Account<'Info, Mint>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"lp", config.key.as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Account<'Info, Mint>,

    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'Info, TokenAccount>,

    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'Info, TokenAccount>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"config", seed.to_le_bytes().as_ref()],
        bump,
        space = Config::INIT_SPACE,
    )]
    pub config: Account<'Info, Config>,

    pub token_program: Program<'Info, Token>,
    pub associated_token: Program<'Info, AssociatedToken>,
    pub system_program: Program<'Info, System>
}

impl<'Info> Initialize<'Info> {
    pub fn init(
        &mut self,
        seed: u64,
        fee: u16,
        authority: Option<Pubkey>, // you can pass Some|Pubkey or None
        bumps: &InitializeBumps,
    ) -> Result<()> {
        self.config.set_inner(Config {
            seed,
            authority,
            mint_x: self.mint_x.key(),
            mint_y: self.mint_y.key(),
            fee,
            locked: false,
            config_bump: bumps.,
            
        })
    }
}