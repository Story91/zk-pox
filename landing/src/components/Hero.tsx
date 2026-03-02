import { useEffect, useState } from 'react'
import { Shield } from 'lucide-react'

const USE_CASES = [
  'Geo-gate airdrops, block bots',
  'Prove DePIN coverage privately',
  'Nomad DAO membership proof',
  'Location-aware agent marketplace',
  'Anti-Sybil with stake slashing',
  'Private event attendance',
]

function RotatingText() {
  const [index, setIndex] = useState(0)
  const [visible, setVisible] = useState(true)

  useEffect(() => {
    const interval = setInterval(() => {
      setVisible(false)
      setTimeout(() => {
        setIndex((i) => (i + 1) % USE_CASES.length)
        setVisible(true)
      }, 400)
    }, 3000)
    return () => clearInterval(interval)
  }, [])

  return (
    <span
      className={`inline-block transition-all duration-400 ${
        visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2'
      }`}
    >
      {USE_CASES[index]}
    </span>
  )
}

function AuroraBackground() {
  return (
    <div className="absolute inset-0 overflow-hidden pointer-events-none">
      <div
        className="absolute -top-1/2 -left-1/2 w-[200%] h-[200%] animate-[aurora_15s_ease-in-out_infinite]"
        style={{
          background:
            'radial-gradient(ellipse at 30% 50%, rgba(123,93,242,0.08) 0%, transparent 50%), radial-gradient(ellipse at 70% 50%, rgba(0,230,118,0.06) 0%, transparent 50%)',
        }}
      />
      <div
        className="absolute -top-1/2 -left-1/2 w-[200%] h-[200%] animate-[aurora_20s_ease-in-out_infinite_reverse]"
        style={{
          background:
            'radial-gradient(ellipse at 60% 30%, rgba(123,93,242,0.05) 0%, transparent 40%), radial-gradient(ellipse at 40% 70%, rgba(0,230,118,0.04) 0%, transparent 40%)',
        }}
      />
    </div>
  )
}

export default function Hero() {
  const [headlineVisible, setHeadlineVisible] = useState(false)

  useEffect(() => {
    setTimeout(() => setHeadlineVisible(true), 100)
  }, [])

  return (
    <section className="relative min-h-screen flex items-center justify-center border-b border-border overflow-hidden">
      <AuroraBackground />

      <div className="relative z-10 max-w-4xl mx-auto px-6 py-24 text-center">
        <div className="inline-flex items-center gap-2 px-4 py-1.5 mb-8 border border-border rounded-full text-xs text-dim">
          <Shield className="w-3.5 h-3.5 text-accent" />
          <span>Built on 0x01 Protocol</span>
        </div>

        <h1
          className={`text-4xl sm:text-5xl md:text-6xl font-bold text-bright leading-tight mb-6 transition-all duration-1000 ${
            headlineVisible
              ? 'opacity-100 translate-y-0'
              : 'opacity-0 translate-y-8'
          }`}
        >
          Zero-Knowledge
          <br />
          <span className="text-purple">Proof-of-Experience</span>
        </h1>

        <p
          className={`text-lg md:text-xl text-text max-w-xl mx-auto mb-4 transition-all duration-1000 delay-200 ${
            headlineVisible
              ? 'opacity-100 translate-y-0'
              : 'opacity-0 translate-y-8'
          }`}
        >
          Prove where you were, without revealing where you are.
        </p>

        <div
          className={`h-8 mb-10 text-accent text-sm transition-all duration-1000 delay-500 ${
            headlineVisible ? 'opacity-100' : 'opacity-0'
          }`}
        >
          <span className="text-dim mr-2">{'//'}</span>
          <RotatingText />
        </div>

        <div
          className={`flex flex-col sm:flex-row gap-4 justify-center transition-all duration-1000 delay-700 ${
            headlineVisible
              ? 'opacity-100 translate-y-0'
              : 'opacity-0 translate-y-8'
          }`}
        >
          <a
            href="https://github.com/0x01-a2a/zk-pox"
            target="_blank"
            rel="noopener"
            className="px-6 py-3 bg-purple text-white rounded-md text-sm font-medium hover:bg-purple/80 transition-colors"
          >
            View on GitHub
          </a>
          <a
            href="#how-it-works"
            className="px-6 py-3 border border-border text-text rounded-md text-sm font-medium hover:border-accent hover:text-accent transition-colors"
          >
            How it works
          </a>
        </div>

        <div
          className={`mt-16 flex justify-center gap-8 text-xs text-dim transition-all duration-1000 delay-1000 ${
            headlineVisible ? 'opacity-100' : 'opacity-0'
          }`}
        >
          <div>
            <span className="block text-accent text-lg font-bold">61</span>
            tests passing
          </div>
          <div className="w-px bg-border" />
          <div>
            <span className="block text-accent text-lg font-bold">2.6K+</span>
            lines of code
          </div>
          <div className="w-px bg-border" />
          <div>
            <span className="block text-accent text-lg font-bold">~2KB</span>
            proof size
          </div>
        </div>
      </div>
    </section>
  )
}
