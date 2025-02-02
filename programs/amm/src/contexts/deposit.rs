use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use constant_product_curve::ConstantProduct;

use crate::{errors::AmmError, state::Config};

#[derive(Accounts)]
pub struct Deposit<'Info> {
    #[account(mut)]
    pub lp_provider: Signer<'Info>,
    pub mint_x: Account<'Info, Mint>,
    pub mint_y: Account<'Info, Mint>,
    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'Info, Config>,

    #[account(
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Account<'Info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'Info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'Info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = lp_provider,
    )]
    pub lp_provider_x: Account<'Info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = lp_provider,
    )]
    pub lp_provider_y: Account<'Info, TokenAccount>,

    #[account(
        init_if_needed,
        payer=lp_provider,
        associated_token::mint = mint_lp,
        associated_token::authority = lp_provider,
    )]
    pub lp_provider_lp: Account<'Info, TokenAccount>,

    pub token_program: Program<'Info, Token>,
    pub system_program: Program<'Info, System>,
    pub associated_token: Program<'Info, AssociatedToken>,
}

impl<'Info> Deposit<'Info> {
    pub fn deposit(&mut self, 
        lp_amount: u64, // Amount of LP tokens that the user wants to "claim"
        max_x: u64, // Maximum amount of token X that the user is willing to deposit
        max_y: u64, // Maximum amount of token Y that the user is willing to deposit
    ) -> Result<()> {
        let (x, y) = match self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
            true => (max_x, max_y),
            false => {
                let amount = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    lp_amount,
                    6
                ).unwrap();
                (amounts.x, amounts.y)
            },
        };

        Ok(())
    }
    
    fn deposit_token(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from,to) = 
        match is_x {
            true => (self.lp_provider_x.to_account_info(), self.vault_x.to_account_info()),
            false => (self.lp_provider_y.to_account_info(), self.vault_y.to_account_info()),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_account = Transfer {
            from, // lp_provider_x OR lp_provider_y ?
            to, // vault_x OR vault_y ?
            authority: self.lp_provider.to_account_info(),
        };

        let ctx= CpiContext::new(cpi_program, cpi_account);
        transfer(ctx, amount)?;

        Ok(())
    }

    fn mint_lp_tokens(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_account = MintTo {
            mint: self.mint_lp.to_account_info(), 
            to: self.lp_provider_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let ctx= CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);
        mint_to(ctx, amount);

        Ok(())
    }
}