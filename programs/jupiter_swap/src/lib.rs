use anchor_lang::prelude::*;

declare_id!("71gmhF6PWXzKV1xhkv4DmQikAhCL5zXSQGccLwurzZd5");

#[program]
pub mod jupiter_swap {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
