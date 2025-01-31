use anchor_lang::prelude::*;
use anchor_spl::token_interface;

declare_id!("Fv8pLNSPg3nenVsP1VbKqaPxyj2s47x2jmZLAkG9kV52");

#[error_code]
pub enum SplUtilsError {
    #[msg("Mint and Token program mismatch")]
    MintAndTokenProgramMismatch,
}

#[program]
pub mod spl_utils {
    use super::*;

    pub fn transfer_all_and_close(ctx: Context<TransferAllAndClose>) -> Result<()> {
        handle_transfer_all_and_close(ctx)
    }
}

#[derive(Accounts)]
pub struct TransferAllAndClose<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = owner,
        token::token_program = token_program,
    )]
    pub token_account: InterfaceAccount<'info, token_interface::TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::token_program = token_program,
    )]
    pub transfer_dest: InterfaceAccount<'info, token_interface::TokenAccount>,


    #[account(
        constraint = {
            require!(mint.to_account_info().owner == &token_program.key(), SplUtilsError::MintAndTokenProgramMismatch);
            true
        }
    )]
    pub mint: Box<InterfaceAccount<'info, token_interface::Mint>>,

    /// CHECK: this account can be different from the owner on some gasless transactions
    #[account(mut)]
    pub close_dest: AccountInfo<'info>,

    pub token_program: Interface<'info, token_interface::TokenInterface>,
}

pub fn handle_transfer_all_and_close(ctx: Context<TransferAllAndClose>) -> Result<()> {
    let owner = &ctx.accounts.owner;
    let token_account = &mut ctx.accounts.token_account;
    let mint = &ctx.accounts.mint;
    let token_program = &ctx.accounts.token_program;

    let amount = token_account.amount;

    // Transfer all tokens to the destination account
    token_interface::transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            token_interface::TransferChecked {
                from: token_account.to_account_info(),
                to: ctx.accounts.transfer_dest.to_account_info(),
                authority: owner.to_account_info(),
                mint: mint.to_account_info(),
            },
        ),
        amount,
        mint.decimals,
    )?;

    // Close the account after transferring all tokens
    token_interface::close_account(
        CpiContext::new(
            token_program.to_account_info(),
            token_interface::CloseAccount {
                account: token_account.to_account_info(),
                destination: ctx.accounts.close_dest.to_account_info(),
                authority: owner.to_account_info(),
            },
        ),
    )?;

    Ok(())
}
