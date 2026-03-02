import { useEffect, useRef, useState } from 'react'
import { Gauge, Users, Coins } from 'lucide-react'

function CountUp({ end, suffix = '' }: { end: number; suffix?: string }) {
  const [value, setValue] = useState(0)
  const ref = useRef<HTMLSpanElement>(null)
  const started = useRef(false)

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !started.current) {
          started.current = true
          const duration = 1500
          const startTime = performance.now()
          const animate = (now: number) => {
            const elapsed = now - startTime
            const progress = Math.min(elapsed / duration, 1)
            const eased = 1 - Math.pow(1 - progress, 3)
            setValue(Math.floor(eased * end))
            if (progress < 1) requestAnimationFrame(animate)
          }
          requestAnimationFrame(animate)
        }
      },
      { threshold: 0.5 },
    )
    if (ref.current) observer.observe(ref.current)
    return () => observer.disconnect()
  }, [end])

  return (
    <span ref={ref}>
      {value}
      {suffix}
    </span>
  )
}

const LAYERS = [
  {
    icon: Gauge,
    title: 'Temporal Analysis',
    desc: 'Velocity checks, teleportation detection, zero-noise mock GPS detection. Runs before proof generation — spoofed data is blocked automatically.',
    stat: <CountUp end={61} />,
    statLabel: 'tests passing',
    color: 'text-accent',
  },
  {
    icon: Users,
    title: 'Peer Corroboration',
    desc: 'Nearby agents verify proofs from ADVERTISE extensions and attest via on-chain add_witness. Spoofing 3+ staked devices simultaneously is exponentially harder.',
    stat: (
      <>
        <CountUp end={98} suffix="." />
        72%
      </>
    ),
    statLabel: 'spoof detection rate',
    color: 'text-purple',
  },
  {
    icon: Coins,
    title: 'Economic Deterrence',
    desc: 'Every agent stakes 10 USDC. Fake proofs trigger a CHALLENGE — guilty agents get slashed. Cost of attack exceeds reward.',
    stat: (
      <>
        <CountUp end={10} /> USDC
      </>
    ),
    statLabel: 'stake at risk',
    color: 'text-accent',
  },
]

export default function AntiSpoof() {
  return (
    <section className="py-24 border-b border-border">
      <div className="max-w-5xl mx-auto px-6">
        <div className="text-xs tracking-[3px] uppercase text-dim mb-8">
          {'// anti-spoofing'}
        </div>
        <h2 className="text-3xl font-bold text-bright mb-4">
          Three layers of defense.
        </h2>
        <p className="text-text mb-16 max-w-xl">
          GPS can be faked. ZK-PoX addresses this with temporal analysis,
          peer witnesses, and economic stakes.
        </p>

        <div className="space-y-6">
          {LAYERS.map((layer) => (
            <div
              key={layer.title}
              className="flex flex-col md:flex-row items-start gap-6 bg-surface border border-border rounded-lg p-6 hover:border-purple/30 transition-colors"
            >
              <div className="flex items-center gap-4 md:w-48 shrink-0">
                <layer.icon className={`w-6 h-6 ${layer.color}`} />
                <div>
                  <h3 className="text-sm font-semibold text-bright">
                    {layer.title}
                  </h3>
                </div>
              </div>
              <p className="text-sm text-text flex-1 leading-relaxed">
                {layer.desc}
              </p>
              <div className="text-right shrink-0 md:w-32">
                <span className={`text-2xl font-bold ${layer.color}`}>
                  {layer.stat}
                </span>
                <span className="block text-xs text-dim mt-1">
                  {layer.statLabel}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}
