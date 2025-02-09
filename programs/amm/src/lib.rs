use anchor_lang::prelude::*;

pub mod contexts;
pub mod state;
pub mod errors;

use crate::contexts::*;
use crate::state::*;
use crate::errors::*;

declare_id!("Eg3aKa1woWAMEpohUQ4SUa5yEWGeXKbaC6YSLfALFTkU");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    
}

