use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use std::collections::HashMap;

declare_id!("56pCh7Zmmnb8UFoincPhC2U7ABKwpKBTLPQAWYFbUpSV");

#[program]
pub mod asset_vault {
    use super::*;

    pub fn setup_vault(ctx: Context<SetupVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault_account;
        vault.depositor_balances = HashMap::new(); // Initialize the HashMap
        msg!("Vault setup completed.");
        Ok(())
    }

    pub fn add_funds(ctx: Context<FundVault>, deposit_amount: u64) -> Result<()> {
        msg!("Depositing {} tokens", deposit_amount);
        let depositor = ctx.accounts.depositor.to_account_info().key();
        let mut vault = &mut ctx.accounts.vault_account;

        // Check if the depositor has enough tokens
        if ctx.accounts.depositor_account.amount < deposit_amount {
            return Err(VaultError::InsufficientFunds.into());
        }

        // Update depositor balance
        let balances = &mut vault.depositor_balances;
        let balance = balances.entry(*depositor).or_insert(0);
        *balance += deposit_amount;

        // Transfer tokens into the vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.depositor_account.to_account_info(),
            to: ctx.accounts.vault_account.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, deposit_amount)?;

        msg!("Funds added successfully.");
        Ok(())
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>, withdraw_amount: u64) -> Result<()> {
        msg!("Attempting to withdraw {} tokens", withdraw_amount);
        let withdrawer = ctx.accounts.withdrawer.to_account_info().key();
        let mut vault = &mut ctx.accounts.vault_account;

        // Check if the withdrawer has sufficient balance
        let balances = &mut vault.depositor_balances;
        let balance = balances
            .get_mut(&withdrawer)
            .ok_or(VaultError::UnauthorizedWithdrawal)?;
        if *balance < withdraw_amount {
            return Err(VaultError::InsufficientFunds.into());
        }

        *balance -= withdraw_amount; // Update the balance

        // Transfer tokens from the vault to the withdrawer
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_account.to_account_info(),
            to: ctx.accounts.withdrawer_account.to_account_info(),
            authority: ctx.accounts.withdrawer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, withdraw_amount)?;

        msg!("Funds withdrawn successfully.");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetupVault<'info> {
    #[account(init, payer = user, space = 8000)]
    pub vault_account: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FundVault<'info> {
    #[account(mut)]
    pub vault_account: Account<'info, Vault>,
    #[account(mut)]
    pub depositor_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut)]
    pub vault_account: Account<'info, Vault>,
    #[account(mut)]
    pub withdrawer_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub withdrawer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub depositor_balances: HashMap<Pubkey, u64>,
}

#[error_code]
pub enum VaultError {
    #[msg("Insufficient funds to complete this transaction.")]
    InsufficientFunds,
    #[msg("Unauthorized withdrawal attempt.")]
    UnauthorizedWithdrawal,
}
