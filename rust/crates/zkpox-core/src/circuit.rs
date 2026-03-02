//! ZK range proof circuit for location claims.
//!
//! Approach: Instead of proving Haversine distance inside a ZK circuit (expensive
//! due to trigonometric functions), we use a **bounding-box approximation**:
//!
//! 1. Convert the circular geofence (center, radius) into a lat/lng bounding box.
//! 2. Prove that a GPS point's integer-scaled coordinates fall within the box
//!    using Bulletproofs range proofs.
//! 3. The verifier checks the range proof against the committed center hash.
//!
//! This is conservative (the bounding box is slightly larger than the circle)
//! but avoids floating-point arithmetic inside the circuit.

use crate::types::{ProofRequest, SignedGPSPoint};

/// Scale factor: GPS coordinates are multiplied by this before integer conversion.
/// 1e7 gives ~1.1cm precision at the equator, more than sufficient.
pub const COORD_SCALE: f64 = 1e7;

/// Meters per degree of latitude (approximately constant).
const METERS_PER_DEG_LAT: f64 = 111_320.0;

/// Convert a circular geofence into a lat/lng bounding box.
///
/// Returns `(lat_min_scaled, lat_max_scaled, lng_min_scaled, lng_max_scaled)`
/// as integer-scaled coordinates (multiplied by `COORD_SCALE`).
pub fn geofence_to_bounding_box(
    center_lat: f64,
    center_lng: f64,
    radius_m: u32,
) -> (i64, i64, i64, i64) {
    let r = radius_m as f64;

    let delta_lat = r / METERS_PER_DEG_LAT;
    let meters_per_deg_lng = METERS_PER_DEG_LAT * center_lat.to_radians().cos();
    let delta_lng = if meters_per_deg_lng > 0.0 {
        r / meters_per_deg_lng
    } else {
        360.0
    };

    let lat_min = ((center_lat - delta_lat) * COORD_SCALE) as i64;
    let lat_max = ((center_lat + delta_lat) * COORD_SCALE) as i64;
    let lng_min = ((center_lng - delta_lng) * COORD_SCALE) as i64;
    let lng_max = ((center_lng + delta_lng) * COORD_SCALE) as i64;

    (lat_min, lat_max, lng_min, lng_max)
}

/// Scale a GPS coordinate to an integer value.
pub fn scale_coord(coord: f64) -> i64 {
    (coord * COORD_SCALE) as i64
}

/// Check whether a point falls within the bounding box (cleartext, for pre-filtering).
pub fn point_in_bbox(
    lat_scaled: i64,
    lng_scaled: i64,
    lat_min: i64,
    lat_max: i64,
    lng_min: i64,
    lng_max: i64,
) -> bool {
    lat_scaled >= lat_min && lat_scaled <= lat_max
        && lng_scaled >= lng_min && lng_scaled <= lng_max
}

/// Pre-filter GPS points: count how many fall within the geofence bounding box
/// and within the time window.
pub fn count_qualifying_points(
    points: &[SignedGPSPoint],
    request: &ProofRequest,
) -> (u32, Vec<usize>) {
    let (lat_min, lat_max, lng_min, lng_max) =
        geofence_to_bounding_box(request.center_lat, request.center_lng, request.radius_m);

    let mut count = 0u32;
    let mut qualifying_indices = Vec::new();

    for (i, p) in points.iter().enumerate() {
        if p.timestamp < request.time_start || p.timestamp > request.time_end {
            continue;
        }

        if request.night_only {
            let hour_of_day = ((p.timestamp % 86_400) / 3_600) as u32;
            if hour_of_day < 22 && hour_of_day >= 7 {
                continue;
            }
        }

        let lat_s = scale_coord(p.lat);
        let lng_s = scale_coord(p.lng);

        if point_in_bbox(lat_s, lng_s, lat_min, lat_max, lng_min, lng_max) {
            count += 1;
            qualifying_indices.push(i);
        }
    }

    (count, qualifying_indices)
}

/// Represents a range proof statement: "value V is in [lower, upper]".
///
/// In the full Bulletproofs implementation, each RangeStatement becomes a
/// committed Pedersen commitment with a range proof. For now, we define the
/// structure and provide a commitment-based proof that can be upgraded to
/// full Bulletproofs ZK proofs.
#[derive(Debug, Clone)]
pub struct RangeStatement {
    /// The secret value being proven.
    pub value: i64,
    /// Lower bound (inclusive).
    pub lower: i64,
    /// Upper bound (inclusive).
    pub upper: i64,
}

impl RangeStatement {
    pub fn is_valid(&self) -> bool {
        self.value >= self.lower && self.value <= self.upper
    }
}

/// Build range statements for all qualifying points.
///
/// For each qualifying point, we create 2 range statements:
///   1. lat_scaled ∈ [lat_min, lat_max]
///   2. lng_scaled ∈ [lng_min, lng_max]
pub fn build_range_statements(
    points: &[SignedGPSPoint],
    qualifying_indices: &[usize],
    lat_min: i64,
    lat_max: i64,
    lng_min: i64,
    lng_max: i64,
) -> Vec<RangeStatement> {
    let mut statements = Vec::with_capacity(qualifying_indices.len() * 2);

    for &idx in qualifying_indices {
        let p = &points[idx];
        let lat_s = scale_coord(p.lat);
        let lng_s = scale_coord(p.lng);

        statements.push(RangeStatement {
            value: lat_s,
            lower: lat_min,
            upper: lat_max,
        });
        statements.push(RangeStatement {
            value: lng_s,
            lower: lng_min,
            upper: lng_max,
        });
    }

    statements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_warsaw() {
        // Warsaw: 52.2297°N, 21.0122°E, 200m radius
        let (lat_min, lat_max, lng_min, lng_max) =
            geofence_to_bounding_box(52.2297, 21.0122, 200);

        let center_lat_s = scale_coord(52.2297);
        let center_lng_s = scale_coord(21.0122);

        assert!(lat_min < center_lat_s);
        assert!(lat_max > center_lat_s);
        assert!(lng_min < center_lng_s);
        assert!(lng_max > center_lng_s);

        // 200m ≈ 0.0018° lat, bounding box should be roughly ±0.0018°
        let delta_lat = (lat_max - lat_min) as f64 / COORD_SCALE;
        assert!(delta_lat > 0.003 && delta_lat < 0.004);
    }

    #[test]
    fn test_point_in_bbox() {
        let (lat_min, lat_max, lng_min, lng_max) =
            geofence_to_bounding_box(52.2297, 21.0122, 200);

        // Center should be inside
        assert!(point_in_bbox(
            scale_coord(52.2297), scale_coord(21.0122),
            lat_min, lat_max, lng_min, lng_max
        ));

        // Far away should be outside
        assert!(!point_in_bbox(
            scale_coord(48.8566), scale_coord(2.3522),
            lat_min, lat_max, lng_min, lng_max
        ));
    }

    #[test]
    fn test_count_qualifying_points() {
        let points = vec![
            SignedGPSPoint { lat: 52.2297, lng: 21.0122, timestamp: 1000, accuracy: 5.0, signature: vec![0u8; 64] },
            SignedGPSPoint { lat: 52.2298, lng: 21.0123, timestamp: 2000, accuracy: 5.0, signature: vec![0u8; 64] },
            SignedGPSPoint { lat: 48.8566, lng: 2.3522, timestamp: 3000, accuracy: 5.0, signature: vec![0u8; 64] },
        ];

        let request = ProofRequest {
            claim_type: crate::types::ClaimType::Attendance,
            center_lat: 52.2297,
            center_lng: 21.0122,
            radius_m: 200,
            time_start: 0,
            time_end: 5000,
            min_count: 1,
            night_only: false,
        };

        let (count, indices) = count_qualifying_points(&points, &request);
        assert_eq!(count, 2);
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn test_range_statement_validity() {
        let valid = RangeStatement { value: 50, lower: 10, upper: 100 };
        assert!(valid.is_valid());

        let invalid = RangeStatement { value: 5, lower: 10, upper: 100 };
        assert!(!invalid.is_valid());
    }
}
