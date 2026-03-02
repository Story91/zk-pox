# ZK-PoX Extension

ZK-PoX integrates with the 0x01 mesh as an **extension payload**, not a core protocol change.

The core node (`zerox1-node`) remains ignorant of ZK cryptography. It simply
broadcasts whatever JSON is in the `extensions` field of ADVERTISE messages.

## Architecture

```
Mobile App                          Core Node                    Receiving Agent
    |                                   |                              |
    |  generate proof (Rust JNI)        |                              |
    |  format as extension JSON         |                              |
    |                                   |                              |
    |  agent.start({ extensions: {      |                              |
    |    zk_pox: { proof_type, proof }  |                              |
    |  }})                              |                              |
    |                                   |                              |
    |  ────ADVERTISE (normal JSON)────> |                              |
    |                                   |  ────broadcast via mesh────> |
    |                                   |                              |
    |                                   |     checks extensions.zk_pox |
    |                                   |     verifies proof off-mesh  |
    |                                   |     decides to hire/trust    |
```

## What the core node sees

A normal ADVERTISE message with an `extensions` field:

```json
{
  "format": "CBOR",
  "services": ["local_delivery"],
  "geo": { "country": "PL", "city": "Warsaw" },
  "extensions": {
    "zk_pox": {
      "proof_type": "RESIDENCY",
      "radius_m": 2000,
      "time_window_days": 30,
      "count_proven": 28,
      "proof_hash": "a1b2c3...",
      "proof_bytes_b64": "base64-encoded-bulletproof...",
      "commitments_b64": "base64-encoded-commitments...",
      "center_hash": "d4e5f6..."
    }
  }
}
```

The core node does NOT:
- Parse the `zk_pox` extension
- Import any Bulletproofs or curve25519 dependencies
- Add any new message types to the protocol
- Handle CORROBORATE_REQUEST or CORROBORATE_RESPONSE

## Files

| File | Purpose |
|------|---------|
| `zkpox_extension.rs` | Format `ProofResult` into ADVERTISE extension JSON |
| `zkpox_verifier_ext.rs` | Verify a ZK-PoX extension payload (for receiving agents) |

## How corroboration works

Peer corroboration does NOT use custom message types. Instead:

1. Agent generates proof locally
2. Agent broadcasts ADVERTISE with `extensions.zk_pox`
3. Nearby agents who received the ADVERTISE verify the proof themselves
4. If they want to attest, they submit `add_witness` to the on-chain credential
5. The credential's `witness_count` increases

This is pull-based (verifier-initiated), not push-based (prover-initiated).
No changes to the core mesh protocol needed.
