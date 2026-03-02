# Workspace Patch — Adding zk-pox to Anchor Workspace

## File: `node/programs/workspace/Cargo.toml`

### Add to `[workspace.members]`:

```toml
members = [
    # ... existing programs ...
    "programs/zk-pox",
]
```

### Create directory:

```bash
mkdir -p node/programs/workspace/programs/zk-pox/src
```

### Copy files:

```bash
cp zk-pox/solana/Cargo.toml  node/programs/workspace/programs/zk-pox/Cargo.toml
cp zk-pox/solana/src/lib.rs   node/programs/workspace/programs/zk-pox/src/lib.rs
```

### Update Anchor.toml:

Add under `[programs.localnet]` and `[programs.devnet]`:

```toml
zk_pox = "ZKPoX1111111111111111111111111111111111111"
```

### Deploy:

```bash
anchor build -p zk_pox
anchor deploy -p zk_pox --provider.cluster devnet
```

After deployment, replace the placeholder program ID (`ZKPoX111...`) with the
actual deployed program ID in both `lib.rs` and `Anchor.toml`.
