use {super::deposit::VaultPda, quasar_lang::prelude::*};

#[derive(Accounts)]
pub struct Withdraw {
    #[account(mut)]
    pub user: Signer,
    #[account(mut, address = VaultPda::seeds(user.address()))]
    pub vault: UncheckedAccount,
    pub system_program: Program<SystemProgram>,
}

impl Withdraw {
    #[inline(always)]
    pub fn withdraw(&self, amount: u64, bumps: &WithdrawBumps) -> Result<(), ProgramError> {
        let bump = [bumps.vault];
        let seeds = [
            Seed::from(b"vault" as &[u8]),
            Seed::from(self.user.address().as_ref()),
            Seed::from(bump.as_ref()),
        ];
        self.system_program
            .transfer(&self.vault, &self.user, amount)
            .invoke_signed(&seeds)
    }
}
