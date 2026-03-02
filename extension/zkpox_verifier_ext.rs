//! ZK-PoX extension verifier for receiving agents.
//!
//! When an agent receives an ADVERTISE message with extensions.zk_pox,
//! this module extracts the proof and verifies it using zkpox-core.
//! This runs on the RECEIVING side only — the broadcasting node
//! never touches this code.
//!
//! Dependencies: zkpox-core, base64, hex, serde_json

use serde_json::Value;

extern crate zkpox_core;

/// Result of verifying a ZK-PoX extension from an ADVERTISE message.
#[derive(Debug)]
pub enum ExtensionVerifyResult {
    /// No zk_pox extension present — not a ZK-PoX agent.
    NotPresent,
    /// Extension present but malformed.
    Malformed(String),
    /// Proof verified successfully.
    Valid {
        proof_type: String,
        count_proven: u32,
        time_window_days: u32,
    },
    /// Proof present but cryptographically invalid.
    Invalid(String),
}

/// Check if an ADVERTISE payload contains a ZK-PoX extension.
pub fn has_zkpox_extension(advertise_json: &Value) -> bool {
    advertise_json
        .get("extensions")
        .and_then(|e| e.get("zk_pox"))
        .is_some()
}

/// Extract and verify the ZK-PoX extension from an ADVERTISE payload.
///
/// In production, this calls `zkpox_core::verifier::verify_proof()`
/// on the decoded Bulletproof bytes and commitments. For now,
/// it performs structural validation.
pub fn verify_extension(advertise_json: &Value) -> ExtensionVerifyResult {
    let ext = match advertise_json
        .get("extensions")
        .and_then(|e| e.get("zk_pox"))
    {
        Some(v) => v,
        None => return ExtensionVerifyResult::NotPresent,
    };

    let proof_type = match ext.get("proof_type").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return ExtensionVerifyResult::Malformed("missing proof_type".into()),
    };

    let count_proven = match ext.get("count_proven").and_then(|v| v.as_u64()) {
        Some(n) => n as u32,
        None => return ExtensionVerifyResult::Malformed("missing count_proven".into()),
    };

    let time_window_days = match ext.get("time_window_days").and_then(|v| v.as_u64()) {
        Some(n) => n as u32,
        None => return ExtensionVerifyResult::Malformed("missing time_window_days".into()),
    };

    let proof_b64 = match ext.get("proof_bytes_b64").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return ExtensionVerifyResult::Malformed("missing proof_bytes_b64".into()),
    };

    let commitments_b64 = match ext.get("commitments_b64").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return ExtensionVerifyResult::Malformed("missing commitments_b64".into()),
    };

    let center_hash_hex = match ext.get("center_hash").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return ExtensionVerifyResult::Malformed("missing center_hash".into()),
    };

    let radius_m = match ext.get("radius_m").and_then(|v| v.as_u64()) {
        Some(n) => n as u32,
        None => return ExtensionVerifyResult::Malformed("missing radius_m".into()),
    };

    // Decode base64 payloads
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;

    let proof_bytes = match engine.decode(proof_b64) {
        Ok(b) => b,
        Err(e) => return ExtensionVerifyResult::Malformed(format!("proof_bytes_b64 decode: {e}")),
    };

    let commitments = match engine.decode(commitments_b64) {
        Ok(b) => b,
        Err(e) => return ExtensionVerifyResult::Malformed(format!("commitments_b64 decode: {e}")),
    };

    let center_hash: [u8; 32] = match hex::decode(center_hash_hex) {
        Ok(b) if b.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&b);
            arr
        }
        Ok(b) => return ExtensionVerifyResult::Malformed(
            format!("center_hash wrong length: expected 32, got {}", b.len()),
        ),
        Err(e) => return ExtensionVerifyResult::Malformed(format!("center_hash hex decode: {e}")),
    };

    // Reconstruct ProofResult for cryptographic verification
    let proof_result = zkpox_core::types::ProofResult {
        proof_bytes,
        public_inputs: zkpox_core::types::PublicInputs {
            center_hash,
            radius_m,
            time_window_days,
            min_count: 1,
            count_proven,
        },
        claim_type: match proof_type.as_str() {
            "RESIDENCY" => zkpox_core::types::ClaimType::Residency,
            "COMMUTE" => zkpox_core::types::ClaimType::Commute,
            "ATTENDANCE" => zkpox_core::types::ClaimType::Attendance,
            "ABSENCE" => zkpox_core::types::ClaimType::Absence,
            "STABILITY" => zkpox_core::types::ClaimType::Stability,
            "TRAVEL" => zkpox_core::types::ClaimType::Travel,
            _ => return ExtensionVerifyResult::Invalid(format!("unknown claim type: {proof_type}")),
        },
        generated_at: 0,
        total_points_evaluated: count_proven,
        commitments,
    };

    match zkpox_core::verifier::verify_proof(&proof_result) {
        Ok(()) => ExtensionVerifyResult::Valid {
            proof_type,
            count_proven,
            time_window_days,
        },
        Err(e) => ExtensionVerifyResult::Invalid(format!("{e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_no_extension() {
        let msg = json!({ "services": ["delivery"], "geo": { "country": "PL" } });
        assert!(!has_zkpox_extension(&msg));
        assert!(matches!(verify_extension(&msg), ExtensionVerifyResult::NotPresent));
    }

    #[test]
    fn test_valid_extension() {
        let msg = json!({
            "services": ["delivery"],
            "geo": { "country": "PL", "city": "Warsaw" },
            "extensions": {
                "zk_pox": {
                    "proof_type": "RESIDENCY",
                    "radius_m": 2000,
                    "time_window_days": 30,
                    "count_proven": 28,
                    "proof_hash": "abcdef",
                    "proof_bytes_b64": "ZmFrZQ==",
                    "commitments_b64": "ZmFrZQ==",
                    "center_hash": "123456"
                }
            }
        });
        assert!(has_zkpox_extension(&msg));
        match verify_extension(&msg) {
            ExtensionVerifyResult::Valid { proof_type, count_proven, time_window_days } => {
                assert_eq!(proof_type, "RESIDENCY");
                assert_eq!(count_proven, 28);
                assert_eq!(time_window_days, 30);
            }
            other => panic!("expected Valid, got {:?}", other),
        }
    }

    #[test]
    fn test_malformed_extension() {
        let msg = json!({
            "extensions": {
                "zk_pox": {
                    "proof_type": "RESIDENCY"
                    // missing required fields
                }
            }
        });
        assert!(matches!(verify_extension(&msg), ExtensionVerifyResult::Malformed(_)));
    }
}
