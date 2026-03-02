import { useState } from 'react'
import { MapPin, Lock, CheckCircle } from 'lucide-react'

const STEPS = [
  {
    num: '01',
    title: 'Collect',
    subtitle: 'Passive. Silent. Signed.',
    desc: 'Your phone records GPS coordinates every 5 minutes, each signed with your Ed25519 cryptographic identity. Stored in encrypted local SQLite. Never leaves your device.',
    icon: MapPin,
    color: 'text-accent',
    borderColor: 'hover:border-accent/50',
  },
  {
    num: '02',
    title: 'Prove',
    subtitle: 'Bulletproofs on committed coordinates.',
    desc: 'Aggregate range proofs over Pedersen-committed GPS offsets. Anti-spoofing gate blocks fake data. The proof says "N points are inside this geofence" without revealing which ones.',
    icon: Lock,
    color: 'text-purple',
    borderColor: 'hover:border-purple/50',
  },
  {
    num: '03',
    title: 'Verify',
    subtitle: 'On-chain. Soulbound. Witnessed.',
    desc: 'Credential is submitted to Solana as a soulbound PDA. Mesh peers verify the proof off-chain and add witness attestations. Tampered proofs are rejected cryptographically.',
    icon: CheckCircle,
    color: 'text-accent',
    borderColor: 'hover:border-accent/50',
  },
]

function SpotlightCard({
  children,
  borderColor,
}: {
  children: React.ReactNode
  borderColor: string
}) {
  const [pos, setPos] = useState({ x: 0, y: 0 })
  const [hovering, setHovering] = useState(false)

  return (
    <div
      className={`relative bg-surface border border-border rounded-lg p-6 transition-all duration-300 ${borderColor} group`}
      onMouseMove={(e) => {
        const rect = e.currentTarget.getBoundingClientRect()
        setPos({ x: e.clientX - rect.left, y: e.clientY - rect.top })
      }}
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
    >
      {hovering && (
        <div
          className="absolute inset-0 rounded-lg pointer-events-none transition-opacity duration-300"
          style={{
            background: `radial-gradient(300px circle at ${pos.x}px ${pos.y}px, rgba(123,93,242,0.06), transparent 60%)`,
          }}
        />
      )}
      <div className="relative z-10">{children}</div>
    </div>
  )
}

export default function HowItWorks() {
  return (
    <section id="how-it-works" className="py-24 border-b border-border">
      <div className="max-w-5xl mx-auto px-6">
        <div className="text-xs tracking-[3px] uppercase text-dim mb-8">
          {'// how it works'}
        </div>
        <h2 className="text-3xl font-bold text-bright mb-4">
          Three steps. No human action.
        </h2>
        <p className="text-text mb-16 max-w-xl">
          GPS collection is passive. Proof generation is on-demand. Verification
          is cryptographic. Your phone does everything.
        </p>

        <div className="grid md:grid-cols-3 gap-6">
          {STEPS.map((step) => (
            <SpotlightCard key={step.num} borderColor={step.borderColor}>
              <div className="flex items-center gap-3 mb-4">
                <span className="text-dim text-xs">{step.num}</span>
                <step.icon className={`w-5 h-5 ${step.color}`} />
              </div>
              <h3 className="text-xl font-semibold text-bright mb-1">
                {step.title}
              </h3>
              <p className={`text-xs mb-3 ${step.color}`}>{step.subtitle}</p>
              <p className="text-sm text-text leading-relaxed">{step.desc}</p>
            </SpotlightCard>
          ))}
        </div>
      </div>
    </section>
  )
}
