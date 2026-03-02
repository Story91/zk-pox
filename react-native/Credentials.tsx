import React, { useEffect, useState } from 'react';
import {
  ActivityIndicator,
  Alert,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  TouchableOpacity,
  View,
} from 'react-native';
import { useZkPox } from './useZkPox';

const CLAIM_TYPES = [
  { label: 'Residency', value: 0 },
  { label: 'Commute', value: 1 },
  { label: 'Attendance', value: 2 },
  { label: 'Absence', value: 3 },
  { label: 'Stability', value: 4 },
  { label: 'Travel', value: 5 },
];

const VERDICT_COLORS: Record<string, string> = {
  Clean: '#22cc88',
  Suspicious: '#ffaa33',
  LikelySpoofed: '#ff4444',
};

export default function CredentialsScreen() {
  const {
    stats,
    statsLoading,
    refreshStats,
    proofStatus,
    proofResult,
    proofError,
    generateProof,
    spoofAnalysis,
    spoofLoading,
    analyzeSpoofRisk,
  } = useZkPox();

  const [selectedClaim, setSelectedClaim] = useState(0);
  const [radiusM, setRadiusM] = useState('200');
  const [daysBack, setDaysBack] = useState('30');
  const [minCount, setMinCount] = useState('10');

  useEffect(() => {
    analyzeSpoofRisk(30);
  }, [analyzeSpoofRisk]);

  const handleGenerate = async () => {
    const now = Math.floor(Date.now() / 1000);
    const days = parseInt(daysBack, 10) || 30;
    const timeStart = now - days * 86400;

    try {
      await generateProof({
        claim_type: selectedClaim,
        center_lat: 0,
        center_lng: 0,
        radius_m: parseInt(radiusM, 10) || 200,
        time_start: timeStart,
        time_end: now,
        min_count: parseInt(minCount, 10) || 10,
        night_only: selectedClaim === 0,
      });
    } catch {
      Alert.alert('Proof Failed', proofError || 'Unknown error');
    }
  };

  return (
    <ScrollView style={styles.container}>
      <Text style={styles.title}>ZK-PoX Credentials</Text>

      {/* GPS Stats */}
      <View style={styles.card}>
        <Text style={styles.cardTitle}>GPS Collection</Text>
        {statsLoading ? (
          <ActivityIndicator />
        ) : stats ? (
          <>
            <StatRow label="Total Points" value={stats.totalPoints.toLocaleString()} />
            <StatRow label="Days Tracked" value={`${stats.daysTracked}`} />
            <StatRow
              label="Oldest"
              value={
                stats.oldestTimestamp
                  ? new Date(stats.oldestTimestamp * 1000).toLocaleDateString()
                  : '\u2014'
              }
            />
            <StatRow
              label="Newest"
              value={
                stats.newestTimestamp
                  ? new Date(stats.newestTimestamp * 1000).toLocaleDateString()
                  : '\u2014'
              }
            />
          </>
        ) : (
          <Text style={styles.muted}>No GPS data yet</Text>
        )}
        <TouchableOpacity style={styles.secondaryBtn} onPress={refreshStats}>
          <Text style={styles.secondaryBtnText}>Refresh</Text>
        </TouchableOpacity>
      </View>

      {/* Anti-Spoofing Analysis */}
      <View style={styles.card}>
        <Text style={styles.cardTitle}>GPS Integrity</Text>
        {spoofLoading ? (
          <ActivityIndicator />
        ) : spoofAnalysis ? (
          <>
            <View style={styles.verdictRow}>
              <Text style={styles.verdictLabel}>Verdict</Text>
              <Text
                style={[
                  styles.verdictValue,
                  { color: VERDICT_COLORS[spoofAnalysis.verdict] || '#fff' },
                ]}
              >
                {spoofAnalysis.verdict}
              </Text>
            </View>
            <StatRow
              label="Suspicion Score"
              value={`${(spoofAnalysis.suspicion_score * 100).toFixed(1)}%`}
            />
            <StatRow label="Points Analyzed" value={`${spoofAnalysis.total_points}`} />
            <StatRow label="Teleportations" value={`${spoofAnalysis.teleport_count}`} />
            <StatRow label="Velocity Anomalies" value={`${spoofAnalysis.impossible_velocity_count}`} />
            <StatRow label="Zero-Noise Runs" value={`${spoofAnalysis.zero_noise_count}`} />
          </>
        ) : (
          <Text style={styles.muted}>Run analysis to check GPS data integrity</Text>
        )}
        <TouchableOpacity
          style={styles.secondaryBtn}
          onPress={() => analyzeSpoofRisk(parseInt(daysBack, 10) || 30)}
        >
          <Text style={styles.secondaryBtnText}>Analyze</Text>
        </TouchableOpacity>
      </View>

      {/* Proof Generator */}
      <View style={styles.card}>
        <Text style={styles.cardTitle}>Generate Proof</Text>

        <Text style={styles.label}>Claim Type</Text>
        <View style={styles.chipRow}>
          {CLAIM_TYPES.map((ct) => (
            <TouchableOpacity
              key={ct.value}
              style={[
                styles.chip,
                selectedClaim === ct.value && styles.chipActive,
              ]}
              onPress={() => setSelectedClaim(ct.value)}
            >
              <Text
                style={[
                  styles.chipText,
                  selectedClaim === ct.value && styles.chipTextActive,
                ]}
              >
                {ct.label}
              </Text>
            </TouchableOpacity>
          ))}
        </View>

        <Text style={styles.label}>Radius (meters)</Text>
        <TextInput
          style={styles.input}
          value={radiusM}
          onChangeText={setRadiusM}
          keyboardType="numeric"
          placeholder="200"
          placeholderTextColor="#666"
        />

        <Text style={styles.label}>Days Back</Text>
        <TextInput
          style={styles.input}
          value={daysBack}
          onChangeText={setDaysBack}
          keyboardType="numeric"
          placeholder="30"
          placeholderTextColor="#666"
        />

        <Text style={styles.label}>Min. Qualifying Points</Text>
        <TextInput
          style={styles.input}
          value={minCount}
          onChangeText={setMinCount}
          keyboardType="numeric"
          placeholder="10"
          placeholderTextColor="#666"
        />

        <TouchableOpacity
          style={[styles.primaryBtn, proofStatus === 'generating' && styles.btnDisabled]}
          onPress={handleGenerate}
          disabled={proofStatus === 'generating'}
        >
          {proofStatus === 'generating' ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text style={styles.primaryBtnText}>Generate ZK Proof</Text>
          )}
        </TouchableOpacity>
      </View>

      {/* Proof Result */}
      {proofResult && (
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Proof Generated</Text>
          <StatRow
            label="Claim"
            value={CLAIM_TYPES[proofResult.claim_type]?.label || '?'}
          />
          <StatRow
            label="Points Proven"
            value={`${proofResult.public_inputs.count_proven} / ${proofResult.public_inputs.min_count} required`}
          />
          <StatRow
            label="Time Window"
            value={`${proofResult.public_inputs.time_window_days} days`}
          />
          <StatRow
            label="Radius"
            value={`${proofResult.public_inputs.radius_m} m`}
          />
          <StatRow
            label="Proof Size"
            value={`${(proofResult.proof_bytes.length / 2).toLocaleString()} bytes`}
          />
          <StatRow
            label="Commitments"
            value={`${(proofResult.commitments.length / 2 / 32)} points`}
          />
          <StatRow
            label="Generated"
            value={new Date(proofResult.generated_at * 1000).toLocaleString()}
          />
          <TouchableOpacity style={styles.primaryBtn}>
            <Text style={styles.primaryBtnText}>Submit to Chain</Text>
          </TouchableOpacity>
        </View>
      )}

      {proofError && proofStatus === 'error' && (
        <View style={[styles.card, styles.errorCard]}>
          <Text style={styles.errorText}>{proofError}</Text>
        </View>
      )}
    </ScrollView>
  );
}

function StatRow({ label, value }: { label: string; value: string }) {
  return (
    <View style={styles.statRow}>
      <Text style={styles.statLabel}>{label}</Text>
      <Text style={styles.statValue}>{value}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0a0a0f',
    padding: 16,
  },
  title: {
    fontSize: 28,
    fontWeight: '700',
    color: '#fff',
    marginBottom: 20,
    marginTop: 8,
  },
  card: {
    backgroundColor: '#141420',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
    borderWidth: 1,
    borderColor: '#1e1e30',
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#e0e0ff',
    marginBottom: 14,
  },
  statRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 6,
  },
  statLabel: {
    color: '#8888aa',
    fontSize: 14,
  },
  statValue: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  verdictRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
    marginBottom: 4,
  },
  verdictLabel: {
    color: '#8888aa',
    fontSize: 16,
    fontWeight: '500',
  },
  verdictValue: {
    fontSize: 16,
    fontWeight: '700',
  },
  label: {
    color: '#8888aa',
    fontSize: 13,
    marginTop: 12,
    marginBottom: 4,
  },
  input: {
    backgroundColor: '#1a1a2e',
    borderRadius: 10,
    paddingHorizontal: 14,
    paddingVertical: 10,
    color: '#fff',
    fontSize: 15,
    borderWidth: 1,
    borderColor: '#2a2a40',
  },
  chipRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  chip: {
    paddingHorizontal: 14,
    paddingVertical: 7,
    borderRadius: 20,
    backgroundColor: '#1a1a2e',
    borderWidth: 1,
    borderColor: '#2a2a40',
  },
  chipActive: {
    backgroundColor: '#3b1de2',
    borderColor: '#5b3df2',
  },
  chipText: {
    color: '#8888aa',
    fontSize: 13,
    fontWeight: '500',
  },
  chipTextActive: {
    color: '#fff',
  },
  primaryBtn: {
    backgroundColor: '#3b1de2',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
    marginTop: 16,
  },
  primaryBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  btnDisabled: {
    opacity: 0.5,
  },
  secondaryBtn: {
    borderWidth: 1,
    borderColor: '#3b1de2',
    borderRadius: 10,
    paddingVertical: 10,
    alignItems: 'center',
    marginTop: 14,
  },
  secondaryBtnText: {
    color: '#7b5df2',
    fontSize: 14,
    fontWeight: '500',
  },
  errorCard: {
    borderColor: '#cc3333',
  },
  errorText: {
    color: '#ff6666',
    fontSize: 14,
  },
  muted: {
    color: '#555',
    fontSize: 14,
  },
});
