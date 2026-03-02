//! TRAVEL claim: prove visits to N distinct geographic regions.
//!
//! Unlike ATTENDANCE (single geofence), TRAVEL proves "I visited
//! N separate regions during period P, spending at least min_days
//! in each." The regions are discovered via spatial clustering —
//! not predefined by the user.
//!
//! Use case: Nomad DAO membership — prove participation in 3+
//! pop-up cities without revealing which cities or exact dates.

use crate::types::{haversine_distance_m, ProofRequest, SignedGPSPoint};

/// A discovered geographic cluster (region).
#[derive(Debug, Clone)]
pub struct Region {
    pub centroid_lat: f64,
    pub centroid_lng: f64,
    /// Number of distinct days with GPS points in this cluster.
    pub distinct_days: u32,
    /// Total GPS points in this cluster.
    pub point_count: u32,
    /// Indices into the original points array.
    pub point_indices: Vec<usize>,
}

/// Result of travel analysis.
#[derive(Debug, Clone)]
pub struct TravelAnalysis {
    /// Discovered regions sorted by distinct_days descending.
    pub regions: Vec<Region>,
    /// Total points that fell within the time window.
    pub total_in_window: u32,
    /// Indices of points belonging to top-N qualifying regions.
    /// For the prover, we pick the region with the most points
    /// as the "anchor" for the range proof.
    pub qualifying_indices: Vec<usize>,
    /// Centroid of the largest qualifying region (for range proof anchor).
    pub anchor_lat: f64,
    pub anchor_lng: f64,
}

/// Analyze GPS points for multi-region travel.
///
/// Algorithm (grid-based spatial clustering):
/// 1. Filter by time window
/// 2. Assign each point to a grid cell (cell size = `radius_m * 2`)
/// 3. Merge adjacent cells with points into regions
/// 4. For each region: compute centroid, count distinct days
/// 5. Return regions sorted by days descending
///
/// `radius_m` defines the cluster radius — points within this distance
/// of each other belong to the same region.
/// `min_count` = minimum number of distinct regions required.
pub fn analyze_travel(
    points: &[SignedGPSPoint],
    request: &ProofRequest,
) -> TravelAnalysis {
    let time_filtered: Vec<(usize, &SignedGPSPoint)> = points
        .iter()
        .enumerate()
        .filter(|(_, p)| p.timestamp >= request.time_start && p.timestamp <= request.time_end)
        .collect();

    if time_filtered.is_empty() {
        return TravelAnalysis {
            regions: vec![],
            total_in_window: 0,
            qualifying_indices: vec![],
            anchor_lat: 0.0,
            anchor_lng: 0.0,
        };
    }

    let total_in_window = time_filtered.len() as u32;
    let cluster_radius_m = request.radius_m as f64;

    let regions = cluster_points(&time_filtered, cluster_radius_m);

    let (qualifying_indices, anchor_lat, anchor_lng) = if regions.is_empty() {
        (vec![], 0.0, 0.0)
    } else {
        let mut all_indices: Vec<usize> = Vec::new();
        for r in &regions {
            all_indices.extend_from_slice(&r.point_indices);
        }
        (all_indices, regions[0].centroid_lat, regions[0].centroid_lng)
    };

    TravelAnalysis {
        regions,
        total_in_window,
        qualifying_indices,
        anchor_lat,
        anchor_lng,
    }
}

/// Simple leader-based clustering: iterate through points, assign each
/// to the nearest existing cluster if within radius, otherwise create new.
/// Then compute per-cluster distinct day counts.
fn cluster_points(
    points: &[(usize, &SignedGPSPoint)],
    radius_m: f64,
) -> Vec<Region> {
    struct Cluster {
        sum_lat: f64,
        sum_lng: f64,
        count: usize,
        indices: Vec<usize>,
        timestamps: Vec<i64>,
    }

    let mut clusters: Vec<Cluster> = Vec::new();

    for &(idx, p) in points {
        let mut assigned = false;
        for c in &mut clusters {
            let centroid_lat = c.sum_lat / c.count as f64;
            let centroid_lng = c.sum_lng / c.count as f64;
            let d = haversine_distance_m(p.lat, p.lng, centroid_lat, centroid_lng);
            if d <= radius_m {
                c.sum_lat += p.lat;
                c.sum_lng += p.lng;
                c.count += 1;
                c.indices.push(idx);
                c.timestamps.push(p.timestamp);
                assigned = true;
                break;
            }
        }
        if !assigned {
            clusters.push(Cluster {
                sum_lat: p.lat,
                sum_lng: p.lng,
                count: 1,
                indices: vec![idx],
                timestamps: vec![p.timestamp],
            });
        }
    }

    let mut regions: Vec<Region> = clusters
        .into_iter()
        .map(|c| {
            let centroid_lat = c.sum_lat / c.count as f64;
            let centroid_lng = c.sum_lng / c.count as f64;
            let distinct_days = count_distinct_days(&c.timestamps);
            Region {
                centroid_lat,
                centroid_lng,
                distinct_days,
                point_count: c.count as u32,
                point_indices: c.indices,
            }
        })
        .collect();

    regions.sort_by(|a, b| b.distinct_days.cmp(&a.distinct_days));
    regions
}

fn count_distinct_days(timestamps: &[i64]) -> u32 {
    let mut days: Vec<i64> = timestamps.iter().map(|&t| t / 86_400).collect();
    days.sort_unstable();
    days.dedup();
    days.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ClaimType, ProofRequest, SignedGPSPoint};

    fn travel_request(radius_m: u32, min_regions: u32) -> ProofRequest {
        ProofRequest {
            claim_type: ClaimType::Travel,
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m,
            time_start: 1_699_000_000,
            time_end: 1_710_000_000,
            min_count: min_regions,
            night_only: false,
        }
    }

    fn make_multi_city_points() -> Vec<SignedGPSPoint> {
        let base = 1_700_000_000i64;
        let day = 86_400i64;
        vec![
            // Warsaw — 5 days
            gps(52.2297, 21.0122, base),
            gps(52.2300, 21.0130, base + day),
            gps(52.2290, 21.0110, base + 2 * day),
            gps(52.2310, 21.0140, base + 3 * day),
            gps(52.2295, 21.0120, base + 4 * day),
            // Krakow — 3 days (~250km from Warsaw)
            gps(50.0647, 19.9450, base + 10 * day),
            gps(50.0650, 19.9460, base + 11 * day),
            gps(50.0640, 19.9440, base + 12 * day),
            // Gdansk — 4 days (~300km from Warsaw)
            gps(54.3520, 18.6466, base + 20 * day),
            gps(54.3530, 18.6480, base + 21 * day),
            gps(54.3510, 18.6450, base + 22 * day),
            gps(54.3525, 18.6470, base + 23 * day),
        ]
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
    fn test_three_distinct_cities() {
        let points = make_multi_city_points();
        let analysis = analyze_travel(&points, &travel_request(50_000, 3));
        assert_eq!(
            analysis.regions.len(),
            3,
            "should find 3 clusters: Warsaw, Krakow, Gdansk"
        );
        assert!(analysis.regions[0].distinct_days >= 3);
        assert!(analysis.regions[1].distinct_days >= 3);
        assert!(analysis.regions[2].distinct_days >= 3);
    }

    #[test]
    fn test_single_city_one_region() {
        let base = 1_700_000_000i64;
        let day = 86_400i64;
        let points: Vec<_> = (0..10)
            .map(|i| gps(52.2297 + (i as f64) * 0.0001, 21.0122, base + i * day))
            .collect();
        let analysis = analyze_travel(&points, &travel_request(50_000, 1));
        assert_eq!(analysis.regions.len(), 1);
        assert_eq!(analysis.regions[0].distinct_days, 10);
    }

    #[test]
    fn test_empty_input() {
        let analysis = analyze_travel(&[], &travel_request(50_000, 1));
        assert_eq!(analysis.total_in_window, 0);
        assert!(analysis.regions.is_empty());
    }

    #[test]
    fn test_time_filter() {
        let points = make_multi_city_points();
        let req = ProofRequest {
            claim_type: ClaimType::Travel,
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m: 50_000,
            time_start: 1_700_000_000,
            time_end: 1_700_000_000 + 5 * 86_400,
            min_count: 1,
            night_only: false,
        };
        let analysis = analyze_travel(&points, &req);
        assert_eq!(analysis.regions.len(), 1, "only Warsaw in 5-day window");
    }

    #[test]
    fn test_distinct_days_dedup() {
        let base = 1_700_006_400i64; // 2023-11-15 01:46:40 UTC — well within a single day
        let points = vec![
            gps(52.23, 21.01, base),
            gps(52.23, 21.01, base + 1800),
            gps(52.23, 21.01, base + 3600),
        ];
        let analysis = analyze_travel(&points, &travel_request(50_000, 1));
        assert_eq!(analysis.regions.len(), 1);
        assert_eq!(
            analysis.regions[0].distinct_days, 1,
            "3 points same day = 1 distinct day"
        );
    }

    #[test]
    fn test_regions_sorted_by_days() {
        let points = make_multi_city_points();
        let analysis = analyze_travel(&points, &travel_request(50_000, 1));
        for w in analysis.regions.windows(2) {
            assert!(w[0].distinct_days >= w[1].distinct_days);
        }
    }
}
