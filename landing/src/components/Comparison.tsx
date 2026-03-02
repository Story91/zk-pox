const FEATURES = [
  'Private location proofs',
  'Anti-spoofing + slashing',
  'No special hardware',
  'Passive GPS collection',
  'On-chain soulbound credentials',
  'Mesh peer attestation',
  'Multiple proof types',
  'DePIN compatible',
]

type Status = 'yes' | 'no' | 'partial'

const PROJECTS: {
  name: string
  highlight?: boolean
  features: Status[]
}[] = [
  {
    name: 'ZK-PoX',
    highlight: true,
    features: ['yes', 'yes', 'yes', 'yes', 'yes', 'yes', 'yes', 'yes'],
  },
  {
    name: 'Helium',
    features: ['no', 'partial', 'no', 'yes', 'no', 'no', 'no', 'yes'],
  },
  {
    name: 'POAP',
    features: ['no', 'no', 'yes', 'no', 'yes', 'no', 'no', 'no'],
  },
  {
    name: 'zkLocus',
    features: ['yes', 'no', 'yes', 'no', 'partial', 'no', 'partial', 'no'],
  },
  {
    name: 'Hivemapper',
    features: ['no', 'partial', 'no', 'yes', 'no', 'no', 'no', 'yes'],
  },
]

function StatusCell({ status }: { status: Status }) {
  if (status === 'yes')
    return <span className="text-accent font-bold">&#x2713;</span>
  if (status === 'no')
    return <span className="text-red/60">&#x2715;</span>
  return <span className="text-dim">~</span>
}

export default function Comparison() {
  return (
    <section className="py-24 border-b border-border">
      <div className="max-w-5xl mx-auto px-6">
        <div className="text-xs tracking-[3px] uppercase text-dim mb-8">
          {'// competitive landscape'}
        </div>
        <h2 className="text-3xl font-bold text-bright mb-4">
          The only system combining all eight.
        </h2>
        <p className="text-text mb-12 max-w-xl">
          DePIN networks leak your home address. POAPs are trivially shared.
          ZK-PoX proves coverage and attendance without revealing where you live.
        </p>

        <div className="overflow-x-auto">
          <table className="w-full text-sm border-collapse">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left py-3 pr-4 text-dim font-normal text-xs">
                  Feature
                </th>
                {PROJECTS.map((p) => (
                  <th
                    key={p.name}
                    className={`py-3 px-3 text-center font-medium text-xs ${
                      p.highlight ? 'text-accent' : 'text-dim'
                    }`}
                  >
                    {p.name}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {FEATURES.map((feat, i) => (
                <tr key={feat} className="border-b border-border/50">
                  <td className="py-3 pr-4 text-text text-xs">{feat}</td>
                  {PROJECTS.map((p) => (
                    <td key={p.name} className="py-3 px-3 text-center">
                      <StatusCell status={p.features[i]} />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="mt-8 p-4 bg-surface border border-border rounded-lg">
          <p className="text-xs text-dim">
            <span className="text-accent font-bold">Honest caveat:</span>{' '}
            ZK-PoX proves where a <em>device</em> was, not a <em>human</em>.
            A Sybil attacker with 20 phones still gets 20 valid histories.
            This is a fundamental GPS limitation — mitigated by economic stake slashing,
            not cryptography alone.
          </p>
        </div>
      </div>
    </section>
  )
}
