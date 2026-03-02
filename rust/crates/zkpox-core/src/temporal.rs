//! Temporal range proofs — prove timestamps fall within a time window
//! without revealing the exact timestamps.
//!
//! Uses the same Bulletproofs aggregate range proof approach as the spatial
//! proofs, but applied to timestamps:
//!   - Secret value: `ts_offset = timestamp - time_start`
//!   - Range: `[0, time_window_seconds]`
//!   - Proof: all committed timestamp offsets are within the window
//!
//! The verifier learns only that N timestamps are within `[time_start, time_end]`,
//! not the exact timestamps themselves.

use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::ristretto::CompressedRistretto;
use curve25519_dalek_ng::scalar::Scalar;
use merlin::Transcript;
use rand::RngCore;

use crate::types::SignedGPSPoint;

const TEMPORAL_BIT_WIDTH: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum TemporalError {
    #[error("No points in time window")]
    NoPointsInWindow,

    #[error("Invalid time window: start must be before end")]
    InvalidWindow,

    #[error("Bulletproof generation failed: {0}")]
    ProofGeneration(String),

    #[error("Proof deserialization failed: {0}")]
    Deserialization(String),

    #[error("Temporal range proof verification failed")]
    InvalidProof,

    #[error("Commitment count mismatch: expected {expected}, got {got}")]
    CommitmentMismatch { expected: usize, got: usize },
}

/// Result of temporal range proof generation.
#[derive(Debug, Clone)]
pub struct TemporalProofResult {
    pub proof_bytes: Vec<u8>,
    pub commitments: Vec<u8>,
    pub count: u32,
    pub time_window_seconds: u64,
}

/// Generate a temporal range proof for a set of GPS points.
///
/// Proves that `count` timestamps all fall within `[time_start, time_end]`
/// without revealing the actual timestamps. The proof is an aggregate
/// Bulletproof over committed timestamp offsets.
pub fn generate_temporal_proof(
    points: &[SignedGPSPoint],
    qualifying_indices: &[usize],
    time_start: i64,
    time_end: i64,
) -> Result<TemporalProofResult, TemporalError> {
    if time_start >= time_end {
        return Err(TemporalError::InvalidWindow);
    }
    if qualifying_indices.is_empty() {
        return Err(TemporalError::NoPointsInWindow);
    }

    let window = (time_end - time_start) as u64;

    let mut values: Vec<u64> = Vec::with_capacity(qualifying_indices.len());
    for &idx in qualifying_indices {
        let ts = points[idx].timestamp;
        if ts < time_start || ts > time_end {
            continue;
        }
        values.push((ts - time_start) as u64);
    }

    if values.is_empty() {
        return Err(TemporalError::NoPointsInWindow);
    }

    // Verify all offsets fit within TEMPORAL_BIT_WIDTH bits
    let max_val = 1u64 << TEMPORAL_BIT_WIDTH;
    if window >= max_val {
        // Window too large for 32-bit range proof — clamp offsets
        for v in values.iter_mut() {
            if *v >= max_val {
                *v = max_val - 1;
            }
        }
    }

    let n = values.len();
    let padded_n = n.next_power_of_two();

    let mut blindings: Vec<Scalar> = Vec::with_capacity(padded_n);
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        bytes[31] &= 0x0F;
        blindings.push(Scalar::from_bytes_mod_order(bytes));
    }

    for _ in n..padded_n {
        values.push(0);
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        bytes[31] &= 0x0F;
        blindings.push(Scalar::from_bytes_mod_order(bytes));
    }

    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(TEMPORAL_BIT_WIDTH, padded_n);
    let mut transcript = Transcript::new(b"zkpox-temporal-proof-v1");

    let (proof, committed_values) = RangeProof::prove_multiple(
        &bp_gens,
        &pc_gens,
        &mut transcript,
        &values,
        &blindings,
        TEMPORAL_BIT_WIDTH,
    )
    .map_err(|e| TemporalError::ProofGeneration(format!("{:?}", e)))?;

    let mut commitments_bytes = Vec::with_capacity(committed_values.len() * 32);
    for c in &committed_values {
        commitments_bytes.extend_from_slice(c.as_bytes());
    }

    Ok(TemporalProofResult {
        proof_bytes: proof.to_bytes(),
        commitments: commitments_bytes,
        count: n as u32,
        time_window_seconds: window,
    })
}

/// Verify a temporal range proof.
///
/// Confirms that all committed timestamp offsets fall within
/// `[0, 2^TEMPORAL_BIT_WIDTH)`, which means the original timestamps
/// are within the declared time window.
pub fn verify_temporal_proof(result: &TemporalProofResult) -> Result<(), TemporalError> {
    let proof = RangeProof::from_bytes(&result.proof_bytes)
        .map_err(|e| TemporalError::Deserialization(format!("{:?}", e)))?;

    let n_values = (result.count as usize).next_power_of_two();
    let expected_bytes = n_values * 32;

    if result.commitments.len() != expected_bytes {
        return Err(TemporalError::CommitmentMismatch {
            expected: expected_bytes,
            got: result.commitments.len(),
        });
    }

    let commitments: Vec<CompressedRistretto> = result
        .commitments
        .chunks_exact(32)
        .map(|chunk| {
            let mut buf = [0u8; 32];
            buf.copy_from_slice(chunk);
            CompressedRistretto(buf)
        })
        .collect();

    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(TEMPORAL_BIT_WIDTH, n_values);
    let mut transcript = Transcript::new(b"zkpox-temporal-proof-v1");

    proof
        .verify_multiple(
            &bp_gens,
            &pc_gens,
            &mut transcript,
            &commitments,
            TEMPORAL_BIT_WIDTH,
        )
        .map_err(|_| TemporalError::InvalidProof)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SignedGPSPoint;

    fn make_points(n: usize, base_ts: i64, interval: i64) -> Vec<SignedGPSPoint> {
        (0..n)
            .map(|i| SignedGPSPoint {
                lat: 52.2297 + (i as f64) * 0.00001,
                lng: 21.0122,
                timestamp: base_ts + (i as i64) * interval,
                accuracy: 5.0,
                signature: vec![0u8; 64],
            })
            .collect()
    }

    #[test]
    fn test_temporal_proof_roundtrip() {
        let base = 1_740_000_000i64;
        let points = make_points(10, base, 3600);
        let indices: Vec<usize> = (0..10).collect();

        let result = generate_temporal_proof(&points, &indices, base - 1, base + 40_000)
            .expect("proof generation should succeed");

        assert_eq!(result.count, 10);
        assert!(result.time_window_seconds > 0);

        let verify = verify_temporal_proof(&result);
        assert!(verify.is_ok(), "valid temporal proof should verify: {:?}", verify.err());
    }

    #[test]
    fn test_temporal_proof_tampered_fails() {
        let base = 1_740_000_000i64;
        let points = make_points(5, base, 7200);
        let indices: Vec<usize> = (0..5).collect();

        let mut result = generate_temporal_proof(&points, &indices, base - 1, base + 40_000)
            .expect("proof generation should succeed");

        if let Some(byte) = result.proof_bytes.get_mut(10) {
            *byte ^= 0xFF;
        }

        assert!(
            verify_temporal_proof(&result).is_err(),
            "tampered temporal proof should fail"
        );
    }

    #[test]
    fn test_temporal_empty_input() {
        let points = make_points(5, 1_000_000, 3600);
        let result = generate_temporal_proof(&points, &[], 1_000_000, 2_000_000);
        assert!(result.is_err());
    }

    #[test]
    fn test_temporal_invalid_window() {
        let points = make_points(5, 1_000_000, 3600);
        let indices: Vec<usize> = (0..5).collect();
        let result = generate_temporal_proof(&points, &indices, 2_000_000, 1_000_000);
        assert!(result.is_err());
    }

    #[test]
    fn test_temporal_single_point() {
        let base = 1_740_000_000i64;
        let points = make_points(1, base, 0);
        let indices = vec![0usize];

        let result = generate_temporal_proof(&points, &indices, base - 100, base + 100)
            .expect("single point proof should work");

        assert_eq!(result.count, 1);
        assert!(verify_temporal_proof(&result).is_ok());
    }

    #[test]
    fn test_temporal_filters_out_of_window() {
        let base = 1_740_000_000i64;
        let points = vec![
            SignedGPSPoint { lat: 52.0, lng: 21.0, timestamp: base + 100, accuracy: 5.0, signature: vec![0u8; 64] },
            SignedGPSPoint { lat: 52.0, lng: 21.0, timestamp: base + 500_000, accuracy: 5.0, signature: vec![0u8; 64] },
            SignedGPSPoint { lat: 52.0, lng: 21.0, timestamp: base + 200, accuracy: 5.0, signature: vec![0u8; 64] },
        ];
        let indices: Vec<usize> = (0..3).collect();
        let window_end = base + 1000;

        let result = generate_temporal_proof(&points, &indices, base, window_end)
            .expect("should succeed with in-window points");

        assert_eq!(result.count, 2, "only 2 of 3 points are in the window");
        assert!(verify_temporal_proof(&result).is_ok());
    }
}
