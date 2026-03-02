import { NativeModules } from 'react-native';

interface GpsStats {
  totalPoints: number;
  oldestTimestamp?: number;
  newestTimestamp?: number;
  daysTracked: number;
}

interface ProofRequest {
  claim_type: number;
  center_lat: number;
  center_lng: number;
  radius_m: number;
  time_start: number;
  time_end: number;
  min_count: number;
  night_only?: boolean;
}

interface SpoofAnalysis {
  total_points: number;
  teleport_count: number;
  zero_noise_count: number;
  impossible_velocity_count: number;
  suspicion_score: number;
  verdict: 'Clean' | 'Suspicious' | 'LikelySpoofed';
}

interface ZkPoxExtension {
  proof_type: string;
  radius_m: number;
  time_window_days: number;
  count_proven: number;
  proof_hash: string;
  proof_bytes_b64: string;
  commitments_b64: string;
  center_hash: string;
}

interface ZkPoxModuleInterface {
  getGpsStats(): Promise<GpsStats>;
  generateProof(requestJson: string): Promise<string>;
  verifyProof(resultJson: string): Promise<boolean>;
  analyzeSpoofRisk(days: number): Promise<string>;
  countNightsNear(
    centerLat: number,
    centerLng: number,
    radiusM: number,
    days: number,
  ): Promise<number>;
  /** Format proof result as ADVERTISE extension JSON for agent.start() */
  formatAsExtension(proofResultJson: string): Promise<string>;
}

const { ZkPoxModule } = NativeModules;

export default ZkPoxModule as ZkPoxModuleInterface;
export type {
  GpsStats,
  ProofRequest,
  SpoofAnalysis,
  ZkPoxExtension,
  ZkPoxModuleInterface,
};
