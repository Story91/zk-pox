import { Home, ArrowRightLeft, CalendarCheck, Ban, Activity, Globe } from 'lucide-react'

const PROOFS = [
  {
    type: 'ATTENDANCE',
    icon: CalendarCheck,
    desc: 'Within R meters of E for T+ hours on date D',
    use: 'Geo-gated airdrops, IRL quests, anti-bot event verification',
    color: 'text-accent',
  },
  {
    type: 'RESIDENCY',
    icon: Home,
    desc: 'Near location H for N+ nights in period P',
    use: 'DePIN coverage, agent marketplace locality',
    color: 'text-purple',
  },
  {
    type: 'STABILITY',
    icon: Activity,
    desc: 'Location variance below threshold over period P',
    use: 'DePIN uptime proof, consistent coverage for Helium/GEODNET',
    color: 'text-accent',
  },
  {
    type: 'TRAVEL',
    icon: Globe,
    desc: 'In N distinct regions during period P',
    use: 'Nomad DAO membership, network state participation proof',
    color: 'text-purple',
  },
  {
    type: 'COMMUTE',
    icon: ArrowRightLeft,
    desc: 'Traveled A to B, D days/week, for W weeks',
    use: 'Agent work pattern verification, delivery coverage',
    color: 'text-accent',
  },
  {
    type: 'ABSENCE',
    icon: Ban,
    desc: 'NOT within R meters of X during period P',
    use: 'Geo-exclusion compliance, trustless zoning',
    color: 'text-red',
  },
]

export default function ProofTypes() {
  return (
    <section className="py-24 border-b border-border">
      <div className="max-w-5xl mx-auto px-6">
        <div className="text-xs tracking-[3px] uppercase text-dim mb-8">
          {'// proof types'}
        </div>
        <h2 className="text-3xl font-bold text-bright mb-4">
          Six claim types. One protocol.
        </h2>
        <p className="text-text mb-16 max-w-xl">
          Each proof type is a different assertion over your GPS history.
          All use the same Bulletproofs engine.
        </p>

        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {PROOFS.map((p) => (
            <div
              key={p.type}
              className="bg-surface border border-border rounded-lg p-5 hover:border-purple/30 transition-colors group"
            >
              <div className="flex items-center gap-3 mb-3">
                <p.icon
                  className={`w-4 h-4 ${p.color} group-hover:scale-110 transition-transform`}
                />
                <span className={`text-xs font-bold tracking-wider ${p.color}`}>
                  {p.type}
                </span>
              </div>
              <p className="text-sm text-bright mb-2">{p.desc}</p>
              <p className="text-xs text-dim">{p.use}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}
