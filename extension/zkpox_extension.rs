//! ZK-PoX ADVERTISE extension formatter.
//!
//! Converts a ProofResult into a JSON payload suitable for the
//! `extensions.zk_pox` field in ADVERTISE messages. The core node
//! broadcasts this as opaque JSON — no ZK dependencies required.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The extension payload that goes into ADVERTISE.extensions.zk_pox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkPoxExtension {
    /// Claim type: "RESIDENCY", "ATTENDANCE", "STABILITY", etc.
    pub proof_type: String,
    /// Geofence radius used in the proof.
    pub radius_m: u32,
    /// Time window in days.
    pub time_window_days: u32,
    /// Number of GPS points cryptographically proven.
    pub count_proven: u32,
    /// SHA-256 hash of the Bulletproof bytes (hex).
    pub proof_hash: String,
    /// Base64-encoded Bulletproof bytes (~2KB).
    pub proof_bytes_b64: String,
    /// Base64-encoded Pedersen commitments.
    pub commitments_b64: String,
    /// SHA-256 hash of the committed center position (hex).
    pub center_hash: String,
}

/// Format a proof result into an extension payload.
///
/// This is called on the mobile side after proof generation.
/// The result is serialized to JSON and injected into the
/// agent's ADVERTISE extensions field.
pub fn format_extension(
    claim_type_str: &str,
    radius_m: u32,
    time_window_days: u32,
    count_proven: u32,
    proof_bytes: &[u8],
    commitments: &[u8],
    center_hash: &[u8; 32],
) -> ZkPoxExtension {
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;

    let mut hasher = Sha256::new();
    hasher.update(proof_bytes);
    let p_hash: [u8; 32] = hasher.finalize().into();

    ZkPoxExtension {
        proof_type: claim_type_str.to_string(),
        radius_m,
        time_window_days,
        count_proven,
        proof_hash: hex::encode(p_hash),
        proof_bytes_b64: engine.encode(proof_bytes),
        commitments_b64: engine.encode(commitments),
        center_hash: hex::encode(center_hash),
    }
}

/// Build the full ADVERTISE extensions JSON fragment.
///
/// Returns a `serde_json::Value` that can be merged into the
/// agent's ADVERTISE payload under the `extensions` key.
pub fn build_extensions_json(ext: &ZkPoxExtension) -> serde_json::Value {
    serde_json::json!({
        "zk_pox": ext
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_extension() {
        let ext = format_extension(
            "RESIDENCY",
            2000,
            30,
            28,
            b"fake-proof-bytes",
            b"fake-commitments",
            &[0xAA; 32],
        );
        assert_eq!(ext.proof_type, "RESIDENCY");
        assert_eq!(ext.radius_m, 2000);
        assert_eq!(ext.count_proven, 28);
        assert!(!ext.proof_hash.is_empty());
        assert!(!ext.proof_bytes_b64.is_empty());
    }

    #[test]
    fn test_build_extensions_json() {
        let ext = format_extension(
            "STABILITY",
            5000,
            90,
            85,
            b"proof",
            b"commitments",
            &[0xBB; 32],
        );
        let json = build_extensions_json(&ext);
        assert!(json["zk_pox"]["proof_type"].as_str() == Some("STABILITY"));
        assert!(json["zk_pox"]["radius_m"].as_u64() == Some(5000));
    }
}
