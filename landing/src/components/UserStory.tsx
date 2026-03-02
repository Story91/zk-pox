import { useState } from 'react'

const STORIES = [
  {
    name: 'ETH Denver',
    role: 'airdrop',
    scenario: 'Geo-gated airdrop — prove attendance, block bots',
    conversation: [
      {
        who: 'Protocol',
        text: '"ADVERTISE: Airdrop for ETH Denver attendees. Must prove 6+ hours within 200m of venue on March 1. Budget: 50,000 USDC pool."',
      },
      {
        who: 'Agent',
        text: '"You have 14 signed GPS points within the venue geofence today. Generating ATTENDANCE proof..."',
      },
      {
        who: 'Agent',
        text: '"Proof generated (2.1 KB). Anti-spoof: Clean. 2 mesh witnesses confirmed proximity. Submitting to airdrop contract. Claim amount: 12.50 USDC."',
      },
    ],
  },
  {
    name: 'Helium Node',
    role: 'DePIN',
    scenario: 'Prove coverage area without revealing home address',
    conversation: [
      {
        who: 'DePIN',
        text: '"Verify: operator provides coverage in Warsaw district Mokotow. Must prove 30+ days of stable presence. No exact address required."',
      },
      {
        who: 'Agent',
        text: '"Generating STABILITY proof from 90 days of GPS data... Location variance: 0.8km (below 2km threshold). 28/30 nights confirmed."',
      },
      {
        who: 'Agent',
        text: '"Proof submitted on-chain. Coverage credential attached to your SATI identity. Your home address stays private — verifier only sees: \'stable presence in region H for 30+ days.\'"',
      },
    ],
  },
  {
    name: 'Zuzalu',
    role: 'nomad DAO',
    scenario: 'Prove participation in 3 pop-up cities',
    conversation: [
      {
        who: 'DAO',
        text: '"Membership requires visiting 3 Zuzalu pop-up locations for 3+ days each in the past 12 months."',
      },
      {
        who: 'Agent',
        text: '"Found qualifying stays: Montenegro (7 days), Thailand (5 days), Costa Rica (4 days). Generating TRAVEL proof..."',
      },
      {
        who: 'Agent',
        text: '"Proof proves: \'3 distinct geofences visited, 3+ days each, within 12-month window.\' No flight data, no passport stamps, no exact dates revealed. Submitting to DAO contract."',
      },
    ],
  },
]

export default function UserStory() {
  const [active, setActive] = useState(0)
  const story = STORIES[active]

  return (
    <section className="py-24 border-b border-border">
      <div className="max-w-5xl mx-auto px-6">
        <div className="text-xs tracking-[3px] uppercase text-dim mb-8">
          {'// real scenarios'}
        </div>
        <h2 className="text-3xl font-bold text-bright mb-4">
          Smart contracts verify. Humans stay private.
        </h2>
        <p className="text-text mb-12 max-w-xl">
          ZK-PoX works best where the verifier is a smart contract, trust is
          low, and spoofing is lucrative.
        </p>

        <div className="flex gap-3 mb-8 overflow-x-auto pb-2">
          {STORIES.map((s, i) => (
            <button
              key={s.name}
              onClick={() => setActive(i)}
              className={`shrink-0 px-4 py-2 rounded-md text-xs font-medium transition-all ${
                active === i
                  ? 'bg-purple text-white'
                  : 'bg-surface border border-border text-dim hover:text-text hover:border-purple/30'
              }`}
            >
              {s.name} ({s.role})
            </button>
          ))}
        </div>

        <div className="bg-surface border border-border rounded-lg overflow-hidden">
          <div className="px-5 py-3 border-b border-border flex items-center justify-between">
            <span className="text-xs text-dim">{story.scenario}</span>
            <span className="text-xs text-accent">0x01 mesh</span>
          </div>
          <div className="p-5 space-y-4">
            {story.conversation.map((msg, i) => (
              <div
                key={i}
                className={`flex gap-3 ${
                  msg.who !== 'Agent' ? 'justify-end' : ''
                }`}
              >
                {msg.who === 'Agent' && (
                  <span className="shrink-0 w-8 h-8 rounded-full bg-purple/20 text-purple text-xs flex items-center justify-center font-bold">
                    AI
                  </span>
                )}
                <div
                  className={`max-w-lg rounded-lg px-4 py-3 text-sm leading-relaxed ${
                    msg.who === 'Agent'
                      ? 'bg-surface-2 text-text border border-border'
                      : 'bg-accent/5 text-bright border border-accent/20'
                  }`}
                >
                  {msg.text}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  )
}
