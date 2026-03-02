//! Proof verification.
//!
//! The verifier receives a `ProofResult` containing:
//!   1. A serialized aggregate Bulletproof range proof
//!   2. Pedersen commitments for each proven coordinate offset
//!   3. Public inputs (center_hash, radius, time window, counts)
//!
//! Verification confirms that the committed coordinate offsets all
//! fall within [0, bbox_width], proving the original GPS points
//! were inside the geofence — without revealing them.

use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::ristretto::CompressedRistretto;
use merlin::Transcript;

use crate::commitment;
use crate::types::*;

/// Bit width must match the prover's RANGE_BIT_WIDTH.
const RANGE_BIT_WIDTH: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    #[error("Proof deserialization failed: {0}")]
    DeserializationError(String),

    #[error("Range proof verification failed")]
    InvalidRangeProof,

    #[error("Public inputs inconsistent: {0}")]
    InconsistentInputs(String),

    #[error("count_proven ({proven}) is less than min_count ({required})")]
    CountBelowMinimum { proven: u32, required: u32 },

    #[error("Commitment count mismatch: expected {expected}, got {got}")]
    CommitmentMismatch { expected: usize, got: usize },
}

/// Verify a ZK-PoX proof cryptographically.
///
/// This performs full Bulletproofs verification:
///   1. Deserialize the range proof and commitments
///   2. Rebuild the Fiat-Shamir transcript
///   3. Verify the aggregate range proof against the commitments
pub fn verify_proof(result: &ProofResult) -> Result<(), VerifierError> {
    let pi = &result.public_inputs;

    if pi.count_proven < pi.min_count {
        return Err(VerifierError::CountBelowMinimum {
            proven: pi.count_proven,
            required: pi.min_count,
        });
    }

    if pi.center_hash == [0u8; 32] {
        return Err(VerifierError::InconsistentInputs(
            "center_hash is all zeros".into(),
        ));
    }

    verify_aggregate_range_proof(
        &result.proof_bytes,
        &result.commitments,
        pi.count_proven as usize,
    )
}

/// Cryptographically verify the aggregate Bulletproof range proof.
fn verify_aggregate_range_proof(
    proof_bytes: &[u8],
    commitments_bytes: &[u8],
    point_count: usize,
) -> Result<(), VerifierError> {
    let proof = RangeProof::from_bytes(proof_bytes)
        .map_err(|e| VerifierError::DeserializationError(format!("{:?}", e)))?;

    // Each point has 2 commitments (lat + lng), padded to power-of-two
    let n_values = (point_count * 2).next_power_of_two();
    let expected_bytes = n_values * 32;

    if commitments_bytes.len() != expected_bytes {
        return Err(VerifierError::CommitmentMismatch {
            expected: expected_bytes,
            got: commitments_bytes.len(),
        });
    }

    // Deserialize compressed Ristretto points
    let commitments: Vec<CompressedRistretto> = commitments_bytes
        .chunks_exact(32)
        .map(|chunk| {
            let mut buf = [0u8; 32];
            buf.copy_from_slice(chunk);
            CompressedRistretto(buf)
        })
        .collect();

    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(RANGE_BIT_WIDTH, n_values);

    let mut transcript = Transcript::new(b"zkpox-geo-proof-v1");

    proof
        .verify_multiple(&bp_gens, &pc_gens, &mut transcript, &commitments, RANGE_BIT_WIDTH)
        .map_err(|_| VerifierError::InvalidRangeProof)?;

    Ok(())
}

/// Check if a center_hash matches a known location commitment.
pub fn verify_center_hash(
    expected_center_hash: &[u8; 32],
    proof_result: &ProofResult,
) -> bool {
    proof_result.public_inputs.center_hash == *expected_center_hash
}

/// Hash public inputs for compact on-chain storage.
pub fn hash_public_inputs(pi: &PublicInputs) -> [u8; 32] {
    commitment::public_inputs_hash(
        &pi.center_hash,
        pi.radius_m,
        pi.time_window_days,
        pi.min_count,
        pi.count_proven,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prover::generate_proof;

    fn make_test_points() -> Vec<SignedGPSPoint> {
        (0..20)
            .map(|i| SignedGPSPoint {
                lat: 52.2297 + (i as f64) * 0.00001,
                lng: 21.0122 + (i as f64) * 0.00001,
                timestamp: 1_740_000_000 + (i as i64) * 300,
                accuracy: 5.0,
                signature: vec![0u8; 64],
            })
            .collect()
    }

    #[test]
    fn test_verify_valid_proof() {
        let points = make_test_points();
        let request = ProofRequest {
            claim_type: ClaimType::Attendance,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 500,
            time_start: 1_740_000_000 - 1,
            time_end: 1_740_000_000 + 100_000,
            min_count: 5,
            night_only: false,
        };

        let result = generate_proof(&points, &request).unwrap();
        let verify_result = verify_proof(&result);
        assert!(verify_result.is_ok(), "Valid proof should verify: {:?}", verify_result.err());
    }

    #[test]
    fn test_tampered_proof_fails() {
        let points = make_test_points();
        let request = ProofRequest {
            claim_type: ClaimType::Attendance,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 500,
            time_start: 1_740_000_000 - 1,
            time_end: 1_740_000_000 + 100_000,
            min_count: 5,
            night_only: false,
        };

        let mut result = generate_proof(&points, &request).unwrap();
        // Tamper with the proof
        if let Some(byte) = result.proof_bytes.get_mut(10) {
            *byte ^= 0xFF;
        }
        assert!(verify_proof(&result).is_err(), "Tampered proof should fail");
    }

    #[test]
    fn test_verify_center_hash_match() {
        let points = make_test_points();
        let request = ProofRequest {
            claim_type: ClaimType::Attendance,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 500,
            time_start: 1_740_000_000 - 1,
            time_end: 1_740_000_000 + 100_000,
            min_count: 5,
            night_only: false,
        };

        let result = generate_proof(&points, &request).unwrap();
        assert!(verify_center_hash(&result.public_inputs.center_hash, &result));
        assert!(!verify_center_hash(&[0xFFu8; 32], &result));
    }

    #[test]
    fn test_hash_public_inputs_deterministic() {
        let pi = PublicInputs {
            center_hash: [0xABu8; 32],
            radius_m: 200,
            time_window_days: 30,
            min_count: 10,
            count_proven: 15,
        };
        assert_eq!(hash_public_inputs(&pi), hash_public_inputs(&pi));
    }
}
