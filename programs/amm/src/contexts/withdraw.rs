use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token::{burn, mint_to, transfer, Burn, Mint, MintTo, Token, TokenAccount, Transfer},
};

use constant_product_curve::ConstantProduct;

use crate::{errors::AmmError, state::Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub lp_provider: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [b"lp", config.key().as_ref()],
        mint::decimals = 6,
        mint::authority = config,
        bump = config.lp_bump,
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = lp_provider,
    )]
    pub lp_provider_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = lp_provider,
    )]
    pub lp_provider_y: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = lp_provider,
        associated_token::mint = mint_lp,
        associated_token::authority = lp_provider,
    )]
    pub lp_provider_lp: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, 
        lp_amount: u64, // Amount of LP tokens that the user wants to "burn"
        min_x: u64,  // Minimum amount of token X that the user wants to receive
        min_y: u64,  // Minimum amount of token Y that the user wants to receive
    ) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(lp_amount != 0, AmmError::InvalidAmount);
        require!(min_x != 0 || min_y != 0, AmmError::InvalidAmount);
        
        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            lp_amount,
    6
        ).map_err(AmmError::from)?;

        require!(min_x <= amounts.x && min_y <= amounts.y, AmmError::SlippageExceeded);

        // deposit token x
        self.withdraw_tokens(true, amounts.x)?;
        // deposit token y
        self.withdraw_tokens(false, amounts.y)?;
        // mint lp tokens
        self.burn_lp_tokens(lp_amount);

        Ok(())
    }
    
    fn withdraw_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from,to) = 
        match is_x {
            true => (self.vault_x.to_account_info(), self.lp_provider_x.to_account_info()),
            false => (self.vault_y.to_account_info(), self.lp_provider_y.to_account_info()),
        };

        let cpi_program = self.token_program.to_account_info();
        
        let cpi_accounts = Transfer {
            from, // vault_x OR vault_y ?
            to, // lp_provider_x OR lp_provider_y ?
            authority: self.config.to_account_info(),
        };

        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_context= CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_context, amount)?;

        Ok(())
    }

    fn burn_lp_tokens(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(), 
            from: self.lp_provider_lp.to_account_info(),
            authority: self.lp_provider.to_account_info(),
        };

        let cpi_contxt = CpiContext::new(cpi_program, cpi_accounts);
        burn(cpi_contxt, amount);

        Ok(())
    }
}