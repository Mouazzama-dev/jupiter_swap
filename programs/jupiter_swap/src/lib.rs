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

        /*Approach used for remaining accounts is to be preferred because it allows us to use dynamic meme coins instead of hardcoding them wrt 
        their length as well as their selecection */
        msg!("Swapping SOL into three different meme tokens via Jupiter...");
        for i in 0..3 {
            //The remaining_accounts array allows users to pass any arbitrary set of accounts during program execution.
            let meme_coin = ctx.remaining_accounts[i].clone(); // Meme token mint accounts
            swap_on_jupiter(
                &ctx.remaining_accounts,
                ctx.accounts.jupiter_program.clone(),
                data.clone(),
                sol_split,
                meme_coin,
            )?;
        }
        /*Wrapped SOL is used only for the Jupiter swap because Jupiter supports token swaps (not native SOL). 
        Once the swaps are complete, the wSOL account is no longer needed.*/
        msg!("Closing wSOL account...");
        close_program_wsol(
            ctx.accounts.program_authority.clone(),
            ctx.accounts.program_wsol_account.clone(),
            ctx.accounts.token_program.clone(),
            &authority_bump,
        )?;

        /* The final step was to trasfer that swapped meme Tokens to the user */
        msg!("Swaps complete! Transferring meme tokens to user...");
        for i in 0..3 {
            let meme_token_account = ctx.remaining_accounts[i + 3].clone(); // Meme token accounts
            transfer_meme_tokens(
                ctx.accounts.program_authority.clone(),
                meme_token_account,
                ctx.accounts.user_account.clone(),
                ctx.accounts.token_program.clone(),
                &authority_bump,
            )?;
        }

        Ok(())
    }
}

/// Function to execute the swap on the Jupiter Aggregator
fn swap_on_jupiter<'info>(
    remaining_accounts: &[AccountInfo],
    jupiter_program: Program<'info, Jupiter>,
    data: Vec<u8>,
    amount: u64,
    meme_coin: AccountInfo<'info>,
) -> ProgramResult {
    // Create AccountMeta list for Jupiter's instruction
    let accounts: Vec<AccountMeta> = remaining_accounts
        .iter()
        .map(|acc| AccountMeta {
            pubkey: *acc.key,
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    // Clone AccountInfo list
    let accounts_infos: Vec<AccountInfo> = remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    // Call Jupiter swap program
    invoke_signed(
        &Instruction {
            program_id: *jupiter_program.key,
            accounts,
            data,
        },
        &accounts_infos,
        &[],
    )
}



#[derive(Accounts)]
pub struct Initialize {}


