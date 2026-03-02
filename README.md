# ZK-PoX — Zero-Knowledge Proof-of-Experience

Private, verifiable location credentials for Web3 and decentralized physical infrastructure.

## What It Does

Your phone passively records GPS coordinates signed with your SATI cryptographic identity (Ed25519).
You can then generate **zero-knowledge proofs** about WHERE you were and WHEN — without
revealing exact coordinates, exact times, or your identity.

No cameras. No microphones. No human action. Just GPS + time + cryptography.

## Practical Use Cases

### 1. Geo-Gated Airdrops & IRL Quests
Crypto protocols lose millions to bots spoofing GPS for location-based airdrops. ZK-PoX's ATTENDANCE proof + anti-spoofing + stake slashing makes botting geo-bounded events economically unviable.

### 2. DePIN Coverage Verification
Networks like Helium, Hivemapper, and GEODNET pay users for physical coverage — but face massive location fraud. ZK-PoX lets providers prove they cover a region via STABILITY/RESIDENCY proofs without broadcasting their home address to a public ledger.

### 3. Nomad DAOs & Network States
Communities like Zuzalu and Cabin require proof of IRL participation. TRAVEL proofs let users demonstrate "I visited 3 DAO pop-up cities for 3+ days each" without revealing flight details, exact dates, or passport data.

### 4. Location-Aware Agent Marketplace
Within the 0x01 mesh, service-provider agents attach RESIDENCY proofs to ADVERTISE broadcasts. This proves physical presence in the operating area — preventing a node in Singapore from claiming it can pick up groceries in Warsaw.

## What Makes It Unique

| Feature | ZK-PoX | zkLocus | Helium | POAP |
|---------|--------|---------|--------|------|
| Aggregate range proofs on committed GPS | Yes | No (single-point) | No | No |
| Anti-spoofing (velocity, teleport, noise) | Yes | No | Infra-based | No |
| Economic deterrence (stake slashing) | Yes | No | Token-based | No |
| Mesh peer attestation (witnesses) | Yes | No | No | No |
| Mobile-native (Android JNI) | Yes | No | No | No |
| On-chain soulbound credentials | Yes (Solana) | Partial (Mina) | No | Yes |
| No special hardware | Yes | Yes | No (hotspot) | Yes |

## Limitations

We're honest about what ZK-PoX does NOT solve:

- **Phone != Human.** ZK-PoX proves where a *device* was, not where a *person* was. A Sybil attacker with 20 phones in a backpack generates 20 valid GPS histories. This is a fundamental limitation of any GPS-based system.
- **Battery & UX friction.** 24/7 background GPS tracking drains battery and triggers OS warnings. Privacy-conscious users may not want location history stored locally at all.
- **Not a legal document.** Banks, immigration offices, and insurers won't accept ZK proofs instead of utility bills today. This tool targets crypto-native ecosystems where smart contracts are the verifiers, not human bureaucrats.
- **Over-engineered for simple attendance.** A QR code at a concert takes 2 seconds. ZK-PoX is for scenarios where trust is low, spoofing is lucrative, and the verifier is a smart contract — not a bouncer.

## Project Status

### Done

- [x] **Rust core library** (`zkpox-core`) — types, commitments, circuit, prover, verifier
- [x] **Real Bulletproofs** — aggregate range proofs on Pedersen-committed GPS coordinate offsets
- [x] **Cryptographic verification** — full prove/verify round-trip with tamper detection
- [x] **Anti-spoofing module** — teleportation, velocity, zero-noise detection
- [x] **STABILITY proof** — centroid computation, variance analysis, DePIN coverage
- [x] **TRAVEL proof** — multi-region clustering, distinct day counting, nomad DAO
- [x] **ABSENCE proof** — exclusion zone analysis, violation detection, geo-compliance
- [x] **Ed25519 GPS signature verification** — verify + batch verify + tamper detection
- [x] **Prover dispatches by claim type** — each ClaimType has its own qualifying logic
- [x] **JNI bridge** (`zkpox-mobile`) — anti-spoof gate before proof generation
- [x] **GPS Logger + Database** (Kotlin) — passive collection, encrypted SQLite
- [x] **React Native bridge + UI** — GPS stats, proof gen, spoof analysis
- [x] **Solana Anchor program** — soulbound credentials with witness attestation
- [x] **Extension module** — ADVERTISE payload formatter + verifier (no core node changes)
- [x] **61/61 Rust tests passing**
- [x] **Landing page** — Vite + React + Tailwind

### Recently Completed

- [x] SDK helper for injecting extension into ADVERTISE config
- [x] Full cryptographic verification in extension verifier
- [x] Temporal range proofs (prove timestamp in window without revealing it)
- [x] Anchor TypeScript tests (10 test cases for all 3 instructions)
- [x] CI/CD pipeline: GitHub Actions (Rust tests, Android NDK, Clippy)
- [x] ZeroClaw skill (natural language -> proof type mapping)

### TODO

- [ ] Benchmarks on real Android devices
- [ ] Recursive proof compression

## Repository Layout

```
rust/                     Rust workspace
  crates/zkpox-core/        Core: types, commitments, circuit, prover, verifier, antispoof,
                             stability, travel, absence, temporal
  crates/zkpox-mobile/      JNI bridge (compiles to libzkpox_mobile.so)
android/                  Kotlin files for mobile app
react-native/             React Native layer
solana/                   Anchor program (standalone, not in node workspace)
extension/                Extension payload formatter + verifier (no core node changes)
landing/                  Landing page (Vite + React + Tailwind)
zeroclaw-skill/           ZeroClaw SKILL.toml for natural language proof mapping
.github/workflows/        CI/CD: Rust tests, Android NDK cross-compile, Clippy
INTEGRATION.md            Step-by-step guide (extension model — no node.rs patching)
```

**Important**: ZK-PoX is an extension, not a core protocol change. The core node
(`zerox1-node`) does NOT import ZK dependencies. Proofs travel as opaque JSON in
the `extensions` field of ADVERTISE messages.

## Proof Types

| Type | Proves | Best For |
|------|--------|----------|
| ATTENDANCE | "Within R meters of E for T+ hours on date D" | Geo-gated airdrops, IRL quests, event POAPs |
| RESIDENCY | "Near location H for N+ nights in period P" | DePIN coverage, agent marketplace locality |
| STABILITY | "Location variance below threshold over period P" | DePIN uptime, consistent coverage proof |
| TRAVEL | "In N distinct regions during period P" | Nomad DAOs, network state participation |
| COMMUTE | "Traveled A to B, D days/week, for W weeks" | Agent work pattern verification |
| ABSENCE | "NOT within R meters of X during period P" | Geo-exclusion compliance |

## Building

```bash
cd rust && cargo build
cd rust && cargo test   # 61 tests
```

## Integration

See [INTEGRATION.md](INTEGRATION.md) for step-by-step instructions.

## References

- "Private Proofs of When and Where" — Columbia/MIT, ePrint 2026/136
- "Zero-Knowledge Location Privacy via Floating-Point SNARKs" — TU Munich
- "zkLocus: Authenticated Private Geolocation" — Recursive zkSNARKs
- "OLP Protocol" — Bulletproofs + BLS location verification
