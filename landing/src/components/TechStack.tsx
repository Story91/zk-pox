const TECH = [
  { name: 'Bulletproofs', desc: 'ZK range proofs, no trusted setup', href: 'https://crates.io/crates/bulletproofs' },
  { name: 'Pedersen Commitments', desc: 'Hiding + binding on GPS offsets', href: 'https://crates.io/crates/curve25519-dalek-ng' },
  { name: 'Merlin Transcripts', desc: 'Fiat-Shamir heuristic', href: 'https://crates.io/crates/merlin' },
  { name: 'Ed25519', desc: 'GPS point signing', href: 'https://crates.io/crates/ed25519-dalek' },
  { name: 'Solana Anchor', desc: 'On-chain credential program', href: 'https://www.anchor-lang.com/' },
  { name: 'JNI (Android)', desc: 'Native Rust on mobile', href: 'https://crates.io/crates/jni' },
  { name: 'React Native', desc: 'Cross-platform mobile UI', href: 'https://reactnative.dev/' },
  { name: 'libp2p', desc: 'Peer-to-peer mesh network', href: 'https://libp2p.io/' },
]

export default function TechStack() {
  return (
    <section className="py-24 border-b border-border">
      <div className="max-w-5xl mx-auto px-6">
        <div className="text-xs tracking-[3px] uppercase text-dim mb-8">
          {'// technical stack'}
        </div>
        <h2 className="text-3xl font-bold text-bright mb-4">
          Built on proven cryptography.
        </h2>
        <p className="text-text mb-12 max-w-xl">
          Every primitive is battle-tested. Bulletproofs powers Monero.
          Ed25519 powers SSH. Solana secures billions.
        </p>

        <div className="flex flex-wrap gap-3">
          {TECH.map((t) => (
            <a
              key={t.name}
              href={t.href}
              target="_blank"
              rel="noopener"
              className="group flex items-center gap-3 bg-surface border border-border rounded-lg px-4 py-3 hover:border-purple/40 transition-colors"
            >
              <span className="text-sm font-medium text-bright group-hover:text-purple transition-colors">
                {t.name}
              </span>
              <span className="text-xs text-dim hidden sm:inline">
                {t.desc}
              </span>
            </a>
          ))}
        </div>
      </div>
    </section>
  )
}
