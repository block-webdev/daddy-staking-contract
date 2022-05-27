use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer, Token};
use spl_token::instruction::AuthorityType;
use std::mem::size_of;
use spl_token::state;

pub mod account;
pub mod error;
pub mod constants;
pub mod utils;

use account::*;
use constants::*;
use error::*;
use utils::*;


declare_id!("A4RDTZxpskjCkY9mWgkovkx5aXUYEwZHJkQ6uWi1nWWH");

#[program]
pub mod daddy_staking_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {

        Ok(())
    }

    pub fn init_user_pool(ctx: Context<InitUserPool>, stake_mode: u8) -> Result<()> {
        let user_pool = &mut ctx.accounts.user_pool;
        user_pool.owner = ctx.accounts.owner.key();
        user_pool.rand = ctx.accounts.rand.key();
        user_pool.stake_mode = stake_mode;
        let timestamp = Clock::get()?.unix_timestamp;
        user_pool.stake_time = timestamp;
        user_pool.reward_time = timestamp;

        Ok(())
    }


    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn stake_nft(
        ctx: Context<StakeNft>, 
        global_bump: u8,
        rarity: u8,
    ) -> Result<()> {

        let user_pool = &mut ctx.accounts.user_pool;
        user_pool.add_nft(ctx.accounts.nft_mint.key(), rarity);

        ctx.accounts.global_authority.total_nft_count += 1;

        let cpi_accounts = Transfer {
            from: ctx.accounts.source_nft_account.clone(),
            to: ctx.accounts.dest_nft_account.clone(),
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
    pub fn unstake_nft(
        ctx: Context<UnstakeNft>, 
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
            from: ctx.accounts.source_nft_account.clone(),
            to: ctx.accounts.dest_nft_account.clone(),
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

    pub fn get_reward(
        ctx: Context<GetRewardAmount>,
    ) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;

        let user_pool = &mut ctx.accounts.user_pool;
        let mut reward: u64 = user_pool.calc_reward(
            timestamp
        )?;
        let decimals = ctx.accounts.reward_mint.decimals;
        let x: i32 = 10;
        reward = reward * (x.pow(decimals as u32)) as u64;
        user_pool.reward_amount = reward;

        Ok(())
    }

    pub fn claim_reward(
        ctx: Context<ClaimReward>,
        global_bump: u8,
        reward_amount: u64,
    ) -> Result<()> {
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
            reward_amount
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {

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
pub struct InitUserPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [(*rand.key).as_ref()],
        bump,
        payer = owner,
        space=size_of::<UserPool>() + 8,
    )]
    pub user_pool: Account<'info, UserPool>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub rand : AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct StakeNft<'info> {
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
    nft_mint : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    source_nft_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    dest_nft_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    // pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct UnstakeNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, constraint = owner.key() == user_pool.owner)]
    pub user_pool: Account<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    nft_mint : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    source_nft_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    dest_nft_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}


#[derive(Accounts)]
pub struct GetRewardAmount<'info> {
    #[account(mut)]
    pub user_pool: Account<'info, UserPool>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    reward_mint : Account<'info, Mint>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

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