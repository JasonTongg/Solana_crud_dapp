use anchor_lang::prelude::*;

#[cfg(test)]
mod tests;

declare_id!("CqPgrRpnxq3uS1PvmFLe1zT5cGYT6kuF25qAnXSGRbiN");

#[program]
pub mod crud {
    use super::*;

    pub fn create_journal_entry(ctx: Context<CreateEntry>, title: String, message: String) -> Result<()> {
        require!(title.len() <= 32, CrudError::TitleTooLong);
        require!(message.len() <= 128, CrudError::MessageTooLong);
        let journal_entry = &mut ctx.accounts.journal_entry;
        journal_entry.owner = ctx.accounts.owner.key();
        journal_entry.title = title;
        journal_entry.message = message;
        Ok(())
    }

    pub fn update_journal_entry(ctx: Context<UpdateJournal>, _title: String, message: String) -> Result<()> {
        require!(message.len() <= 128, CrudError::MessageTooLong);
        ctx.accounts.journal_entry.message = message;
        Ok(())
    }

    pub fn delete_journal_entry(_ctx: Context<DeleteJournal>, _title: String) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct DeleteJournal<'info> {
    #[account(
        mut,
        seeds = [title.as_bytes(), owner.key().as_ref()],
        bump,
        close = owner
    )]
    pub journal_entry: Account<'info, JouralEntryState>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct UpdateJournal<'info> {
    #[account(
        mut,
        seeds = [title.as_bytes(), owner.key().as_ref()],
        bump,
        realloc = 8 + JouralEntryState::INIT_SPACE,
        realloc::payer = owner,
        realloc::zero = true
    )]
    pub journal_entry: Account<'info, JouralEntryState>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct CreateEntry<'info> {
    #[account(
        init,
        seeds = [title.as_bytes(), owner.key().as_ref()],
        bump,
        space = 8 + JouralEntryState::INIT_SPACE,
        payer = owner
    )]
    pub journal_entry: Account<'info, JouralEntryState>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>
}

#[error_code]
pub enum CrudError {
    #[msg("Title exceeds maximum length of 32 characters")]
    TitleTooLong,
    #[msg("Message exceeds maximum length of 128 characters")]
    MessageTooLong,
}

#[account]
#[derive(InitSpace)]
pub struct JouralEntryState {
    pub owner: Pubkey,

    #[max_len(32)]
    pub title: String,

    #[max_len(128)]
    pub message: String
}