use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer, Token};
use spl_token::instruction::AuthorityType;
use std::mem::size_of;

pub mod account;
pub mod error;
pub mod constants;
pub mod utils;

use account::*;
use constants::*;
use error::*;
use utils::*;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod daddy_staking_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, stake_mode: u8) -> Result<()> {
        let user_pool = &mut ctx.accounts.user_pool;
        user_pool.owner = ctx.accounts.owner.key();
        user_pool.stake_mode = stake_mode;
        let timestamp = Clock::get()?.unix_timestamp;
        user_pool.stake_time = timestamp;

        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn stake_nft(
        ctx: Context<StakeNftToPool>, 
        rarity: u8,
    ) -> Result<()> {

        let user_pool = &mut ctx.accounts.user_pool;
        user_pool.add_nft(ctx.accounts.nft_mint.key(), rarity);

        ctx.accounts.global_authority.total_nft_count += 1;

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_nft_token_account.to_account_info(),
            to: ctx.accounts.dest_nft_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info()
        };
        let token_program = ctx.accounts.token_program.clone();
        let transfer_ctx = CpiContext::new(token_program, cpi_accounts);
        token::transfer(
            transfer_ctx,
            1
        )?;
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn withdraw_nft(
        ctx: Context<WithdrawNftFromPool>, 
        global_bump: u8,
    ) -> Result<()> {

        let user_pool = &mut ctx.accounts.user_pool;
        let item_count = user_pool.remove_nft(
            ctx.accounts.owner.key(),
            ctx.accounts.nft_mint.key()
        );

        ctx.accounts.global_authority.total_nft_count -= 1;

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.staked_nft_token_account.to_account_info(),
            to: ctx.accounts.user_nft_token_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        let token_program = ctx.accounts.token_program.clone();
        let transfer_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
        token::transfer(
            transfer_ctx,
            1
        )?;

        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn claim_reward(
        ctx: Context<ClaimReward>,
        global_bump: u8,
    ) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;

        let user_pool = &mut ctx.accounts.user_pool;
        let reward: u64 = user_pool.claim_reward(
            timestamp
        )?;

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.source_account.to_account_info(),
            to: ctx.accounts.dest_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        let token_program = ctx.accounts.token_program.clone();
        let transfer_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
        token::transfer(
            transfer_ctx,
            reward
        )?;

        Ok(())
    }
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init_if_needed,
        seeds = [USER_POOL_SEED.as_ref()],
        bump,
        payer = owner,
        space=size_of::<UserPool>() + 8,
    )]
    pub user_pool: Account<'info, UserPool>,

    #[account(
        init_if_needed,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
        payer = owner,
        space=size_of::<GlobalPool>() + 8,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,

}


#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8)]
pub struct StakeNftToPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: Account<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(mut)]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = ["staked-nft".as_ref(), nft_mint.key.as_ref()],
        bump,
        token::mint = nft_mint,
        token::authority = user_pool
    )]
    pub dest_nft_token_account: Account<'info, TokenAccount>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
    // pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8)]
pub struct WithdrawNftFromPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: Account<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = user_nft_token_account.owner == owner.key()
    )]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = ["staked-nft".as_ref(), nft_mint.key.as_ref()],
        bump = staked_nft_bump
    )]
    pub staked_nft_token_account: Account<'info, TokenAccount>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: Account<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    source_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    dest_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}


// Access control modifiers
fn user(pool_loader: &Account<UserPool>, user: &AccountInfo) -> Result<()> {
    require!(pool_loader.owner == *user.key, StakingError::InvalidUserPool);
    Ok(())
}