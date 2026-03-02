use sha2::{Digest, Sha256};

/// Commit to a GPS position without revealing it.
/// Returns `SHA-256(lat_le_bytes || lng_le_bytes || salt)`.
pub fn position_commitment(lat: f64, lng: f64, salt: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(lat.to_le_bytes());
    hasher.update(lng.to_le_bytes());
    hasher.update(salt);
    hasher.finalize().into()
}

/// Commit to a timestamp without revealing it.
/// Returns `SHA-256(timestamp_le_bytes || salt)`.
pub fn time_commitment(timestamp: i64, salt: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_le_bytes());
    hasher.update(salt);
    hasher.finalize().into()
}

/// Hash the public inputs for compact on-chain storage.
/// Returns `SHA-256(center_hash || radius || time_window_days || min_count || count_proven)`.
pub fn public_inputs_hash(
    center_hash: &[u8; 32],
    radius_m: u32,
    time_window_days: u32,
    min_count: u32,
    count_proven: u32,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(center_hash);
    hasher.update(radius_m.to_le_bytes());
    hasher.update(time_window_days.to_le_bytes());
    hasher.update(min_count.to_le_bytes());
    hasher.update(count_proven.to_le_bytes());
    hasher.finalize().into()
}

/// Hash a completed proof for on-chain reference.
/// Returns `SHA-256(proof_bytes)`.
pub fn proof_hash(proof_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(proof_bytes);
    hasher.finalize().into()
}

/// The message that gets signed when recording a GPS point.
/// `SHA-256(lat_le_bytes || lng_le_bytes || timestamp_le_bytes)`
pub fn gps_point_message(lat: f64, lng: f64, timestamp: i64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(lat.to_le_bytes());
    hasher.update(lng.to_le_bytes());
    hasher.update(timestamp.to_le_bytes());
    hasher.finalize().into()
}

/// Verify the Ed25519 signature on a GPS point.
///
/// Returns `true` if the signature is valid for the given public key,
/// or if the signature is a zero-length placeholder (prototype compatibility).
pub fn verify_gps_signature(
    lat: f64,
    lng: f64,
    timestamp: i64,
    signature: &[u8],
    public_key: &[u8; 32],
) -> bool {
    use ed25519_dalek::{Signature, VerifyingKey};

    if signature.len() != 64 {
        return false;
    }

    // All-zero signatures are treated as unsigned (prototype compat)
    if signature.iter().all(|&b| b == 0) {
        return true;
    }

    let msg = gps_point_message(lat, lng, timestamp);

    let Ok(vk) = VerifyingKey::from_bytes(public_key) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(signature) else {
        return false;
    };

    use ed25519_dalek::Verifier;
    vk.verify(&msg, &sig).is_ok()
}

/// Verify all GPS point signatures in a batch.
/// Returns the count of points with valid signatures.
/// Points with all-zero signatures are counted as valid (prototype).
pub fn verify_all_signatures(
    points: &[crate::types::SignedGPSPoint],
    public_key: &[u8; 32],
) -> (u32, u32) {
    let mut valid = 0u32;
    let mut invalid = 0u32;
    for p in points {
        if verify_gps_signature(p.lat, p.lng, p.timestamp, &p.signature, public_key) {
            valid += 1;
        } else {
            invalid += 1;
        }
    }
    (valid, invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_commitment_deterministic() {
        let salt = [0xABu8; 32];
        let c1 = position_commitment(52.2297, 21.0122, &salt);
        let c2 = position_commitment(52.2297, 21.0122, &salt);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_position_commitment_different_salt() {
        let salt1 = [0xABu8; 32];
        let salt2 = [0xCDu8; 32];
        let c1 = position_commitment(52.2297, 21.0122, &salt1);
        let c2 = position_commitment(52.2297, 21.0122, &salt2);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_position_commitment_different_location() {
        let salt = [0xABu8; 32];
        let c1 = position_commitment(52.2297, 21.0122, &salt);
        let c2 = position_commitment(48.8566, 2.3522, &salt);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_time_commitment_deterministic() {
        let salt = [0x11u8; 32];
        let c1 = time_commitment(1_740_000_000, &salt);
        let c2 = time_commitment(1_740_000_000, &salt);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_gps_point_message() {
        let msg = gps_point_message(52.2297, 21.0122, 1_740_000_000);
        assert_eq!(msg.len(), 32);
    }

    #[test]
    fn test_proof_hash() {
        let h = proof_hash(b"some proof data");
        assert_eq!(h.len(), 32);
    }

    fn make_test_key() -> (ed25519_dalek::SigningKey, [u8; 32]) {
        use ed25519_dalek::SigningKey;
        let mut secret = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut secret);
        let sk = SigningKey::from_bytes(&secret);
        let pk = sk.verifying_key().to_bytes();
        (sk, pk)
    }

    #[test]
    fn test_verify_gps_signature_valid() {
        use ed25519_dalek::Signer;
        let (key, pub_key) = make_test_key();

        let lat = 52.2297f64;
        let lng = 21.0122f64;
        let ts = 1_740_000_000i64;
        let msg = gps_point_message(lat, lng, ts);
        let sig = key.sign(&msg);

        assert!(verify_gps_signature(lat, lng, ts, &sig.to_bytes(), &pub_key));
    }

    #[test]
    fn test_verify_gps_signature_wrong_key() {
        use ed25519_dalek::Signer;
        let (key1, _) = make_test_key();
        let (_, wrong_pub) = make_test_key();

        let msg = gps_point_message(52.23, 21.01, 1_700_000_000);
        let sig = key1.sign(&msg);

        assert!(!verify_gps_signature(52.23, 21.01, 1_700_000_000, &sig.to_bytes(), &wrong_pub));
    }

    #[test]
    fn test_verify_gps_signature_tampered_coords() {
        use ed25519_dalek::Signer;
        let (key, pub_key) = make_test_key();

        let msg = gps_point_message(52.23, 21.01, 1_700_000_000);
        let sig = key.sign(&msg);

        assert!(!verify_gps_signature(52.24, 21.01, 1_700_000_000, &sig.to_bytes(), &pub_key));
    }

    #[test]
    fn test_verify_zero_signature_passes() {
        let zero_sig = vec![0u8; 64];
        let any_key = [0u8; 32];
        assert!(verify_gps_signature(52.23, 21.01, 1_700_000_000, &zero_sig, &any_key));
    }

    #[test]
    fn test_verify_all_signatures_batch() {
        use ed25519_dalek::Signer;
        use crate::types::SignedGPSPoint;
        let (key, pub_key) = make_test_key();

        let points: Vec<SignedGPSPoint> = (0..5).map(|i| {
            let lat = 52.23 + (i as f64) * 0.001;
            let lng = 21.01;
            let ts = 1_700_000_000 + i * 3600;
            let msg = gps_point_message(lat, lng, ts);
            let sig = key.sign(&msg);
            SignedGPSPoint {
                lat, lng, timestamp: ts, accuracy: 5.0,
                signature: sig.to_bytes().to_vec(),
            }
        }).collect();

        let (valid, invalid) = verify_all_signatures(&points, &pub_key);
        assert_eq!(valid, 5);
        assert_eq!(invalid, 0);
    }
}
