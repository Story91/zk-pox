import { EyeOff, Eye } from 'lucide-react'

const HIDDEN = [
  'Exact GPS coordinates',
  'Exact timestamps of each visit',
  'Your identity (until you link it)',
  'Full GPS trajectory',
  'Which points qualified',
  'Movement patterns during the day',
]

const PROVEN = [
  'Within radius of a committed location',
  'N nights/visits in a time window',
  'Signed by a valid SATI identity',
  'Anti-spoof analysis passed',
  'Pedersen commitments are valid',
  'Aggregate range proof verified',
]

export default function Privacy() {
  return (
    <section className="py-24 border-b border-border">
      <div className="max-w-5xl mx-auto px-6">
        <div className="text-xs tracking-[3px] uppercase text-dim mb-8">
          {'// privacy model'}
        </div>
        <h2 className="text-3xl font-bold text-bright mb-4">
          What stays <span className="text-accent">private</span> vs what is{' '}
          <span className="text-purple">proven</span>
        </h2>
        <p className="text-text mb-16 max-w-xl">
          The verifier learns only that your committed data satisfies the claim.
          Nothing else leaks.
        </p>

        <div className="grid md:grid-cols-2 gap-8">
          <div className="bg-surface border border-border rounded-lg p-6">
            <div className="flex items-center gap-2 mb-6">
              <EyeOff className="w-5 h-5 text-accent" />
              <h3 className="text-sm font-semibold text-accent uppercase tracking-wider">
                Hidden (private inputs)
              </h3>
            </div>
            <ul className="space-y-3">
              {HIDDEN.map((item) => (
                <li key={item} className="flex items-start gap-3 text-sm">
                  <span className="text-accent mt-0.5 shrink-0">&#x2715;</span>
                  <span className="text-text">{item}</span>
                </li>
              ))}
            </ul>
          </div>

          <div className="bg-surface-2 border border-purple/20 rounded-lg p-6">
            <div className="flex items-center gap-2 mb-6">
              <Eye className="w-5 h-5 text-purple" />
              <h3 className="text-sm font-semibold text-purple uppercase tracking-wider">
                Proven (public outputs)
              </h3>
            </div>
            <ul className="space-y-3">
              {PROVEN.map((item) => (
                <li key={item} className="flex items-start gap-3 text-sm">
                  <span className="text-purple mt-0.5 shrink-0">&#x2713;</span>
                  <span className="text-text">{item}</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    </section>
  )
}
