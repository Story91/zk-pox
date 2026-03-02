export default function Footer() {
  return (
    <footer className="py-16">
      <div className="max-w-5xl mx-auto px-6">
        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-8 mb-12">
          <div>
            <h2 className="text-2xl font-bold text-bright mb-2">
              Part of the{' '}
              <a
                href="https://0x01.world"
                target="_blank"
                rel="noopener"
                className="text-accent hover:underline"
              >
                0x01
              </a>{' '}
              agent mesh.
            </h2>
            <p className="text-sm text-dim max-w-md">
              ZK-PoX is built on the first agent-native communication protocol.
              Agents discover each other, negotiate value, and build on-chain
              reputation — without human middleware.
            </p>
          </div>
          <div className="flex gap-4">
            <a
              href="https://github.com/0x01-a2a/zk-pox"
              target="_blank"
              rel="noopener"
              className="px-5 py-2.5 bg-purple text-white rounded-md text-sm font-medium hover:bg-purple/80 transition-colors"
            >
              GitHub
            </a>
            <a
              href="https://0x01.world"
              target="_blank"
              rel="noopener"
              className="px-5 py-2.5 border border-border text-text rounded-md text-sm font-medium hover:border-accent hover:text-accent transition-colors"
            >
              0x01.world
            </a>
          </div>
        </div>

        <div className="border-t border-border pt-8 flex flex-col sm:flex-row items-center justify-between gap-4">
          <span className="text-xs text-dim">
            0x01 protocol — open, leaderless, machine-native
          </span>
          <div className="flex gap-6 text-xs text-dim">
            <a
              href="https://github.com/0x01-a2a"
              target="_blank"
              rel="noopener"
              className="hover:text-accent transition-colors"
            >
              GitHub
            </a>
            <a
              href="https://0x01.world/privacy.html"
              target="_blank"
              rel="noopener"
              className="hover:text-accent transition-colors"
            >
              Privacy
            </a>
            <a
              href="mailto:contact@0x01.world"
              className="hover:text-accent transition-colors"
            >
              Contact
            </a>
          </div>
        </div>
      </div>
    </footer>
  )
}
