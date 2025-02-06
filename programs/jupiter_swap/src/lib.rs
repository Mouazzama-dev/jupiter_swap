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


/// Function to create wSOL token account (if not already created)
fn create_wsol_token_idempotent<'info>(
    program_authority: SystemAccount<'info>,
    program_wsol_account: UncheckedAccount<'info>,
    sol_mint: Account<'info, Mint>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    authority_bump: &[u8],
    wsol_bump: &[u8],
) -> Result<TokenAccount> {
    if program_wsol_account.data_is_empty() {
        let signer_seeds: &[&[&[u8]]] = &[
            &[AUTHORITY_SEED, authority_bump.as_ref()],
            &[WSOL_SEED, wsol_bump.as_ref()],
        ];

        msg!("Initialize program wSOL account");
        let rent = Rent::get()?;
        let space = TokenAccount::LEN;
        let lamports = rent.minimum_balance(space);
        system_program::create_account(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                system_program::CreateAccount {
                    from: program_authority.to_account_info(),
                    to: program_wsol_account.to_account_info(),
                },
                signer_seeds,
            ),
            lamports,
            space as u64,
            token_program.key,
        )?;

        msg!("Initialize program wSOL token account");
        token::initialize_account3(CpiContext::new(
            token_program.to_account_info(),
            token::InitializeAccount3 {
                account: program_wsol_account.to_account_info(),
                mint: sol_mint.to_account_info(),
                authority: program_authority.to_account_info(),
            },
        ))?;

        Ok(TokenAccount::try_from(&program_wsol_account)?)
    } else {
        Ok(TokenAccount::try_from(&program_wsol_account)?)
    }
}

/// Function to close wSOL account after swap
fn close_program_wsol<'info>(
    program_authority: SystemAccount<'info>,
    program_wsol_account: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    authority_bump: &[u8],
) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[AUTHORITY_SEED, authority_bump.as_ref()]];
    token::close_account(CpiContext::new_with_signer(
        token_program.to_account_info(),
        token::CloseAccount {
            account: program_wsol_account.to_account_info(),
            destination: program_authority.to_account_info(),
            authority: program_authority.to_account_info(),
        },
        signer_seeds,
    ))
}


/// Function to transfer acquired meme tokens to user
fn transfer_meme_tokens<'info>(
    program_authority: SystemAccount<'info>,
    meme_token_account: UncheckedAccount<'info>,
    user_account: Signer<'info>,
    token_program: Program<'info, Token>,
    authority_bump: &[u8],
) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[AUTHORITY_SEED, authority_bump.as_ref()]];
    token::transfer(CpiContext::new_with_signer(
        token_program.to_account_info(),
        token::Transfer {
            from: meme_token_account.to_account_info(),
            to: user_account.to_account_info(),
            authority: program_authority.to_account_info(),
        },
        signer_seeds,
    ), 1_000_000)?; // Transfer the acquired meme tokens
    Ok(())
}




#[derive(Accounts)]
pub struct Initialize {}


