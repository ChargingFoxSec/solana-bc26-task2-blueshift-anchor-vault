use anchor_lang::prelude::*;

declare_id!("ERG5xSpUfEB9xqTU99ahtgVovEZDBk22jwi6AGvgqCdW");

#[program]
pub mod blueshift_anchor_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
