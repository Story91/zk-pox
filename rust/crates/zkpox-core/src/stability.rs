//! STABILITY claim: prove location variance is below a threshold.
//!
//! Unlike ATTENDANCE/RESIDENCY (which prove "I was at location X"),
//! STABILITY proves "my GPS points cluster tightly" — the centroid
//! is computed from the data, not supplied by the user.
//!
//! Use case: DePIN coverage proof — prove you cover a region with
//! low variance without revealing your exact address.

use crate::types::{haversine_distance_m, ProofRequest, SignedGPSPoint};

/// Result of stability analysis over a GPS trajectory.
#[derive(Debug, Clone)]
pub struct StabilityAnalysis {
    /// Computed centroid latitude (average of qualifying points).
    pub centroid_lat: f64,
    /// Computed centroid longitude.
    pub centroid_lng: f64,
    /// Maximum distance from any qualifying point to the centroid (meters).
    pub max_deviation_m: f64,
    /// Average distance from qualifying points to centroid (meters).
    pub avg_deviation_m: f64,
    /// Total points that fell within the time window.
    pub total_in_window: u32,
    /// Indices of points within both time window AND radius_m of centroid.
    pub qualifying_indices: Vec<usize>,
}

/// Analyze GPS points for spatial stability.
///
/// 1. Filter points by time window
/// 2. Compute centroid (arithmetic mean of lat/lng)
/// 3. Compute deviation metrics (max, avg distance from centroid)
/// 4. Select points within `radius_m` of centroid as qualifying
///
/// For the prover, `radius_m` acts as the max allowed deviation —
/// the proof demonstrates "all qualifying points are within radius_m
/// of the computed centroid."
pub fn analyze_stability(
    points: &[SignedGPSPoint],
    request: &ProofRequest,
) -> StabilityAnalysis {
    let time_filtered: Vec<(usize, &SignedGPSPoint)> = points
        .iter()
        .enumerate()
        .filter(|(_, p)| p.timestamp >= request.time_start && p.timestamp <= request.time_end)
        .collect();

    if time_filtered.is_empty() {
        return StabilityAnalysis {
            centroid_lat: 0.0,
            centroid_lng: 0.0,
            max_deviation_m: 0.0,
            avg_deviation_m: 0.0,
            total_in_window: 0,
            qualifying_indices: vec![],
        };
    }

    let n = time_filtered.len() as f64;
    let centroid_lat = time_filtered.iter().map(|(_, p)| p.lat).sum::<f64>() / n;
    let centroid_lng = time_filtered.iter().map(|(_, p)| p.lng).sum::<f64>() / n;

    let mut max_deviation_m = 0.0f64;
    let mut total_deviation_m = 0.0f64;
    let mut qualifying_indices = Vec::new();

    for &(i, p) in &time_filtered {
        let d = haversine_distance_m(p.lat, p.lng, centroid_lat, centroid_lng);
        if d > max_deviation_m {
            max_deviation_m = d;
        }
        total_deviation_m += d;

        if d <= request.radius_m as f64 {
            qualifying_indices.push(i);
        }
    }

    let avg_deviation_m = total_deviation_m / n;

    StabilityAnalysis {
        centroid_lat,
        centroid_lng,
        max_deviation_m,
        avg_deviation_m,
        total_in_window: time_filtered.len() as u32,
        qualifying_indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ClaimType;

    fn make_stable_points(n: usize) -> Vec<SignedGPSPoint> {
        (0..n)
            .map(|i| SignedGPSPoint {
                lat: 52.2297 + (i as f64) * 0.0001,
                lng: 21.0122 + (i as f64) * 0.0001,
                timestamp: 1_700_000_000 + (i as i64) * 3600,
                accuracy: 10.0,
                signature: vec![0u8; 64],
            })
            .collect()
    }

    fn make_unstable_points() -> Vec<SignedGPSPoint> {
        vec![
            SignedGPSPoint {
                lat: 52.2297,
                lng: 21.0122,
                timestamp: 1_700_000_000,
                accuracy: 10.0,
                signature: vec![0u8; 64],
            },
            SignedGPSPoint {
                lat: 50.0647,
                lng: 19.9450,
                timestamp: 1_700_003_600,
                accuracy: 10.0,
                signature: vec![0u8; 64],
            },
            SignedGPSPoint {
                lat: 54.3520,
                lng: 18.6466,
                timestamp: 1_700_007_200,
                accuracy: 10.0,
                signature: vec![0u8; 64],
            },
        ]
    }

    fn stability_request(radius_m: u32, min_count: u32) -> ProofRequest {
        ProofRequest {
            claim_type: ClaimType::Stability,
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m,
            time_start: 1_699_999_000,
            time_end: 1_700_100_000,
            min_count,
            night_only: false,
        }
    }

    #[test]
    fn test_tight_cluster_qualifies() {
        let points = make_stable_points(10);
        let analysis = analyze_stability(&points, &stability_request(2000, 5));
        assert!(
            analysis.max_deviation_m < 200.0,
            "tight cluster max deviation should be < 200m, got {}",
            analysis.max_deviation_m
        );
        assert_eq!(analysis.qualifying_indices.len(), 10);
        assert_eq!(analysis.total_in_window, 10);
    }

    #[test]
    fn test_spread_out_fails() {
        let points = make_unstable_points();
        let analysis = analyze_stability(&points, &stability_request(2000, 3));
        assert!(
            analysis.max_deviation_m > 100_000.0,
            "spread points should have high deviation, got {}",
            analysis.max_deviation_m
        );
        assert!(
            analysis.qualifying_indices.len() < 3,
            "spread points should not all qualify with 2km threshold"
        );
    }

    #[test]
    fn test_empty_input() {
        let analysis = analyze_stability(&[], &stability_request(2000, 1));
        assert_eq!(analysis.total_in_window, 0);
        assert!(analysis.qualifying_indices.is_empty());
    }

    #[test]
    fn test_time_window_filter() {
        let points = make_stable_points(10);
        let req = ProofRequest {
            claim_type: ClaimType::Stability,
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m: 2000,
            time_start: 1_700_000_000,
            time_end: 1_700_000_000 + 3 * 3600,
            min_count: 1,
            night_only: false,
        };
        let analysis = analyze_stability(&points, &req);
        assert_eq!(analysis.total_in_window, 4);
    }

    #[test]
    fn test_centroid_is_mean() {
        let points = vec![
            SignedGPSPoint {
                lat: 52.0,
                lng: 20.0,
                timestamp: 1_700_000_000,
                accuracy: 5.0,
                signature: vec![0u8; 64],
            },
            SignedGPSPoint {
                lat: 54.0,
                lng: 22.0,
                timestamp: 1_700_003_600,
                accuracy: 5.0,
                signature: vec![0u8; 64],
            },
        ];
        let analysis = analyze_stability(&points, &stability_request(500_000, 1));
        assert!((analysis.centroid_lat - 53.0).abs() < 0.001);
        assert!((analysis.centroid_lng - 21.0).abs() < 0.001);
    }
}
