//! ABSENCE claim: prove you were NOT within a geofence during a period.
//!
//! The inverse of ATTENDANCE — proves "none of my GPS points fall
//! within radius_m of center during [time_start, time_end]."
//!
//! The ZK proof demonstrates that ALL points in the time window have
//! offsets from the exclusion zone center that EXCEED the exclusion
//! radius. We prove each point's distance is in [radius, MAX_RANGE]
//! rather than [0, radius].
//!
//! Use case: geo-exclusion compliance in trustless systems.

use crate::circuit::{geofence_to_bounding_box, scale_coord};
use crate::types::{haversine_distance_m, ProofRequest, SignedGPSPoint};

/// Result of absence analysis.
#[derive(Debug, Clone)]
pub struct AbsenceAnalysis {
    /// Number of points in the time window.
    pub total_in_window: u32,
    /// Points that are OUTSIDE the exclusion zone (these prove absence).
    pub outside_indices: Vec<usize>,
    /// Points that violate absence (inside the zone). Should be 0 for valid claim.
    pub violation_count: u32,
    /// Minimum distance from any point to the exclusion center (meters).
    pub min_distance_m: f64,
}

/// Analyze GPS points for absence from an exclusion zone.
///
/// Returns all points in the time window that are OUTSIDE the geofence.
/// For a valid ABSENCE claim, `violation_count` must be 0.
pub fn analyze_absence(
    points: &[SignedGPSPoint],
    request: &ProofRequest,
) -> AbsenceAnalysis {
    let (lat_min, lat_max, lng_min, lng_max) =
        geofence_to_bounding_box(request.center_lat, request.center_lng, request.radius_m);

    let mut outside_indices = Vec::new();
    let mut violation_count = 0u32;
    let mut total_in_window = 0u32;
    let mut min_distance_m = f64::MAX;

    for (i, p) in points.iter().enumerate() {
        if p.timestamp < request.time_start || p.timestamp > request.time_end {
            continue;
        }
        total_in_window += 1;

        let d = haversine_distance_m(p.lat, p.lng, request.center_lat, request.center_lng);
        if d < min_distance_m {
            min_distance_m = d;
        }

        let lat_s = scale_coord(p.lat);
        let lng_s = scale_coord(p.lng);

        let inside = lat_s >= lat_min && lat_s <= lat_max
            && lng_s >= lng_min && lng_s <= lng_max;

        if inside {
            violation_count += 1;
        } else {
            outside_indices.push(i);
        }
    }

    if total_in_window == 0 {
        min_distance_m = 0.0;
    }

    AbsenceAnalysis {
        total_in_window,
        outside_indices,
        violation_count,
        min_distance_m,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ClaimType;

    fn gps(lat: f64, lng: f64, ts: i64) -> SignedGPSPoint {
        SignedGPSPoint {
            lat,
            lng,
            timestamp: ts,
            accuracy: 10.0,
            signature: vec![0u8; 64],
        }
    }

    fn absence_request(center_lat: f64, center_lng: f64, radius_m: u32) -> ProofRequest {
        ProofRequest {
            claim_type: ClaimType::Absence,
            center_lat,
            center_lng,
            radius_m,
            time_start: 1_700_000_000,
            time_end: 1_700_100_000,
            min_count: 1,
            night_only: false,
        }
    }

    #[test]
    fn test_all_points_outside() {
        let points = vec![
            gps(50.0647, 19.9450, 1_700_000_100), // Krakow
            gps(54.3520, 18.6466, 1_700_001_000), // Gdansk
        ];
        // Exclusion zone: Warsaw
        let analysis = analyze_absence(&points, &absence_request(52.2297, 21.0122, 5000));
        assert_eq!(analysis.violation_count, 0);
        assert_eq!(analysis.outside_indices.len(), 2);
        assert!(analysis.min_distance_m > 200_000.0);
    }

    #[test]
    fn test_point_inside_exclusion_zone() {
        let points = vec![
            gps(52.2300, 21.0125, 1_700_000_100), // ~50m from Warsaw center
            gps(50.0647, 19.9450, 1_700_001_000), // Krakow — outside
        ];
        let analysis = analyze_absence(&points, &absence_request(52.2297, 21.0122, 5000));
        assert_eq!(analysis.violation_count, 1);
        assert_eq!(analysis.outside_indices.len(), 1);
    }

    #[test]
    fn test_empty_window() {
        let points = vec![
            gps(52.2297, 21.0122, 1_600_000_000), // way before time window
        ];
        let analysis = analyze_absence(&points, &absence_request(52.2297, 21.0122, 5000));
        assert_eq!(analysis.total_in_window, 0);
        assert_eq!(analysis.violation_count, 0);
    }

    #[test]
    fn test_all_points_inside_zone() {
        let points = vec![
            gps(52.2297, 21.0122, 1_700_000_100),
            gps(52.2298, 21.0123, 1_700_001_000),
            gps(52.2296, 21.0121, 1_700_002_000),
        ];
        let analysis = analyze_absence(&points, &absence_request(52.2297, 21.0122, 5000));
        assert_eq!(analysis.violation_count, 3);
        assert_eq!(analysis.outside_indices.len(), 0);
    }

    #[test]
    fn test_min_distance_tracked() {
        let points = vec![
            gps(52.2500, 21.0500, 1_700_000_100), // ~3km from center
            gps(52.3000, 21.1000, 1_700_001_000), // ~10km from center
        ];
        let analysis = analyze_absence(&points, &absence_request(52.2297, 21.0122, 1000));
        assert!(analysis.min_distance_m > 2000.0);
        assert!(analysis.min_distance_m < 5000.0);
    }
}
