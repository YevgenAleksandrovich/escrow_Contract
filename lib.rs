use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("escrw111111111111111111111111111111111");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.escrow.set_inner(Escrow {
            maker: ctx.accounts.maker.key(),
            mint_a: ctx.accounts.mint_a.key(),
            mint_b: ctx.accounts.mint_b.key(),
            receive,
            bump: ctx.bumps.escrow,
            seed,
        });

        // Transfer tokens from maker to vault
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.maker_ata_a.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.maker.to_account_info(),
        };
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), deposit)?;
        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        // Transfer receive amount from taker to maker
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.taker_ata_b.to_account_info(),
            to: ctx.accounts.maker_ata_b.to_account_info(),
            authority: ctx.accounts.taker.to_account_info(),
        };
        token::transfer(CpiContext::new(cpi_program.clone(), cpi_accounts),
                        ctx.accounts.escrow.receive)?;

        // Transfer deposit from vault to taker
        let seeds = &[
            b"vault",
            ctx.accounts.maker.as_ref(),
            ctx.accounts.escrow.mint_a.as_ref(),
            &ctx.accounts.escrow.seed.to_le_bytes(),
            &[ctx.accounts.vault_bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.taker_ata_a.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };
        token::transfer(CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
                        ctx.accounts.vault.amount)?;
        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        // Transfer deposit back to maker and close escrow
        let seeds = &[
            b"vault",
            ctx.accounts.maker.as_ref(),
            ctx.accounts.escrow.mint_a.as_ref(),
            &ctx.accounts.escrow.seed.to_le_bytes(),
            &[ctx.accounts.vault_bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.maker_ata_a.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            signer,
        ), ctx.accounts.vault.amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init,
        payer = maker,
        seeds = [b"escrow", maker.key().as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + Escrow::INIT_SPACE,
    )]
    pub escrow: Box<Account<'info, Escrow>>,
    #[account(
        init,
        payer = maker,
        seeds = [b"vault", maker.key().as_ref(), mint_a.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        token::mint = mint_a,
        token::authority = vault,
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}