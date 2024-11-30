use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked, CloseAccount, transfer_checked, close_account},
    associated_token::AssociatedToken,
};

use pyth_solana_receiver_sdk::price_update::{self,PriceUpdateV2,get_feed_id_from_hex};


mod instructions;
pub use instructions::*;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod manager {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, min_collat_ratio: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;        
        // Initialize state
        state.min_collat_ratio = min_collat_ratio;
        state.weth_mint = ctx.accounts.weth_mint.key();
        state.shusd_mint = ctx.accounts.shusd_mint.key();
        state.authority = ctx.accounts.deployer.key();
        state.bump = ctx.bumps.state;

        // Validation
        require!(
            min_collat_ratio >= 150, // 100% minimum
            ErrorCode::InvalidCollateralRatio
        );

        Ok(())
    }

    // the function that deposits the weth into the vault and than mints corresponding shusd
    pub fn deposit(ctx: Context<DepositWeth>, amount: u64) -> Result<()> {
        // require amount is greater than 0
        require!(amount > 0, ErrorCode::InvalidAmount);
        deposit_weth(ctx, amount)?;
        Ok(())
    }

    pub fn mint(ctx: Context<MintShusd>, amount: u64) -> Result<()> {
        mint_to_depositor(ctx, amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<WithdrawWeth>, amount: u64) -> Result<()> {
        withdraw_weth(ctx, amount)?;
        Ok(())
    }

    pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
        liquidate_user(ctx)?;
        Ok(())
    }

    pub fn burn(ctx: Context<Liquidate>, amount: u64) -> Result<()> {
        burn_shusd(ctx, amount)?;
        Ok(())
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub deployer: Signer<'info>,

    #[account(
        init,
        payer = deployer,
        space = 8 + State::INIT_SPACE,  
        seeds = [b"state".as_ref()],
        bump
    )]
    pub state: Account<'info, State>,

    /// The WETH mint address
    pub weth_mint: InterfaceAccount<'info, Mint>,
    
    /// The shUSD mint address
    pub shusd_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

// so the functionality we want are deposit, withdraw and liquidate - within deposit we do minting, within withdraw we do burning and within liquidate we do a burning too
// but in liquidate the signer is not the user that hold the shusd tokens, so we will use the freeze authority to do the burning and than give the underlying weth to the user
// another thing is we will need a vault where we will be storing all the weth and the authority of the vault will be this contract, well not this contract but some other account 

#[derive(Accounts)]
pub struct DepositWeth<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    // this is ATA
    #[account(
        init,
        payer = depositor,
        associated_token::mint = weth_mint,
        associated_token::authority = deposit_state,
        associated_token::token_program = token_program,
    )]
    pub vault_weth: InterfaceAccount<'info, TokenAccount>,

    // because we are initializing here, we will need to save inner
    #[account(
        init,
        payer = depositor,
        space = DepositState::INIT_SPACE,
        seeds = [b"deposit_state".as_ref(), depositor.key().as_ref()],
        bump
    )]
    pub deposit_state: Account<'info, DepositState>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump
    )]
    pub state: Account<'info, State>,

    #[account(
        mint::token_program = token_program,
        constraint = weth_mint.key() == state.weth_mint
    )]
    pub weth_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        associated_token::mint = weth_mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_weth_account: InterfaceAccount<'info, TokenAccount>,




    pub system_program: Program<'info, System>,
    // only one associated token program for all the accounts
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// withdraw
#[derive(Accounts)]
pub struct WithdrawWeth<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = weth_mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_weth_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = weth_mint,
        associated_token::authority = deposit_state,
        associated_token::token_program = token_program,
    )]
    pub vault_weth: InterfaceAccount<'info, TokenAccount>,

    // add state like above
    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump
    )]
    pub state: Account<'info, State>,

    #[account(
        mut,
        close = depositor,
        seeds = [b"deposit_state".as_ref(), depositor.key().as_ref()],
        bump = deposit_state.bump,
    )]
    pub deposit_state: Account<'info, DepositState>,        

    #[account(
        mint::token_program = token_program,
        constraint = weth_mint.key() == state.weth_mint
    )]
    pub weth_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub price_feed: Account<'info, PriceUpdateV2>,
}

#[derive(Accounts)]
pub struct MintShusd<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"mint_authority".as_ref()],
        bump
    )]
    pub mint_authority: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump
    )]
    pub state: Account<'info, State>,

    #[account(mut)]
    pub price_feed: Account<'info, PriceUpdateV2>,

    #[account(
        mut,
        seeds = [b"deposit_state".as_ref(), depositor.key().as_ref()],
        bump = deposit_state.bump,
    )]
    pub deposit_state: Account<'info, DepositState>,    

    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = shusd_mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_shusd_account: InterfaceAccount<'info, TokenAccount>,

    

    #[account(
        mint::token_program = token_program,
        constraint = shusd_mint.key() == state.shusd_mint
    )]
    pub shusd_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program,
        constraint = weth_mint.key() == state.weth_mint
    )]
    pub weth_mint: InterfaceAccount<'info, Mint>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    // this is ATA
    
    #[account(
        mut,
        associated_token::mint = weth_mint,
        associated_token::authority = deposit_state,
        associated_token::token_program = token_program,
    )]
    pub vault_weth: InterfaceAccount<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump
    )]
    pub state: Account<'info, State>,

    #[account(mut)]
    pub price_feed: Account<'info, PriceUpdateV2>,

    #[account(
        mut,
        seeds = [b"deposit_state".as_ref(), liquidator.key().as_ref()],
        bump = deposit_state.bump,
    )]
    pub deposit_state: Account<'info, DepositState>,    

    #[account(
        mut,
        associated_token::mint = shusd_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program,
    )]
    pub liquidator_shusd_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,  
        payer = liquidator,      
        associated_token::mint = weth_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program,
    )]
    pub liquidator_weth_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = weth_mint,
        associated_token::authority = deposit_state,
        associated_token::token_program = token_program,
    )]
    pub vault_weth: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mint::token_program = token_program,
        constraint = shusd_mint.key() == state.shusd_mint
    )]
    pub shusd_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program,
        constraint = weth_mint.key() == state.weth_mint
    )]
    pub weth_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
#[derive(InitSpace)]
pub struct DepositState {
    pub amount_minted: u64, // This field cannot be marked as mutable in a struct definition
                           // Mutability is determined when accessing the struct instance
    pub amount_deposited: u64,
    pub bump: u8,
}

#[account]
// init space
#[derive(InitSpace)]
pub struct State {
    pub min_collat_ratio: u64,
    pub weth_mint: Pubkey,
    pub shusd_mint: Pubkey,
    pub authority: Pubkey,  // The admin who can update parameters
    pub bump: u8,
}
fn deposit_weth(ctx: Context<DepositWeth>, amount: u64) -> Result<()> {
    // we will transfer tokens from the signer to the vault
    transfer_tokens(
        &ctx.accounts.depositor_weth_account,
        &ctx.accounts.vault_weth,
        &amount,
        &ctx.accounts.weth_mint,
        &ctx.accounts.depositor,
        &ctx.accounts.token_program,
    )?;

    // set the deposit state
    ctx.accounts.deposit_state.set_inner(DepositState {
        amount_minted: 0,
        amount_deposited: amount,
        bump: ctx.bumps.deposit_state,
    });

    Ok(())
}

fn withdraw_weth(ctx: Context<WithdrawWeth>, amount: u64) -> Result<()> {

    // before withdrawing decrease deposited amount by amount 
    let deposit_state = &mut ctx.accounts.deposit_state;
    deposit_state.amount_deposited -= amount;

    // check for collateral ratio
    // collateral ratio function should ideally not take the context, just take the price thing and 
    let collateral_ratio: u128 = collateral_ratio(&ctx.accounts.price_feed, deposit_state)?;
    
    require!(
        collateral_ratio >= ctx.accounts.state.min_collat_ratio as u128, 
        ErrorCode::CollateralRatioTooLow
    );

    let seeds = [
        b"deposit_state", 
        ctx.accounts.depositor.to_account_info().key.as_ref(), 
        &[ctx.accounts.deposit_state.bump]
    ];
    let signer = &[&seeds[..]];

    let accounts = TransferChecked{
        from: ctx.accounts.vault_weth.to_account_info(),
        mint: ctx.accounts.weth_mint.to_account_info(),
        to: ctx.accounts.depositor_weth_account.to_account_info(),
        authority: ctx.accounts.deposit_state.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), accounts, signer);



    transfer_checked(cpi_context, ctx.accounts.deposit_state.amount_deposited, ctx.accounts.weth_mint.decimals)?;

    let close_accounts = CloseAccount{
        account: ctx.accounts.vault_weth.to_account_info(),
        destination: ctx.accounts.depositor.to_account_info(),
        authority: ctx.accounts.deposit_state.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), close_accounts, signer);
    close_account(cpi_context)?;

    Ok(())
}

fn mint_to_depositor(ctx: Context<MintShusd>, amount: u64) -> Result<()> {
    let deposit_state = &mut ctx.accounts.deposit_state;
    deposit_state.amount_minted += amount;

    let collateral_ratio: u128 = collateral_ratio(&ctx.accounts.price_feed, deposit_state)?;

    require!(
        collateral_ratio >= ctx.accounts.state.min_collat_ratio as u128, 
        ErrorCode::CollateralRatioTooLow
    );

    mint_shusd(ctx, amount)?;
    Ok(())
}

fn mint_shusd(ctx: Context<MintShusd>, amount: u64) -> Result<()> {
    // Create seeds for PDA signing
    let seeds = &[
        b"mint_authority".as_ref(),
        &[ctx.bumps.mint_authority]
    ];
    let signer_seeds = &[&seeds[..]];

    // Create the MintTo instruction accounts
    let cpi_accounts = token_interface::MintTo {
        mint: ctx.accounts.shusd_mint.to_account_info(),
        to: ctx.accounts.depositor_shusd_account.to_account_info(),
        authority: ctx.accounts.mint_authority.to_account_info(),
    };

    // Create the CPI context with signer seeds
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    // Execute the mint instruction
    token_interface::mint_to(cpi_ctx, amount)?;

    Ok(())
}

fn collateral_ratio(price_feed: &PriceUpdateV2 , deposit_state: &DepositState) -> Result<u128> {
    let maximum_age: u64 = 30;
    let feed_id = get_feed_id_from_hex("4TQ1VVWkrYUvyQ6hMmjepwr7swvqssvLi75BiJi13Tf3")?;
    
    let weth_price = price_feed.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;
    
    // adjust for weth_price.exponent
    let weth_price_adjusted: u128 = (weth_price.price as u128) * 10u128.pow((-weth_price.exponent) as u32);
    
    let amount_deposited = deposit_state.amount_deposited;  // Assuming field is called 'amount'
    let amount_minted = deposit_state.amount_minted;

    // If nothing minted yet, return max ratio
    if amount_minted == 0 {
        return Ok(u128::MAX);  // Changed to u128::MAX since we're working with u128
    }

    // Calculate total value 
    // Since price comes from Pyth adjusted for exponent (e.g., ~2000_00000000 for $2000)
    let total_value = (amount_deposited as u128)
        .checked_mul(weth_price_adjusted)
        .ok_or(ErrorCode::MathOverflow)?;

    // Calculate ratio
    let collateral_ratio = total_value
        .checked_div(amount_minted as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(collateral_ratio)
}
  

#[error_code]
pub enum ErrorCode {
    #[msg("Collateral ratio below minimum")]
    CollateralRatioTooLow,
    #[msg("Invalid collateral ratio provided")]
    InvalidCollateralRatio,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Protocol has already been initialized")]
    AlreadyInitialized,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Position is not eligible for liquidation")]
    CannotLiquidate,
}

fn burn_shusd(ctx: Context<Liquidate>, amount: u64) -> Result<()> {
    // decrease the minted amount from deposit state
    let deposit_state = &mut ctx.accounts.deposit_state;
    deposit_state.amount_minted -= amount;

    let cpi_accounts = token_interface::Burn {
        mint: ctx.accounts.shusd_mint.to_account_info(),
        from: ctx.accounts.liquidator_shusd_account.to_account_info(),
        authority: ctx.accounts.liquidator.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );

    token_interface::burn(cpi_ctx, amount)?;

    Ok(())
}

fn liquidate_user(ctx: Context<Liquidate>) -> Result<()> {
    // Check if collateral ratio is below minimum
    let collateral_ratio = collateral_ratio(&ctx.accounts.price_feed, &ctx.accounts.deposit_state)?;
    
    require!(
        collateral_ratio < ctx.accounts.state.min_collat_ratio as u128,
        ErrorCode::CannotLiquidate
    );

    // Get amounts to burn and transfer
    let amount_to_burn = ctx.accounts.deposit_state.amount_minted;
    let collateral_to_transfer = ctx.accounts.deposit_state.amount_deposited;

    // Burn all shUSD tokens
    burn_shusd(&ctx, amount_to_burn)?;

    // Transfer all WETH to liquidator
    let seeds = [
        b"deposit_state".as_ref(), 
        ctx.accounts.liquidator.key.as_ref(),
        &[ctx.accounts.deposit_state.bump]
    ];
    let signer = &[&seeds[..]];

    let accounts = TransferChecked {
        from: ctx.accounts.vault_weth.to_account_info(),
        mint: ctx.accounts.weth_mint.to_account_info(),
        to: ctx.accounts.liquidator_weth_account.to_account_info(),
        authority: ctx.accounts.deposit_state.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        signer
    );

    transfer_checked(cpi_context, collateral_to_transfer, ctx.accounts.weth_mint.decimals)?;

    // Reset deposit state
    ctx.accounts.deposit_state.amount_minted = 0;
    ctx.accounts.deposit_state.amount_deposited = 0;

    Ok(())
}