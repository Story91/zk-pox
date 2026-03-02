//! Anti-spoofing: temporal and spatial consistency analysis.
//!
//! Detects GPS spoofing by analyzing the trajectory for physically
//! implausible patterns. A real phone has natural noise, gradual
//! movement, and realistic velocity. A spoofed feed shows:
//!   - Teleportation (instant jumps > reasonable velocity)
//!   - Zero noise (unnaturally precise, constant coordinates)
//!   - Impossible velocity between consecutive points
//!   - Statistically uniform distribution (no natural clustering)
//!
//! This module runs BEFORE proof generation. If anomalies exceed
//! the threshold, proof generation is refused.

use crate::types::SignedGPSPoint;

/// Result of anti-spoofing analysis.
#[derive(Debug, Clone)]
pub struct SpoofAnalysis {
    pub total_points: usize,
    pub teleport_count: usize,
    pub zero_noise_count: usize,
    pub impossible_velocity_count: usize,
    pub suspicion_score: f64,
    pub verdict: SpoofVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpoofVerdict {
    Clean,
    Suspicious,
    LikelySpoofed,
}

/// Max plausible speed in m/s (300 km/h ≈ 83 m/s — covers highway + trains).
const MAX_VELOCITY_MS: f64 = 83.0;

/// Points with GPS accuracy worse than this are excluded from analysis.
const MAX_USEFUL_ACCURACY_M: f32 = 100.0;

/// A jump of more than this many meters in one interval is a teleport.
const TELEPORT_THRESHOLD_M: f64 = 50_000.0;

/// GPS coordinates that don't vary by at least this much (in degrees)
/// across 10+ consecutive points suggest mock location (zero noise).
const MIN_NOISE_DEG: f64 = 0.000005; // ~0.5 meters

/// Score thresholds.
const SUSPICIOUS_THRESHOLD: f64 = 0.15;
const SPOOFED_THRESHOLD: f64 = 0.35;

/// Analyze a GPS trajectory for spoofing indicators.
///
/// Points must be sorted by timestamp (ascending).
pub fn analyze(points: &[SignedGPSPoint]) -> SpoofAnalysis {
    if points.len() < 2 {
        return SpoofAnalysis {
            total_points: points.len(),
            teleport_count: 0,
            zero_noise_count: 0,
            impossible_velocity_count: 0,
            suspicion_score: 0.0,
            verdict: SpoofVerdict::Clean,
        };
    }

    let usable: Vec<&SignedGPSPoint> = points
        .iter()
        .filter(|p| p.accuracy <= MAX_USEFUL_ACCURACY_M)
        .collect();

    let n = usable.len();
    if n < 2 {
        return SpoofAnalysis {
            total_points: points.len(),
            teleport_count: 0,
            zero_noise_count: 0,
            impossible_velocity_count: 0,
            suspicion_score: 0.0,
            verdict: SpoofVerdict::Clean,
        };
    }

    let mut teleport_count = 0usize;
    let mut impossible_velocity_count = 0usize;

    for pair in usable.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let dt = (b.timestamp - a.timestamp).max(1) as f64;
        let dist = haversine_m(a.lat, a.lng, b.lat, b.lng);
        let velocity = dist / dt;

        if dist > TELEPORT_THRESHOLD_M {
            teleport_count += 1;
        } else if velocity > MAX_VELOCITY_MS {
            impossible_velocity_count += 1;
        }
    }

    // Zero-noise detection: check for runs of identical coordinates
    let zero_noise_count = count_zero_noise_runs(&usable);

    // Compute suspicion score (0.0 = clean, 1.0 = definitely spoofed)
    let pairs = (n - 1) as f64;
    let teleport_ratio = teleport_count as f64 / pairs;
    let velocity_ratio = impossible_velocity_count as f64 / pairs;
    let noise_ratio = zero_noise_count as f64 / n as f64;

    let suspicion_score = (teleport_ratio * 3.0 + velocity_ratio * 2.0 + noise_ratio * 1.5)
        .min(1.0);

    let verdict = if suspicion_score >= SPOOFED_THRESHOLD {
        SpoofVerdict::LikelySpoofed
    } else if suspicion_score >= SUSPICIOUS_THRESHOLD {
        SpoofVerdict::Suspicious
    } else {
        SpoofVerdict::Clean
    };

    SpoofAnalysis {
        total_points: points.len(),
        teleport_count,
        zero_noise_count,
        impossible_velocity_count,
        suspicion_score,
        verdict,
    }
}

/// Count points that are part of "zero noise" runs — sequences of 5+
/// consecutive points where lat and lng don't vary beyond MIN_NOISE_DEG.
fn count_zero_noise_runs(points: &[&SignedGPSPoint]) -> usize {
    if points.len() < 5 {
        return 0;
    }

    let mut total_suspicious = 0usize;
    let mut run_start = 0usize;

    for i in 1..points.len() {
        let d_lat = (points[i].lat - points[run_start].lat).abs();
        let d_lng = (points[i].lng - points[run_start].lng).abs();

        if d_lat > MIN_NOISE_DEG || d_lng > MIN_NOISE_DEG {
            // End of low-noise run
            let run_len = i - run_start;
            if run_len >= 5 {
                total_suspicious += run_len;
            }
            run_start = i;
        }
    }

    // Check final run
    let run_len = points.len() - run_start;
    if run_len >= 5 {
        total_suspicious += run_len;
    }

    total_suspicious
}

fn haversine_m(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    R * 2.0 * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(lat: f64, lng: f64, ts: i64) -> SignedGPSPoint {
        SignedGPSPoint {
            lat, lng, timestamp: ts, accuracy: 5.0, signature: vec![0u8; 64],
        }
    }

    #[test]
    fn test_clean_trajectory() {
        // Natural walking trajectory: ~1m/s, slight GPS noise
        let points: Vec<SignedGPSPoint> = (0..50)
            .map(|i| pt(
                52.2297 + (i as f64) * 0.00001 + (i as f64 * 0.7).sin() * 0.000003,
                21.0122 + (i as f64) * 0.00001 + (i as f64 * 1.3).cos() * 0.000003,
                1_740_000_000 + i * 300,
            ))
            .collect();

        let result = analyze(&points);
        assert_eq!(result.verdict, SpoofVerdict::Clean);
        assert_eq!(result.teleport_count, 0);
        assert_eq!(result.impossible_velocity_count, 0);
    }

    #[test]
    fn test_teleportation_detected() {
        let points = vec![
            pt(52.2297, 21.0122, 1_740_000_000),
            pt(52.2298, 21.0123, 1_740_000_300),
            // Teleport to Paris
            pt(48.8566, 2.3522, 1_740_000_600),
            pt(48.8567, 2.3523, 1_740_000_900),
        ];

        let result = analyze(&points);
        assert!(result.teleport_count > 0);
        assert!(result.suspicion_score > 0.0);
    }

    #[test]
    fn test_impossible_velocity() {
        let points = vec![
            pt(52.2297, 21.0122, 1_740_000_000),
            // ~10 km in 1 second = 10,000 m/s
            pt(52.3200, 21.0122, 1_740_000_001),
        ];

        let result = analyze(&points);
        assert!(result.impossible_velocity_count > 0 || result.teleport_count > 0);
    }

    #[test]
    fn test_zero_noise_detected() {
        // 20 points at exactly the same location (mock GPS)
        let points: Vec<SignedGPSPoint> = (0..20)
            .map(|i| pt(52.2297000, 21.0122000, 1_740_000_000 + i * 300))
            .collect();

        let result = analyze(&points);
        assert!(result.zero_noise_count > 0);
        assert!(result.suspicion_score > 0.0);
    }

    #[test]
    fn test_empty_and_single() {
        assert_eq!(analyze(&[]).verdict, SpoofVerdict::Clean);
        assert_eq!(analyze(&[pt(52.0, 21.0, 1000)]).verdict, SpoofVerdict::Clean);
    }
}
