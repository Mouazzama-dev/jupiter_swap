use anchor_lang::prelude::*;
use anchor_lang::{
    prelude::*,
    solana_program::{entrypoint::ProgramResult, instruction::Instruction, program::invoke_signed},
    system_program,
};
use anchor_spl::token::{self, Mint, Token, TokenAccount};

pub const AUTHORITY_SEED: &[u8] = b"authority";
pub const WSOL_SEED: &[u8] = b"wsol";

mod jupiter {
    use anchor_lang::declare_id;
    declare_id!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"); // Jupiter Aggregator ID
}

#[derive(Clone)]
pub struct Jupiter;

impl anchor_lang::Id for Jupiter {
    fn id() -> Pubkey {
        jupiter::id()
    }
}

#[error_code]
pub enum ErrorCode {
    InvalidReturnData,
    InvalidJupiterProgram,
    IncorrectOwner,
    SwapFailed,
}



declare_id!("71gmhF6PWXzKV1xhkv4DmQikAhCL5zXSQGccLwurzZd5");

#[program]
pub mod jupiter_swap {
    use super::*;

    pub fn swap_sol_to_memes(ctx: Context<SolToMemeSwap>, data: Vec<u8>) -> Result<()> {
        let authority_bump = ctx.bumps.get("program_authority").unwrap().to_le_bytes();
        let wsol_bump = ctx.bumps.get("program_wsol_account").unwrap().to_le_bytes();

        let total_sol = ctx.accounts.user_account.to_account_info().lamports();
        require!(total_sol > 0, ErrorCode::SwapFailed);

        // Split SOL into 3 equal parts
        let sol_split = total_sol / 3; 

        //Jupiter Aggregator only supports token swaps when the SOL is wrapped into wSOL (wrapped SOL) to participate in these swaps.
        msg!("Creating wSOL account...");
        create_wsol_token_idempotent(
            ctx.accounts.program_authority.clone(),
            ctx.accounts.program_wsol_account.clone(),
            ctx.accounts.sol_mint.clone(),
            ctx.accounts.token_program.clone(),
            ctx.accounts.system_program.clone(),
            &authority_bump,
            &wsol_bump,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}


