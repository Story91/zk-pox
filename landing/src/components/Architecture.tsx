export default function Architecture() {
  return (
    <section className="py-24 border-b border-border">
      <div className="max-w-5xl mx-auto px-6">
        <div className="text-xs tracking-[3px] uppercase text-dim mb-8">
          {'// architecture'}
        </div>
        <h2 className="text-3xl font-bold text-bright mb-4">
          End-to-end flow
        </h2>
        <p className="text-text mb-12 max-w-xl">
          From passive GPS collection on your phone to a verified soulbound
          credential on Solana.
        </p>

        <div className="bg-surface border border-border rounded-lg p-6 md:p-8 overflow-x-auto">
          <pre className="text-xs md:text-sm leading-relaxed whitespace-pre font-mono">
            <span className="text-dim">{'  Phone (passive)           Rust Core                  Solana\n'}</span>
            <span className="text-dim">{'       |                        |                         |\n'}</span>
            <span className="text-accent">{'    GPS fix '}</span>
            <span className="text-dim">{'──Ed25519 sign──▶ '}</span>
            <span className="text-purple">{'SignedGPSPoint'}</span>
            <span className="text-dim">{'              |\n'}</span>
            <span className="text-dim">{'       |                        |                         |\n'}</span>
            <span className="text-dim">{'  [encrypted SQLite]          |                         |\n'}</span>
            <span className="text-dim">{'       |                        |                         |\n'}</span>
            <span className="text-bright">{`  "Prove I lived here" `}</span>
            <span className="text-dim">{'──▶ '}</span>
            <span className="text-red">{'Anti-spoof check'}</span>
            <span className="text-dim">{'             |\n'}</span>
            <span className="text-dim">{'       |                     '}</span>
            <span className="text-purple">{'Bulletproofs range proof'}</span>
            <span className="text-dim">{'      |\n'}</span>
            <span className="text-dim">{'       |                     '}</span>
            <span className="text-purple">{'Pedersen commitments'}</span>
            <span className="text-dim">{'          |\n'}</span>
            <span className="text-dim">{'       |                        |                         |\n'}</span>
            <span className="text-dim">{'       |                    '}</span>
            <span className="text-accent">{'ProofResult '}</span>
            <span className="text-dim">{'──────────▶ '}</span>
            <span className="text-accent">{'submit_credential\n'}</span>
            <span className="text-dim">{'       |                        |                   '}</span>
            <span className="text-dim">{'(proof_hash, PDA)\n'}</span>
            <span className="text-dim">{'       |                        |                         |\n'}</span>
            <span className="text-dim">{'       |                  '}</span>
            <span className="text-purple">{'ADVERTISE extension '}</span>
            <span className="text-dim">{'───▶ '}</span>
            <span className="text-purple">{'add_witness\n'}</span>
            <span className="text-dim">{'       |                  '}</span>
            <span className="text-dim">{'(opaque JSON in mesh)      '}</span>
            <span className="text-dim">{'(on-chain attestation)\n'}</span>
          </pre>
        </div>

        <div className="grid grid-cols-3 gap-4 mt-8">
          <div className="text-center p-4 bg-surface border border-border rounded-lg">
            <span className="text-accent text-2xl font-bold block">{'<'}2s</span>
            <span className="text-xs text-dim">proof generation</span>
          </div>
          <div className="text-center p-4 bg-surface border border-border rounded-lg">
            <span className="text-purple text-2xl font-bold block">~2KB</span>
            <span className="text-xs text-dim">proof size</span>
          </div>
          <div className="text-center p-4 bg-surface border border-border rounded-lg">
            <span className="text-accent text-2xl font-bold block">32b</span>
            <span className="text-xs text-dim">per commitment</span>
          </div>
        </div>
      </div>
    </section>
  )
}
