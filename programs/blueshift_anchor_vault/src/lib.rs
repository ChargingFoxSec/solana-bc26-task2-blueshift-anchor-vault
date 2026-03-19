// 引入 Anchor 常用内容和 system program 的转账辅助类型
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
// 声明当前程序的 Program ID，必须和实际部署/调用的程序地址一致
declare_id!("22222222222222222222222222222222222222222222");
// Anchor 程序模块宏，标记下面模块里的函数是可调用的链上指令
#[program]
// 程序模块名，通常和程序 crate 名一致
pub mod blueshift_anchor_vault {
    use super::*;
    // deposit 指令：接收账户上下文和存款金额，成功时返回 Ok(())
    pub fn deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {
        // 要求 vault 当前余额为 0，防止重复向一个已存在的 vault 存款
        require_eq!(
            ctx.accounts.vault.lamports(),
            0,
            VaultError::VaultAlreadyExists
        );

        // 要求存入金额大于 0 字节系统账户的免租最低余额
        require_gt!(
            amount,
            Rent::get()?.minimum_balance(0),
            VaultError::InvalidAmount
        );
        // 调用 system program 的转账 helper，把 lamports 从 signer 转到 vault
        transfer(
            CpiContext::new(
                // 将 Anchor 账户对象转成底层 AccountInfo，供 CPI 使用
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<VaultAction>) -> Result<()> {
        // 要求 vault 中必须有余额，否则没有可提取的 lamports
        require_neq!(ctx.accounts.vault.lamports(), 0, VaultError::InvalidAmount);
        // 取出 signer 公钥，后面用它和 bump 重新组成 vault PDA 的 signer seeds
        let signer_key = ctx.accounts.signer.key();
        // 这组 seeds 必须和 vault 账户约束中的 seeds + bump 完全一致
        let signer_seeds = &[b"vault", signer_key.as_ref(), &[ctx.bumps.vault]];

        // 把 vault 中全部 lamports 转回 signer
        transfer(
            // new_with_signer 用于 PDA 作为 signer 的 CPI
            // 第三个参数是 signer seeds；运行时会据此验证 vault PDA 可以代表自己签名
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.signer.to_account_info(),
                },
                &[&signer_seeds[..]],
            ),
            ctx.accounts.vault.lamports(),
        )?;
        Ok(())
    }
}

// 定义 deposit / withdraw 共用的账户上下文
#[derive(Accounts)]
pub struct VaultAction<'info> {
    // 交易签名者，也是 vault 的所有者；余额会变化，所以需要 mut
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        // 要求 vault 必须是由固定前缀 + signer 公钥 推导出的 PDA
        seeds = [b"vault", signer.key().as_ref()],
        // 让 Anchor 自动校验 bump，并把结果放到 ctx.bumps.vault
        bump,
    )]
    // 这里只需要一个普通系统账户来存 lamports，不存自定义数据
    pub vault: SystemAccount<'info>,
    // system program 账户，后续做转账 CPI 时需要传入
    pub system_program: Program<'info, System>,
}

// 自定义错误码，供 require_* 宏失败时返回
#[error_code]
pub enum VaultError {
    // vault 已经有余额，说明它已经被用过
    #[msg("Vault already exists")]
    VaultAlreadyExists,
    // 金额不合法：例如存款太小，或取款时 vault 为空
    #[msg("Invalid amount")]
    InvalidAmount,
}
