use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Token, TokenAccount, Mint, Transfer},
    associated_token::AssociatedToken,
};

declare_id!("8omdpoCLrwZPPvVDrLujvxdTRWCrkTNDFsXFmPyaNNfS");

// Constants
pub const RAYDIUM_PROGRAM_ID: &str = "devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH";
pub const OBSERVATION_STATE_LEN: usize = 8 + 32 + 8 + 8;

#[program]
pub mod clmm_trading_new {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        initial_sqrt_price: u128,
        tick_spacing: u16
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool_state;
        pool.authority = ctx.accounts.authority.key();
        pool.token_mint_0 = ctx.accounts.token_mint_0.key();
        pool.token_mint_1 = ctx.accounts.token_mint_1.key();
        pool.tick_spacing = tick_spacing;
        pool.sqrt_price = initial_sqrt_price;
        pool.observation_index = 0;
        pool.observation_update_duration = 3; // 3 seconds update interval

        msg!("Pool initialized with sqrt price: {}", initial_sqrt_price);
        Ok(())
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        minimum_amount_out: u64,
        sqrt_price_limit: u128,
        is_base_input: bool
    ) -> Result<()> {
        require!(amount_in > 0, ErrorCode::InvalidAmount);
        require!(minimum_amount_out > 0, ErrorCode::InvalidAmount);

        // Transfer tokens to pool
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.pool_token_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, amount_in)?;

        // Calculate swap result
        let amount_out = calculate_swap_output(
            amount_in,
            ctx.accounts.pool_state.sqrt_price,
            sqrt_price_limit,
            is_base_input
        )?;

        require!(
            amount_out >= minimum_amount_out,
            ErrorCode::InsufficientOutputAmount
        );

        // Update pool state
        let pool = &mut ctx.accounts.pool_state;
        pool.sqrt_price = calculate_new_sqrt_price(
            pool.sqrt_price,
            amount_in,
            amount_out,
            is_base_input
        )?;

        msg!("Swap executed: {} in, {} out", amount_in, amount_out);
        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_0: u64,
        amount_1: u64,
        lower_tick: i32,
        upper_tick: i32
    ) -> Result<()> {
        require!(amount_0 > 0 && amount_1 > 0, ErrorCode::InvalidAmount);
        require!(lower_tick < upper_tick, ErrorCode::InvalidTickRange);

        // Add liquidity logic here
        let pool = &mut ctx.accounts.pool_state;
        
        // Transfer tokens to pool vaults
        let transfer_0_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_0_account.to_account_info(),
                to: ctx.accounts.pool_token_0_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(transfer_0_ctx, amount_0)?;

        let transfer_1_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_1_account.to_account_info(),
                to: ctx.accounts.pool_token_1_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(transfer_1_ctx, amount_1)?;

        msg!("Liquidity added: {} token0, {} token1", amount_0, amount_1);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 2 + 16 + 8 + 8
    )]
    pub pool_state: Account<'info, PoolState>,

    pub token_mint_0: Account<'info, Mint>,
    pub token_mint_1: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pool_token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    #[account(mut)]
    pub user_token_0_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_1_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pool_token_0_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pool_token_1_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct PoolState {
    pub authority: Pubkey,
    pub token_mint_0: Pubkey,
    pub token_mint_1: Pubkey,
    pub tick_spacing: u16,
    pub sqrt_price: u128,
    pub observation_index: u64,
    pub observation_update_duration: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Invalid tick range")]
    InvalidTickRange,
    #[msg("Insufficient output amount")]
    InsufficientOutputAmount,
    #[msg("Price limit exceeded")]
    PriceLimitExceeded,
}

// Helper functions
fn calculate_swap_output(
    amount_in: u64,
    current_sqrt_price: u128,
    sqrt_price_limit: u128,
    is_base_input: bool
) -> Result<u64> {
    // Simplified calculation for example
    // In production, implement proper CLMM math
    Ok((amount_in as f64 * 0.997) as u64) // 0.3% fee
}

fn calculate_new_sqrt_price(
    current_sqrt_price: u128,
    amount_in: u64,
    amount_out: u64,
    is_base_input: bool
) -> Result<u128> {
    // Simplified calculation for example
    // In production, implement proper CLMM math
    Ok(current_sqrt_price)
}