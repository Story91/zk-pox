use anchor_lang::prelude::*;

declare_id!("ZKPoX1111111111111111111111111111111111111");

/// Maximum number of witnesses that can corroborate a credential.
const MAX_WITNESSES: usize = 8;

/// Size of ExperienceCredential account in bytes.
/// 8 (discriminator) + 1 + 32 + 1 + 32 + 32 + 32 + 1 + 4 + 8 + 1 + (32 * 8) + 1 = 409
const CREDENTIAL_SIZE: usize = 8 + 1 + 32 + 1 + 32 + 32 + 32 + 1 + 4 + 8 + 1 + (32 * MAX_WITNESSES) + 1;

#[program]
pub mod zk_pox {
    use super::*;

    /// Submit a new experience credential to the chain.
    ///
    /// The ZK proof itself is verified off-chain by mesh peers (via the
    /// CORROBORATE protocol). This instruction records the proof hash,
    /// public inputs hash, and attestation metadata on-chain as a
    /// soulbound credential tied to the agent's SATI identity.
    pub fn submit_credential(
        ctx: Context<SubmitCredential>,
        credential_id: [u8; 32],
        claim_type: u8,
        proof_hash: [u8; 32],
        public_inputs_hash: [u8; 32],
        commitments_hash: [u8; 32],
        count_proven: u32,
    ) -> Result<()> {
        require!(claim_type <= 5, ZkPoxError::InvalidClaimType);
        require!(count_proven > 0, ZkPoxError::InvalidCountProven);

        let credential = &mut ctx.accounts.credential;
        let clock = Clock::get()?;

        credential.version = 2;
        credential.agent_id = ctx.accounts.agent.key().to_bytes();
        credential.claim_type = claim_type;
        credential.proof_hash = proof_hash;
        credential.public_inputs_hash = public_inputs_hash;
        credential.commitments_hash = commitments_hash;
        credential.witness_count = 0;
        credential.count_proven = count_proven;
        credential.issued_at = clock.unix_timestamp;
        credential.revoked = false;
        credential.witnesses = [[0u8; 32]; MAX_WITNESSES];
        credential.bump = ctx.bumps.credential;

        emit!(CredentialSubmitted {
            agent: ctx.accounts.agent.key(),
            credential_id,
            claim_type,
            proof_hash,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Add a witness attestation to an existing credential.
    ///
    /// Called by mesh peers who have verified the ZK proof off-chain
    /// via the CORROBORATE protocol. Each witness can only attest once.
    pub fn add_witness(ctx: Context<AddWitness>) -> Result<()> {
        let credential = &mut ctx.accounts.credential;
        let witness_key = ctx.accounts.witness.key().to_bytes();

        require!(!credential.revoked, ZkPoxError::CredentialRevoked);
        require!(
            (credential.witness_count as usize) < MAX_WITNESSES,
            ZkPoxError::MaxWitnessesReached
        );

        for i in 0..credential.witness_count as usize {
            require!(
                credential.witnesses[i] != witness_key,
                ZkPoxError::AlreadyWitnessed
            );
        }

        credential.witnesses[credential.witness_count as usize] = witness_key;
        credential.witness_count += 1;

        emit!(WitnessAdded {
            credential: ctx.accounts.credential.key(),
            witness: ctx.accounts.witness.key(),
            witness_count: credential.witness_count,
        });

        Ok(())
    }

    /// Revoke a credential. Only the original agent can revoke.
    pub fn revoke_credential(ctx: Context<RevokeCredential>) -> Result<()> {
        let credential = &mut ctx.accounts.credential;

        require!(!credential.revoked, ZkPoxError::CredentialRevoked);

        credential.revoked = true;

        emit!(CredentialRevoked {
            credential: ctx.accounts.credential.key(),
            agent: ctx.accounts.agent.key(),
        });

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(credential_id: [u8; 32])]
pub struct SubmitCredential<'info> {
    #[account(mut)]
    pub agent: Signer<'info>,

    #[account(
        init,
        payer = agent,
        space = CREDENTIAL_SIZE,
        seeds = [b"credential", agent.key().as_ref(), &credential_id],
        bump,
    )]
    pub credential: Account<'info, ExperienceCredential>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddWitness<'info> {
    pub witness: Signer<'info>,

    #[account(
        mut,
        constraint = !credential.revoked @ ZkPoxError::CredentialRevoked,
    )]
    pub credential: Account<'info, ExperienceCredential>,
}

#[derive(Accounts)]
pub struct RevokeCredential<'info> {
    pub agent: Signer<'info>,

    #[account(
        mut,
        constraint = credential.agent_id == agent.key().to_bytes() @ ZkPoxError::Unauthorized,
    )]
    pub credential: Account<'info, ExperienceCredential>,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[account]
pub struct ExperienceCredential {
    pub version: u8,
    pub agent_id: [u8; 32],
    pub claim_type: u8,
    pub proof_hash: [u8; 32],
    pub public_inputs_hash: [u8; 32],
    /// SHA-256 hash of the Pedersen commitments used in the range proof.
    /// Allows the verifier to bind proof verification to the on-chain record.
    pub commitments_hash: [u8; 32],
    pub witness_count: u8,
    /// Number of GPS points cryptographically proven via Bulletproofs.
    pub count_proven: u32,
    pub issued_at: i64,
    pub revoked: bool,
    pub witnesses: [[u8; 32]; MAX_WITNESSES],
    pub bump: u8,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct CredentialSubmitted {
    pub agent: Pubkey,
    pub credential_id: [u8; 32],
    pub claim_type: u8,
    pub proof_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct WitnessAdded {
    pub credential: Pubkey,
    pub witness: Pubkey,
    pub witness_count: u8,
}

#[event]
pub struct CredentialRevoked {
    pub credential: Pubkey,
    pub agent: Pubkey,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[error_code]
pub enum ZkPoxError {
    #[msg("Invalid claim type (must be 0-5)")]
    InvalidClaimType,

    #[msg("Credential has been revoked")]
    CredentialRevoked,

    #[msg("Maximum number of witnesses reached")]
    MaxWitnessesReached,

    #[msg("This witness has already attested")]
    AlreadyWitnessed,

    #[msg("Only the credential owner can perform this action")]
    Unauthorized,

    #[msg("count_proven must be > 0")]
    InvalidCountProven,
}
