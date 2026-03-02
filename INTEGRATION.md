# ZK-PoX Integration Guide

ZK-PoX is an **extension**, not a core protocol change.

The core node (`zerox1-node`) does NOT compile ZK dependencies, does NOT add
new message types, and does NOT handle Bulletproofs. It simply broadcasts
whatever JSON the mobile app puts in the `extensions` field of ADVERTISE messages.

---

## Architecture: Extension Model

```
┌─────────────────────────────────────────────────────────────────┐
│                     MOBILE APP                                   │
│                                                                  │
│  1. GPS Logger collects signed points (GpsLogger.kt)             │
│  2. User requests proof ("Prove I lived here 30 days")           │
│  3. Rust JNI generates Bulletproof (zkpox-mobile → zkpox-core)   │
│  4. ProofResult formatted as extension JSON                      │
│  5. Extension injected into ADVERTISE payload                    │
│                                                                  │
│  agent.start({                                                   │
│    extensions: {                                                 │
│      zk_pox: { proof_type: "RESIDENCY", proof_bytes_b64: "..." } │
│    }                                                             │
│  })                                                              │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   │  Normal ADVERTISE broadcast (no protocol changes)
                   │
┌──────────────────▼──────────────────────────────────────────────┐
│              CORE NODE (zerox1-node)                              │
│                                                                  │
│  The node is IGNORANT of ZK-PoX.                                 │
│  It takes the ADVERTISE JSON and broadcasts it via gossipsub.    │
│  No new MsgType. No Bulletproofs dependency. No changes needed.  │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   │  Mesh broadcast
                   │
┌──────────────────▼──────────────────────────────────────────────┐
│              RECEIVING AGENT                                     │
│                                                                  │
│  1. Receives ADVERTISE with extensions.zk_pox                    │
│  2. Agent logic checks: does this agent have a ZK-PoX proof?     │
│  3. If yes: decode proof, verify using zkpox-core verifier       │
│  4. Decision: trust/hire based on verification result            │
│  5. Optionally: submit add_witness on-chain to attest            │
└──────────────────────────────────────────────────────────────────┘
```

**Key principle**: The core mesh stays lean, neutral, and un-opinionated. ZK-PoX
is opt-in for agents that need location verification. Server-based agents (e.g., a
trading bot in Frankfurt) never compile or interact with Bulletproofs code.

---

## 1. Mobile App — Android (Kotlin)

### 1.1 Copy Kotlin files

```
zk-pox/android/GpsLogger.kt   → mobile/android/.../world/zerox1/node/GpsLogger.kt
zk-pox/android/GpsDatabase.kt → mobile/android/.../world/zerox1/node/GpsDatabase.kt
zk-pox/android/ZkPoxModule.kt → mobile/android/.../world/zerox1/node/ZkPoxModule.kt
```

### 1.2 Wire GPS Logger into NodeService

Follow `zk-pox/android/NodeService.patch`:
1. Add `gpsLogger` and `gpsDatabase` fields to `NodeService`
2. Start the logger in `onCreate()` after wakeLock acquisition
3. Stop the logger in `onDestroy()`

### 1.3 Build the native library

```bash
cd zk-pox/rust
cargo ndk -t arm64-v8a -t armeabi-v7a -o ../mobile/android/app/src/main/jniLibs build --release -p zkpox-mobile
```

### 1.4 Inject extension into ADVERTISE

After generating a proof, format it as an extension and pass it to the SDK:

```kotlin
val proofResult = ZkPoxModule.generateProof(points, request)
val extension = mapOf(
    "zk_pox" to mapOf(
        "proof_type" to proofResult.claimType,
        "radius_m" to proofResult.radiusM,
        "time_window_days" to proofResult.timeWindowDays,
        "count_proven" to proofResult.countProven,
        "proof_hash" to proofResult.proofHash,
        "proof_bytes_b64" to Base64.encode(proofResult.proofBytes),
        "commitments_b64" to Base64.encode(proofResult.commitments),
        "center_hash" to proofResult.centerHash
    )
)
// Pass to agent start config — node broadcasts this as-is
agentConfig.extensions = extension
```

The core node treats this as opaque JSON. No node.rs changes needed.

---

## 2. Mobile App — React Native

### 2.1 Copy React Native files

```
zk-pox/react-native/Credentials.tsx → mobile/src/screens/Credentials.tsx
zk-pox/react-native/useZkPox.ts     → mobile/src/hooks/useZkPox.ts
zk-pox/react-native/ZkPoxModule.ts  → mobile/src/native/ZkPoxModule.ts
```

### 2.2 Add navigation

```tsx
<Stack.Screen name="Credentials" component={CredentialsScreen} />
```

---

## 3. Solana — Anchor Program

### 3.1 Deploy

```bash
cd zk-pox/solana
anchor build -p zk_pox
anchor deploy -p zk_pox --provider.cluster devnet
```

### 3.2 On-chain credential (ExperienceCredential PDA)

| Field | Type | Description |
|-------|------|-------------|
| `version` | u8 | Schema version (2) |
| `agent_id` | [u8; 32] | SATI agent public key |
| `claim_type` | u8 | 0=Residency, ..., 5=Travel |
| `proof_hash` | [u8; 32] | SHA-256 of Bulletproof bytes |
| `public_inputs_hash` | [u8; 32] | SHA-256 of public inputs |
| `commitments_hash` | [u8; 32] | SHA-256 of Pedersen commitments |
| `count_proven` | u32 | GPS points proven |
| `witness_count` | u8 | Peer attestations (0-8) |
| `witnesses` | [[u8; 32]; 8] | Attestation keys |

### 3.3 No changes to node programs workspace

The ZK-PoX Anchor program is a **standalone Solana program**, not embedded
in the `node/programs/workspace`. Deploy it independently.

---

## 4. Node — NO CHANGES REQUIRED

The core node (`zerox1-node`) requires **zero modifications** for ZK-PoX:

- No new message types (no CORROBORATE_REQUEST/RESPONSE)
- No new dependencies (no bulletproofs, no curve25519)
- No new modules (no zkpox.rs in node/src/)
- No constants changes

The ADVERTISE payload already supports arbitrary JSON in the `extensions` field.
ZK-PoX uses this existing mechanism.

### Peer attestation (corroboration)

Corroboration is pull-based, not push-based:

1. Agent A broadcasts ADVERTISE with `extensions.zk_pox`
2. Agent B receives it, verifies the proof using their own `zkpox-core` library
3. If valid, Agent B calls `add_witness` on the Solana program
4. The credential's `witness_count` increases

No custom mesh messages needed. The on-chain `add_witness` instruction
is the attestation mechanism.

---

## 5. Receiving Agent — Optional Verifier

Agents that WANT to verify ZK-PoX proofs need the verifier library:

```toml
# In their Cargo.toml (or use the TypeScript SDK equivalent)
zkpox-core = { path = "path/to/zk-pox/rust/crates/zkpox-core" }
```

Verification flow:
1. Parse `extensions.zk_pox` from received ADVERTISE
2. Decode `proof_bytes_b64` and `commitments_b64` from base64
3. Call `zkpox_core::verifier::verify_proof()` with the decoded data
4. If valid, trust the agent's location claim

Agents that don't care about ZK-PoX simply ignore the extension field.

---

## File Map

| Source | Destination | Action |
|---|---|---|
| `android/GpsLogger.kt` | `mobile/.../GpsLogger.kt` | Copy |
| `android/GpsDatabase.kt` | `mobile/.../GpsDatabase.kt` | Copy |
| `android/ZkPoxModule.kt` | `mobile/.../ZkPoxModule.kt` | Copy |
| `android/NodeService.patch` | Apply to `NodeService.kt` | Manual edit |
| `react-native/Credentials.tsx` | `mobile/src/screens/` | Copy |
| `react-native/useZkPox.ts` | `mobile/src/hooks/` | Copy |
| `react-native/ZkPoxModule.ts` | `mobile/src/native/` | Copy |
| `solana/` | Deploy as standalone program | `anchor deploy` |
| `extension/` | Reference implementation | For SDK integration |
| `rust/` | Build dependency for mobile + verifiers | `cargo ndk` |

**What is NOT in this map**: No files copy to `node/crates/zerox1-node/`.
The core node is not modified.
