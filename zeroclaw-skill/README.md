# ZK-PoX ZeroClaw Skill

Maps natural language location proof requests to ZK-PoX proof types.

## Installation

Copy this directory to the ZeroClaw skills folder:

```bash
cp -r zk-pox/zeroclaw-skill ~/.zeroclaw/workspace/skills/zk-pox
```

## What It Does

When a user says "Prove I attended ETH Denver", the ZeroClaw agent:

1. Parses the intent to `ATTENDANCE` proof type
2. Geocodes "ETH Denver" to coordinates
3. Runs `check_spoof_risk` to verify GPS integrity
4. Calls `prove_attendance` with the right parameters
5. Returns the proof result or error

## Supported Proof Types

| User Says | Maps To | Tool |
|-----------|---------|------|
| "Prove I attended [event]" | ATTENDANCE | `prove_attendance` |
| "Prove I live in [place]" | RESIDENCY | `prove_residency` |
| "Prove my coverage is stable" | STABILITY | `prove_stability` |
| "Prove I visited N places" | TRAVEL | `prove_travel` |
| "Prove I wasn't at [place]" | ABSENCE | `prove_absence` |
| "Check if my GPS is clean" | Anti-spoof | `check_spoof_risk` |

## Integration

The tools output JSON that the mobile app's `ZkPoxModule` consumes.
In production, these shell commands are replaced by native JNI calls
through the React Native bridge.
