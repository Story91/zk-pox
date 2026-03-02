use serde::{Deserialize, Serialize};

/// GPS coordinate point captured by the phone, signed with the agent's Ed25519 key.
/// The signature covers `SHA-256(lat_le_bytes || lng_le_bytes || timestamp_le_bytes)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedGPSPoint {
    pub lat: f64,
    pub lng: f64,
    pub timestamp: i64,
    pub accuracy: f32,
    /// Ed25519 signature bytes (64 bytes). Stored as Vec for serde compatibility.
    pub signature: Vec<u8>,
}

/// Supported claim types for ZK-PoX proofs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ClaimType {
    /// "I was near location H for N+ nights in period P"
    Residency = 0,
    /// "I traveled between A and B, D days/week, for W weeks"
    Commute = 1,
    /// "I was within R meters of E for T+ hours on date D"
    Attendance = 2,
    /// "I was NOT within R meters of X during period P"
    Absence = 3,
    /// "My location variance is below threshold T over period P"
    Stability = 4,
    /// "I was in N distinct geographic regions during period P"
    Travel = 5,
}

impl ClaimType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Residency => "RESIDENCY",
            Self::Commute => "COMMUTE",
            Self::Attendance => "ATTENDANCE",
            Self::Absence => "ABSENCE",
            Self::Stability => "STABILITY",
            Self::Travel => "TRAVEL",
        }
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Residency),
            1 => Some(Self::Commute),
            2 => Some(Self::Attendance),
            3 => Some(Self::Absence),
            4 => Some(Self::Stability),
            5 => Some(Self::Travel),
            _ => None,
        }
    }
}

/// A request to generate a ZK proof from local GPS history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRequest {
    pub claim_type: ClaimType,
    /// Center latitude of the geofence (private — never sent over the wire).
    pub center_lat: f64,
    /// Center longitude of the geofence (private).
    pub center_lng: f64,
    /// Geofence radius in meters.
    pub radius_m: u32,
    /// Start of the time window (unix timestamp, inclusive).
    pub time_start: i64,
    /// End of the time window (unix timestamp, inclusive).
    pub time_end: i64,
    /// Minimum number of qualifying points required for the claim to hold.
    pub min_count: u32,
    /// For RESIDENCY: only count points in nighttime hours (22:00–07:00 local).
    #[serde(default)]
    pub night_only: bool,
}

/// Public inputs embedded in the ZK proof — these ARE revealed to the verifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    /// SHA-256 hash of the committed center position (hides exact location).
    pub center_hash: [u8; 32],
    /// Geofence radius in meters.
    pub radius_m: u32,
    /// Length of the time window in days.
    pub time_window_days: u32,
    /// Minimum qualifying points required by the claim.
    pub min_count: u32,
    /// Actual count of qualifying points proven (>= min_count).
    pub count_proven: u32,
}

/// The result of proof generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResult {
    /// Serialized Bulletproof bytes.
    pub proof_bytes: Vec<u8>,
    /// Public inputs that accompany the proof.
    pub public_inputs: PublicInputs,
    /// Which claim type this proof covers.
    pub claim_type: ClaimType,
    /// Unix timestamp when the proof was generated.
    pub generated_at: i64,
    /// Number of raw GPS points that were evaluated.
    pub total_points_evaluated: u32,
    /// Serialized Pedersen commitments (32 bytes each, concatenated).
    /// These are needed by the verifier to cryptographically verify
    /// the range proof against committed coordinate offsets.
    pub commitments: Vec<u8>,
}

/// On-chain credential stored as a soulbound token on the SATI NFT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceCredential {
    pub version: u8,
    pub agent_id: [u8; 32],
    pub claim_type: ClaimType,
    pub proof_hash: [u8; 32],
    pub public_inputs_hash: [u8; 32],
    pub witness_count: u8,
    pub issued_at: i64,
    pub revoked: bool,
}

/// Haversine distance between two GPS coordinates in meters.
pub fn haversine_distance_m(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const R: f64 = 6_371_000.0; // Earth radius in meters
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    R * c
}

/// Check whether a GPS point falls within a circular geofence.
pub fn point_in_geofence(
    point_lat: f64,
    point_lng: f64,
    center_lat: f64,
    center_lng: f64,
    radius_m: u32,
) -> bool {
    haversine_distance_m(point_lat, point_lng, center_lat, center_lng) <= radius_m as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_zero_distance() {
        let d = haversine_distance_m(52.2297, 21.0122, 52.2297, 21.0122);
        assert!(d < 0.01);
    }

    #[test]
    fn test_haversine_known_distance() {
        // Warsaw center to Warsaw Chopin Airport ≈ 8.5 km
        let d = haversine_distance_m(52.2297, 21.0122, 52.1657, 20.9671);
        assert!(d > 7_000.0 && d < 10_000.0);
    }

    #[test]
    fn test_point_in_geofence() {
        assert!(point_in_geofence(52.2297, 21.0122, 52.2300, 21.0125, 200));
        assert!(!point_in_geofence(52.2297, 21.0122, 52.2500, 21.0500, 200));
    }

    #[test]
    fn test_claim_type_roundtrip() {
        for v in 0..=5u8 {
            let ct = ClaimType::from_u8(v).unwrap();
            assert_eq!(ct as u8, v);
        }
        assert!(ClaimType::from_u8(6).is_none());
    }
}
