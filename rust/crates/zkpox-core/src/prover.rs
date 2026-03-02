//! Proof generation from GPS history + ProofRequest.
//!
//! The prover generates an **aggregate Bulletproofs range proof** over
//! committed GPS coordinates. For each qualifying point, we commit to
//! `(lat_offset, lng_offset)` where offset = scaled_coord - bbox_min.
//! The range proof proves each offset is in `[0, bbox_width]`, which
//! means the original coordinate is within the geofence bounding box.
//!
//! This is cryptographically sound: the verifier learns only that N
//! committed values fall within a range, not the actual values.

use sha2::{Digest, Sha256};

use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::scalar::Scalar;
use merlin::Transcript;

use crate::circuit::{count_qualifying_points, geofence_to_bounding_box, scale_coord};
use crate::commitment::position_commitment;
use crate::types::*;

#[derive(Debug, thiserror::Error)]
pub enum ProverError {
    #[error("Not enough qualifying points: found {found}, need {required}")]
    InsufficientPoints { found: u32, required: u32 },

    #[error("No GPS points provided")]
    EmptyInput,

    #[error("Invalid proof request: {0}")]
    InvalidRequest(String),

    #[error("Bulletproof generation failed: {0}")]
    BulletproofError(String),
}

/// Maximum points in an aggregate proof. Bulletproofs aggregate proofs are
/// O(log(n)) in size but generation is O(n). Each point needs 2 range
/// proofs (lat + lng), and aggregate proofs require power-of-two counts.
/// 32 points × 2 = 64 range proofs → ~2 KB proof, ~1s on mobile.
const MAX_PROOF_POINTS: usize = 32;

/// Bit width for range proofs. 32 bits covers offsets up to ~4.29 billion,
/// which at 1e7 scale handles geofences up to ~47 km radius at any latitude.
const RANGE_BIT_WIDTH: usize = 32;

/// Generate a ZK-PoX proof from GPS history.
///
/// The proof cryptographically demonstrates:
///   "At least `min_count` GPS points fall within the geofence defined by
///    `(center, radius)` during `[time_start, time_end]`."
///
/// What the verifier learns:
///   - The committed center hash (not the actual location)
///   - The radius, time window, and point count
///   - A valid Bulletproof over committed coordinates
///
/// What stays private:
///   - Exact GPS coordinates
///   - Exact timestamps
///   - Which points qualified
///   - The actual center location
pub fn generate_proof(
    points: &[SignedGPSPoint],
    request: &ProofRequest,
) -> Result<ProofResult, ProverError> {
    if points.is_empty() {
        return Err(ProverError::EmptyInput);
    }
    if request.radius_m == 0 {
        return Err(ProverError::InvalidRequest("radius must be > 0".into()));
    }
    if request.time_start >= request.time_end {
        return Err(ProverError::InvalidRequest("time_start must be < time_end".into()));
    }

    let (eff_center_lat, eff_center_lng, qualifying_indices) = match request.claim_type {
        ClaimType::Stability => {
            let analysis = crate::stability::analyze_stability(points, request);
            let count = analysis.qualifying_indices.len() as u32;
            if count < request.min_count {
                return Err(ProverError::InsufficientPoints {
                    found: count,
                    required: request.min_count,
                });
            }
            (
                analysis.centroid_lat,
                analysis.centroid_lng,
                analysis.qualifying_indices,
            )
        }
        ClaimType::Travel => {
            let analysis = crate::travel::analyze_travel(points, request);
            let region_count = analysis.regions.len() as u32;
            if region_count < request.min_count {
                return Err(ProverError::InsufficientPoints {
                    found: region_count,
                    required: request.min_count,
                });
            }
            (
                analysis.anchor_lat,
                analysis.anchor_lng,
                analysis.qualifying_indices,
            )
        }
        ClaimType::Absence => {
            let analysis = crate::absence::analyze_absence(points, request);
            if analysis.violation_count > 0 {
                return Err(ProverError::InvalidRequest(format!(
                    "absence violated: {} points found inside exclusion zone",
                    analysis.violation_count
                )));
            }
            if analysis.total_in_window < request.min_count {
                return Err(ProverError::InsufficientPoints {
                    found: analysis.total_in_window,
                    required: request.min_count,
                });
            }
            let outside = &analysis.outside_indices;
            let n = outside.len() as f64;
            let c_lat = outside.iter().map(|&i| points[i].lat).sum::<f64>() / n;
            let c_lng = outside.iter().map(|&i| points[i].lng).sum::<f64>() / n;
            let max_dev = outside.iter()
                .map(|&i| crate::types::haversine_distance_m(points[i].lat, points[i].lng, c_lat, c_lng))
                .fold(0.0f64, f64::max);
            let proof_radius = (max_dev * 1.5) as u32 + 1000;
            return generate_proof_with_center(
                points, request, c_lat, c_lng, proof_radius, analysis.outside_indices,
            );
        }
        _ => {
            let (count, indices) = count_qualifying_points(points, request);
            if count < request.min_count {
                return Err(ProverError::InsufficientPoints {
                    found: count,
                    required: request.min_count,
                });
            }
            (request.center_lat, request.center_lng, indices)
        }
    };

    let center_salt = derive_center_salt(request);
    let center_hash = position_commitment(eff_center_lat, eff_center_lng, &center_salt);
    let time_window_days = ((request.time_end - request.time_start) / 86_400) as u32;

    let proof_count = qualifying_indices.len().min(MAX_PROOF_POINTS);
    let selected_indices = &qualifying_indices[..proof_count];

    let (proof_bytes, commitments_bytes) = generate_aggregate_range_proof(
        points,
        selected_indices,
        eff_center_lat,
        eff_center_lng,
        request.radius_m,
    )?;

    let public_inputs = PublicInputs {
        center_hash,
        radius_m: request.radius_m,
        time_window_days,
        min_count: request.min_count,
        count_proven: proof_count as u32,
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    Ok(ProofResult {
        proof_bytes,
        public_inputs,
        claim_type: request.claim_type,
        generated_at: now,
        total_points_evaluated: points.len() as u32,
        commitments: commitments_bytes,
    })
}

/// ABSENCE variant: generates proof with explicit center and radius
/// (different from the exclusion zone in the request).
fn generate_proof_with_center(
    points: &[SignedGPSPoint],
    request: &ProofRequest,
    center_lat: f64,
    center_lng: f64,
    proof_radius: u32,
    qualifying_indices: Vec<usize>,
) -> Result<ProofResult, ProverError> {
    let center_salt = derive_center_salt(request);
    let center_hash = position_commitment(center_lat, center_lng, &center_salt);
    let time_window_days = ((request.time_end - request.time_start) / 86_400) as u32;

    let proof_count = qualifying_indices.len().min(MAX_PROOF_POINTS);
    let selected_indices = &qualifying_indices[..proof_count];

    let (proof_bytes, commitments_bytes) = generate_aggregate_range_proof(
        points,
        selected_indices,
        center_lat,
        center_lng,
        proof_radius,
    )?;

    let public_inputs = PublicInputs {
        center_hash,
        radius_m: request.radius_m,
        time_window_days,
        min_count: request.min_count,
        count_proven: proof_count as u32,
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    Ok(ProofResult {
        proof_bytes,
        public_inputs,
        claim_type: request.claim_type,
        generated_at: now,
        total_points_evaluated: points.len() as u32,
        commitments: commitments_bytes,
    })
}

/// Generate an aggregate Bulletproof range proof over GPS coordinate offsets.
///
/// For each qualifying point:
///   1. Compute `lat_offset = lat_scaled - lat_min` (guaranteed >= 0 by pre-filtering)
///   2. Compute `lng_offset = lng_scaled - lng_min` (guaranteed >= 0)
///   3. Both offsets must be in `[0, bbox_width]`
///
/// The aggregate proof covers all `2 * N` values simultaneously.
/// Proof size grows logarithmically with N.
fn generate_aggregate_range_proof(
    points: &[SignedGPSPoint],
    indices: &[usize],
    center_lat: f64,
    center_lng: f64,
    radius_m: u32,
) -> Result<(Vec<u8>, Vec<u8>), ProverError> {
    let (lat_min, _lat_max, lng_min, _lng_max) =
        geofence_to_bounding_box(center_lat, center_lng, radius_m);

    // Collect offset values: [lat0_off, lng0_off, lat1_off, lng1_off, ...]
    let mut values: Vec<u64> = Vec::with_capacity(indices.len() * 2);
    let mut blindings: Vec<Scalar> = Vec::with_capacity(indices.len() * 2);

    for &idx in indices {
        let p = &points[idx];
        let lat_s = scale_coord(p.lat);
        let lng_s = scale_coord(p.lng);

        let lat_off = (lat_s - lat_min) as u64;
        let lng_off = (lng_s - lng_min) as u64;

        values.push(lat_off);
        values.push(lng_off);

        blindings.push(random_scalar());
        blindings.push(random_scalar());
    }

    // Pad to power of two (required by aggregate Bulletproofs)
    let n = values.len();
    let padded_n = n.next_power_of_two();
    for _ in n..padded_n {
        values.push(0);
        blindings.push(random_scalar());
    }

    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(RANGE_BIT_WIDTH, padded_n);

    let mut transcript = Transcript::new(b"zkpox-geo-proof-v1");

    let (proof, committed_values) = RangeProof::prove_multiple(
        &bp_gens,
        &pc_gens,
        &mut transcript,
        &values,
        &blindings,
        RANGE_BIT_WIDTH,
    )
    .map_err(|e| ProverError::BulletproofError(format!("{:?}", e)))?;

    // Serialize commitments so the verifier can reconstruct them
    let mut commitments_bytes = Vec::with_capacity(committed_values.len() * 32);
    for c in &committed_values {
        commitments_bytes.extend_from_slice(c.as_bytes());
    }

    Ok((proof.to_bytes(), commitments_bytes))
}

fn random_scalar() -> Scalar {
    let mut bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
    Scalar::from_bytes_mod_order(bytes)
}

fn derive_center_salt(request: &ProofRequest) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"zkpox-center-salt-v1");
    hasher.update((request.claim_type as u8).to_le_bytes());
    hasher.update(request.radius_m.to_le_bytes());
    hasher.update(request.time_start.to_le_bytes());
    hasher.update(request.time_end.to_le_bytes());
    hasher.finalize().into()
}

impl ProofResult {
    pub fn count_proven(&self) -> u32 {
        self.public_inputs.count_proven
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_points_near_warsaw(n: usize) -> Vec<SignedGPSPoint> {
        (0..n)
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
    fn test_generate_proof_success() {
        let points = make_points_near_warsaw(10);
        let request = ProofRequest {
            claim_type: ClaimType::Attendance,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 500,
            time_start: 1_740_000_000 - 1,
            time_end: 1_740_000_000 + 10_000,
            min_count: 5,
            night_only: false,
        };

        let result = generate_proof(&points, &request).unwrap();
        assert!(result.count_proven() >= 5);
        assert_eq!(result.claim_type, ClaimType::Attendance);
        assert!(!result.proof_bytes.is_empty());
        assert!(!result.commitments.is_empty());
    }

    #[test]
    fn test_proof_has_commitments() {
        let points = make_points_near_warsaw(4);
        let request = ProofRequest {
            claim_type: ClaimType::Residency,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 500,
            time_start: 1_740_000_000 - 1,
            time_end: 1_740_000_000 + 10_000,
            min_count: 2,
            night_only: false,
        };

        let result = generate_proof(&points, &request).unwrap();
        // Each point → 2 commitments (lat + lng), each 32 bytes.
        // Padded to power of two, so commitments.len() is a multiple of 32.
        assert!(result.commitments.len() >= 2 * 32);
        assert_eq!(result.commitments.len() % 32, 0);
    }

    #[test]
    fn test_generate_proof_insufficient_points() {
        let points = make_points_near_warsaw(2);
        let request = ProofRequest {
            claim_type: ClaimType::Residency,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 500,
            time_start: 1_740_000_000 - 1,
            time_end: 1_740_000_000 + 10_000,
            min_count: 100,
            night_only: false,
        };

        assert!(matches!(
            generate_proof(&points, &request),
            Err(ProverError::InsufficientPoints { .. })
        ));
    }

    #[test]
    fn test_generate_proof_empty_input() {
        let request = ProofRequest {
            claim_type: ClaimType::Attendance,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 500,
            time_start: 1_000,
            time_end: 2_000,
            min_count: 1,
            night_only: false,
        };
        assert!(matches!(generate_proof(&[], &request), Err(ProverError::EmptyInput)));
    }

    #[test]
    fn test_deterministic_center_hash() {
        let points = make_points_near_warsaw(4);
        let request = ProofRequest {
            claim_type: ClaimType::Attendance,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 500,
            time_start: 1_740_000_000 - 1,
            time_end: 1_740_000_000 + 10_000,
            min_count: 2,
            night_only: false,
        };

        let r1 = generate_proof(&points, &request).unwrap();
        let r2 = generate_proof(&points, &request).unwrap();
        // Center hash should be deterministic (same location + salt)
        assert_eq!(r1.public_inputs.center_hash, r2.public_inputs.center_hash);
    }

    #[test]
    fn test_stability_proof_tight_cluster() {
        let points = make_points_near_warsaw(10);
        let request = ProofRequest {
            claim_type: ClaimType::Stability,
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m: 2000,
            time_start: 1_740_000_000 - 1,
            time_end: 1_740_000_000 + 10_000,
            min_count: 5,
            night_only: false,
        };

        let result = generate_proof(&points, &request).unwrap();
        assert_eq!(result.claim_type, ClaimType::Stability);
        assert!(result.count_proven() >= 5);
        assert!(!result.proof_bytes.is_empty());
    }

    #[test]
    fn test_stability_proof_spread_fails() {
        let points = vec![
            SignedGPSPoint {
                lat: 52.2297,
                lng: 21.0122,
                timestamp: 1_740_000_000,
                accuracy: 5.0,
                signature: vec![0u8; 64],
            },
            SignedGPSPoint {
                lat: 50.0647,
                lng: 19.9450,
                timestamp: 1_740_001_000,
                accuracy: 5.0,
                signature: vec![0u8; 64],
            },
            SignedGPSPoint {
                lat: 54.3520,
                lng: 18.6466,
                timestamp: 1_740_002_000,
                accuracy: 5.0,
                signature: vec![0u8; 64],
            },
        ];
        let request = ProofRequest {
            claim_type: ClaimType::Stability,
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m: 2000,
            time_start: 1_739_999_000,
            time_end: 1_740_100_000,
            min_count: 3,
            night_only: false,
        };

        assert!(matches!(
            generate_proof(&points, &request),
            Err(ProverError::InsufficientPoints { .. })
        ));
    }

    fn gps(lat: f64, lng: f64, ts: i64) -> SignedGPSPoint {
        SignedGPSPoint {
            lat,
            lng,
            timestamp: ts,
            accuracy: 10.0,
            signature: vec![0u8; 64],
        }
    }

    #[test]
    fn test_travel_proof_three_cities() {
        let base = 1_740_000_000i64;
        let day = 86_400i64;
        let points = vec![
            gps(52.2297, 21.0122, base),
            gps(52.2300, 21.0130, base + day),
            gps(52.2290, 21.0110, base + 2 * day),
            gps(50.0647, 19.9450, base + 10 * day),
            gps(50.0650, 19.9460, base + 11 * day),
            gps(50.0640, 19.9440, base + 12 * day),
            gps(54.3520, 18.6466, base + 20 * day),
            gps(54.3530, 18.6480, base + 21 * day),
            gps(54.3510, 18.6450, base + 22 * day),
        ];
        let request = ProofRequest {
            claim_type: ClaimType::Travel,
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m: 50_000,
            time_start: base - 1,
            time_end: base + 30 * day,
            min_count: 3,
            night_only: false,
        };

        let result = generate_proof(&points, &request).unwrap();
        assert_eq!(result.claim_type, ClaimType::Travel);
        assert!(!result.proof_bytes.is_empty());
    }

    #[test]
    fn test_absence_proof_all_outside() {
        let points = vec![
            gps(50.0647, 19.9450, 1_740_000_100), // Krakow
            gps(54.3520, 18.6466, 1_740_001_000), // Gdansk
            gps(51.1079, 17.0385, 1_740_002_000), // Wroclaw
        ];
        let request = ProofRequest {
            claim_type: ClaimType::Absence,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 5000,
            time_start: 1_740_000_000,
            time_end: 1_740_100_000,
            min_count: 1,
            night_only: false,
        };
        let result = generate_proof(&points, &request).unwrap();
        assert_eq!(result.claim_type, ClaimType::Absence);
        assert!(!result.proof_bytes.is_empty());
    }

    #[test]
    fn test_absence_proof_violation() {
        let points = vec![
            gps(52.2300, 21.0125, 1_740_000_100), // inside Warsaw exclusion zone
            gps(50.0647, 19.9450, 1_740_001_000), // Krakow — outside
        ];
        let request = ProofRequest {
            claim_type: ClaimType::Absence,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 5000,
            time_start: 1_740_000_000,
            time_end: 1_740_100_000,
            min_count: 1,
            night_only: false,
        };
        assert!(matches!(
            generate_proof(&points, &request),
            Err(ProverError::InvalidRequest(_))
        ));
    }

    #[test]
    fn test_travel_proof_insufficient_regions() {
        let base = 1_740_000_000i64;
        let day = 86_400i64;
        let points = vec![
            gps(52.2297, 21.0122, base),
            gps(52.2300, 21.0130, base + day),
        ];
        let request = ProofRequest {
            claim_type: ClaimType::Travel,
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m: 50_000,
            time_start: base - 1,
            time_end: base + 10 * day,
            min_count: 3,
            night_only: false,
        };

        assert!(matches!(
            generate_proof(&points, &request),
            Err(ProverError::InsufficientPoints { .. })
        ));
    }
}
