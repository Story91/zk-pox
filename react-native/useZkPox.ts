import { useCallback, useEffect, useState } from 'react';
import ZkPoxModule, { GpsStats, ProofRequest, SpoofAnalysis, ZkPoxExtension } from './ZkPoxModule';

interface ProofResult {
  proof_bytes: string;
  public_inputs: {
    center_hash: string;
    radius_m: number;
    time_window_days: number;
    min_count: number;
    count_proven: number;
  };
  claim_type: number;
  generated_at: number;
  total_points_evaluated: number;
  commitments: string;
}

type ProofStatus = 'idle' | 'generating' | 'success' | 'error';

export function useZkPox() {
  const [stats, setStats] = useState<GpsStats | null>(null);
  const [statsLoading, setStatsLoading] = useState(true);
  const [proofStatus, setProofStatus] = useState<ProofStatus>('idle');
  const [proofResult, setProofResult] = useState<ProofResult | null>(null);
  const [proofError, setProofError] = useState<string | null>(null);
  const [spoofAnalysis, setSpoofAnalysis] = useState<SpoofAnalysis | null>(null);
  const [spoofLoading, setSpoofLoading] = useState(false);

  const refreshStats = useCallback(async () => {
    setStatsLoading(true);
    try {
      const s = await ZkPoxModule.getGpsStats();
      setStats(s);
    } catch (err: any) {
      console.error('Failed to get GPS stats:', err);
    } finally {
      setStatsLoading(false);
    }
  }, []);

  useEffect(() => {
    refreshStats();
  }, [refreshStats]);

  const generateProof = useCallback(async (request: ProofRequest) => {
    setProofStatus('generating');
    setProofError(null);
    setProofResult(null);

    try {
      const resultJson = await ZkPoxModule.generateProof(
        JSON.stringify(request),
      );
      const result: ProofResult = JSON.parse(resultJson);
      setProofResult(result);
      setProofStatus('success');
      return result;
    } catch (err: any) {
      const msg = err?.message || 'Proof generation failed';
      setProofError(msg);
      setProofStatus('error');
      throw err;
    }
  }, []);

  const analyzeSpoofRisk = useCallback(async (days: number = 30) => {
    setSpoofLoading(true);
    try {
      const json = await ZkPoxModule.analyzeSpoofRisk(days);
      const analysis: SpoofAnalysis = JSON.parse(json);
      setSpoofAnalysis(analysis);
      return analysis;
    } catch (err: any) {
      console.error('Spoof analysis failed:', err);
      return null;
    } finally {
      setSpoofLoading(false);
    }
  }, []);

  const verifyProof = useCallback(async (resultJson: string) => {
    return ZkPoxModule.verifyProof(resultJson);
  }, []);

  const countNightsNear = useCallback(
    async (
      centerLat: number,
      centerLng: number,
      radiusM: number,
      days: number,
    ) => {
      return ZkPoxModule.countNightsNear(centerLat, centerLng, radiusM, days);
    },
    [],
  );

  const formatAsExtension = useCallback(async (resultJson: string): Promise<Record<string, ZkPoxExtension>> => {
    const json = await ZkPoxModule.formatAsExtension(resultJson);
    return JSON.parse(json);
  }, []);

  return {
    stats,
    statsLoading,
    refreshStats,
    proofStatus,
    proofResult,
    proofError,
    generateProof,
    verifyProof,
    spoofAnalysis,
    spoofLoading,
    analyzeSpoofRisk,
    countNightsNear,
    formatAsExtension,
  };
}
